// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::EstimatedGreScore;
use anki_proto::brainlift::MemoryScore;
use anki_proto::brainlift::PerformanceAttempt;
use anki_proto::brainlift::PerformanceScore;
use anki_proto::brainlift::ReadinessScore;
use anki_proto::stats::TopicMasteryEntry;
use anki_proto::stats::TopicMasteryRequest;
use anki_proto::stats::TopicMasteryResponse;

use super::gre_deck_search_str;
use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::calibration::apply_calibration_honesty;
use crate::gre_atlas::calibration::maintain_readiness_calibration;
use crate::gre_atlas::calibration::OutcomeInputs;
use crate::gre_atlas::calibration::ReadinessPredictionSnapshot;
use crate::gre_atlas::compute_coverage;
use crate::gre_atlas::estimated_gre::compute_estimated_gre_score;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::is_topic_covered;
use crate::gre_atlas::readiness::compute_memory_score;
use crate::gre_atlas::readiness::compute_performance_score;
use crate::gre_atlas::readiness::compute_readiness_score;
use crate::gre_atlas::readiness::MemoryInputs;
use crate::gre_atlas::readiness::PerformanceInputs;
use crate::gre_atlas::storage::PerformanceAttemptRow;
use crate::gre_atlas::GreCoverage;
use crate::gre_atlas::TOPIC_TAG_PREFIX;
use crate::timestamp::TimestampMillis;

pub(crate) struct GreAtlasSignals {
    pub memory: MemoryScore,
    pub performance: PerformanceScore,
    pub readiness: ReadinessScore,
    pub estimated_gre: EstimatedGreScore,
    pub mastery: TopicMasteryResponse,
    pub coverage: GreCoverage,
    pub recent_attempts: Vec<PerformanceAttemptRow>,
    pub practice_by_topic: HashMap<String, (u32, u32)>,
    pub computed_at_millis: i64,
}

/// Cached GRE signal bundle (everything except `recent_attempts`, which varies
/// by caller limit).
#[derive(Clone, Debug)]
pub(crate) struct GreAtlasSignalsCache {
    collection_mod: TimestampMillis,
    /// Monotonic sidecar revision; increments on every practice/sync write.
    gre_atlas_usn: i32,
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
    estimated_gre: EstimatedGreScore,
    mastery: TopicMasteryResponse,
    coverage: GreCoverage,
    practice_by_topic: HashMap<String, (u32, u32)>,
    computed_at_millis: i64,
}

impl GreAtlasSignalsCache {
    fn into_signals(self, recent_attempts: Vec<PerformanceAttemptRow>) -> GreAtlasSignals {
        GreAtlasSignals {
            memory: self.memory,
            performance: self.performance,
            readiness: self.readiness,
            estimated_gre: self.estimated_gre,
            mastery: self.mastery,
            coverage: self.coverage,
            practice_by_topic: self.practice_by_topic,
            computed_at_millis: self.computed_at_millis,
            recent_attempts,
        }
    }
}

impl Collection {
    pub(crate) fn gre_collection_revision(&self) -> Result<TimestampMillis> {
        Ok(self.storage.get_collection_timestamps()?.collection_change)
    }

    pub(crate) fn load_gre_atlas_signals(
        &mut self,
        recent_activity_limit: u32,
    ) -> Result<GreAtlasSignals> {
        let collection_mod = self.gre_collection_revision()?;
        let gre_atlas_usn = {
            let storage = gre_atlas_storage(self)?;
            storage.sync_status()?.current_usn
        };

        if let Some(cached) = self.state.gre_atlas_signals_cache.clone() {
            if cached.collection_mod == collection_mod && cached.gre_atlas_usn == gre_atlas_usn {
                let storage = gre_atlas_storage(self)?;
                let recent_attempts = storage.recent_attempts("", recent_activity_limit.max(1))?;
                return Ok(cached.into_signals(recent_attempts));
            }
        }

        self.compute_gre_atlas_signals(collection_mod, recent_activity_limit)
    }

    fn compute_gre_atlas_signals(
        &mut self,
        collection_mod: TimestampMillis,
        recent_activity_limit: u32,
    ) -> Result<GreAtlasSignals> {
        let mastery = self.compute_topic_mastery(TopicMasteryRequest {
            search: gre_deck_search_str().into(),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            mastery_threshold: None,
            min_reviews: 1,
        })?;
        let summary = mastery.summary.clone().unwrap_or_default();

        let coverage = compute_coverage(
            &mastery
                .topics
                .iter()
                .filter(|topic| topic.studied_cards > 0)
                .map(|topic| topic.topic_id.as_str())
                .collect::<Vec<_>>(),
        );

        let memory = compute_memory_score(&MemoryInputs {
            fsrs_enabled: mastery.fsrs_enabled,
            overall_retrievability: summary.overall_avg_retrievability,
            coverage_ratio: summary.coverage_ratio,
            studied_cards: summary.studied_cards,
            topics: &mastery.topics,
        });

        let storage = gre_atlas_storage(self)?;
        let ((correct, total), practice_by_topic) = storage.performance_stats()?;
        let performance = compute_performance_score(&PerformanceInputs { correct, total });
        let recent_attempts = storage.recent_attempts("", recent_activity_limit.max(1))?;

        let computed_at_millis = TimestampMillis::now().0;
        let readiness = compute_readiness_score(&memory, &performance, computed_at_millis);
        let snapshot = prediction_snapshot(&memory, &performance, &readiness);
        let outcome_inputs = OutcomeInputs {
            memory_score: memory.value.unwrap_or(0.0),
            performance_score: performance.value.unwrap_or(0.0),
            coverage_ratio: memory.coverage_ratio,
            practice_correct: correct,
            practice_total: total,
        };
        let calibration =
            maintain_readiness_calibration(storage, snapshot.as_ref(), &outcome_inputs)?;
        let readiness = apply_calibration_honesty(readiness, &calibration);
        let estimated_gre = compute_estimated_gre_score(
            &memory,
            &performance,
            &readiness,
            &coverage,
            &mastery.topics,
            &practice_by_topic,
        );

        let gre_atlas_usn = storage.sync_status()?.current_usn;
        self.state.gre_atlas_signals_cache = Some(GreAtlasSignalsCache {
            collection_mod,
            gre_atlas_usn,
            memory: memory.clone(),
            performance: performance.clone(),
            readiness: readiness.clone(),
            estimated_gre: estimated_gre.clone(),
            mastery: mastery.clone(),
            coverage: coverage.clone(),
            practice_by_topic: practice_by_topic.clone(),
            computed_at_millis,
        });

        Ok(GreAtlasSignals {
            memory,
            performance,
            readiness,
            estimated_gre,
            mastery,
            coverage,
            recent_attempts,
            practice_by_topic,
            computed_at_millis,
        })
    }
}

pub(crate) fn attempt_row_to_proto(row: &PerformanceAttemptRow) -> PerformanceAttempt {
    PerformanceAttempt {
        question_id: row.question_id.clone(),
        topic: row.topic.clone(),
        answered_at_secs: row.answered_at_secs.0,
        answer: row.answer.clone(),
        correct: row.correct,
        response_time_ms: row.response_time_ms,
        confidence: row.confidence,
        difficulty: row.difficulty,
        session_id: row.session_id.clone(),
    }
}

pub(crate) fn mastery_map(topics: &[TopicMasteryEntry]) -> HashMap<String, TopicMasteryEntry> {
    topics
        .iter()
        .map(|topic| (topic.topic_id.clone(), topic.clone()))
        .collect()
}

pub(crate) fn practice_stats_for_topic(
    topic_id: &str,
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> Option<(u32, u32)> {
    let mut best: Option<(u32, u32)> = None;
    for (topic, stats) in practice_by_topic {
        if topic == topic_id
            || (topic.len() > topic_id.len()
                && topic.starts_with(topic_id)
                && topic[topic_id.len()..].starts_with("::"))
            || (topic_id.len() > topic.len()
                && topic_id.starts_with(topic)
                && topic_id[topic.len()..].starts_with("::"))
        {
            let entry = best.get_or_insert((0, 0));
            entry.0 += stats.0;
            entry.1 += stats.1;
        }
    }
    best.filter(|(_, total)| *total > 0)
}

pub(crate) fn observed_tags_from_mastery(topics: &[TopicMasteryEntry]) -> Vec<String> {
    topics
        .iter()
        .filter(|topic| topic.studied_cards > 0)
        .map(|topic| topic.topic_id.clone())
        .collect()
}

pub(crate) fn is_leaf_covered(
    leaf_id: &str,
    observed_tags: &[&str],
    mastery: &HashMap<String, TopicMasteryEntry>,
) -> bool {
    is_topic_covered(leaf_id, observed_tags)
        || mastery
            .get(leaf_id)
            .is_some_and(|entry| entry.studied_cards > 0)
}

fn prediction_snapshot(
    memory: &MemoryScore,
    performance: &PerformanceScore,
    readiness: &ReadinessScore,
) -> Option<ReadinessPredictionSnapshot> {
    if !readiness.sufficient_data {
        return None;
    }
    Some(ReadinessPredictionSnapshot {
        projected_score: readiness.projected_score?,
        projected_score_low: readiness.projected_score_low,
        projected_score_high: readiness.projected_score_high,
        memory_score: memory.value?,
        performance_score: performance.value?,
        coverage_ratio: readiness.coverage_ratio,
        confidence_level: readiness.confidence_level.clone(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::gre_atlas::gre_atlas_storage;

    fn isolated_col() -> Result<crate::collection::Collection> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        CollectionBuilder::new(dir.path().join("test.anki2")).build()
    }

    #[test]
    fn load_gre_atlas_signals_uses_cache_on_second_call() -> Result<()> {
        let mut col = isolated_col()?;
        let q_id;
        let q_topic;
        let q_difficulty;
        let q_answer;
        let session_id;
        {
            let storage = gre_atlas_storage(&mut col)?;
            let q = storage.list_questions("", 1)?.pop().unwrap();
            q_id = q.id.clone();
            q_topic = q.topic.clone();
            q_difficulty = q.difficulty;
            q_answer = q.correct_answer.clone();
            let session = storage.create_session("practice")?;
            session_id = session.id.clone();
            storage.record_attempt(
                &q_id,
                &q_topic,
                q_difficulty,
                &q_answer,
                true,
                1000,
                None,
                Some(&session_id),
            )?;
        }

        let signals = col.load_gre_atlas_signals(5)?;
        let collection_mod = col.gre_collection_revision()?;
        let usn = gre_atlas_storage(&mut col)?.sync_status()?.current_usn;
        assert!(col.state.gre_atlas_signals_cache.is_some());
        let cached = col.state.gre_atlas_signals_cache.as_ref().unwrap();
        assert_eq!(cached.collection_mod, collection_mod);
        assert_eq!(cached.gre_atlas_usn, usn);
        assert_eq!(signals.performance.attempt_count, 1);

        {
            let storage = gre_atlas_storage(&mut col)?;
            storage.record_attempt(
                &q_id,
                &q_topic,
                q_difficulty,
                "wrong",
                false,
                900,
                None,
                Some(&session_id),
            )?;
        }
        let signals = col.load_gre_atlas_signals(10)?;
        assert_eq!(signals.performance.attempt_count, 2);

        col.load_gre_atlas_signals(10)?;
        assert_eq!(
            col.state
                .gre_atlas_signals_cache
                .as_ref()
                .unwrap()
                .collection_mod,
            collection_mod
        );
        Ok(())
    }
}
