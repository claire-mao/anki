// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::MemoryScore;
use anki_proto::brainlift::PerformanceAttempt;
use anki_proto::brainlift::PerformanceScore;
use anki_proto::brainlift::ReadinessScore;
use anki_proto::stats::TopicMasteryEntry;
use anki_proto::stats::TopicMasteryResponse;
use anki_proto::stats::TopicMasteryRequest;

use crate::brainlift::calibration::apply_calibration_honesty;
use crate::brainlift::calibration::maintain_readiness_calibration;
use crate::brainlift::calibration::OutcomeInputs;
use crate::brainlift::calibration::ReadinessPredictionSnapshot;
use crate::brainlift::brainlift_storage;
use crate::brainlift::compute_coverage;
use crate::brainlift::is_topic_covered;
use crate::brainlift::readiness::compute_memory_score;
use crate::brainlift::readiness::compute_performance_score;
use crate::brainlift::readiness::compute_readiness_score;
use crate::brainlift::readiness::MemoryInputs;
use crate::brainlift::readiness::PerformanceInputs;
use crate::brainlift::storage::PerformanceAttemptRow;
use crate::brainlift::GreCoverage;
use crate::brainlift::TOPIC_TAG_PREFIX;
use crate::collection::Collection;
use crate::error::Result;
use crate::timestamp::TimestampMillis;

use super::GRE_DECK_NAME;

pub(crate) struct BrainliftSignals {
    pub memory: MemoryScore,
    pub performance: PerformanceScore,
    pub readiness: ReadinessScore,
    pub mastery: TopicMasteryResponse,
    pub coverage: GreCoverage,
    pub recent_attempts: Vec<PerformanceAttemptRow>,
    pub practice_by_topic: HashMap<String, (u32, u32)>,
    pub computed_at_millis: i64,
}

impl Collection {
    pub(crate) fn load_brainlift_signals(
        &mut self,
        recent_activity_limit: u32,
    ) -> Result<BrainliftSignals> {
        let mastery = self.compute_topic_mastery(TopicMasteryRequest {
            search: format!("deck:\"{}\"", GRE_DECK_NAME),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            mastery_threshold: None,
            min_reviews: 1,
        })?;
        let summary = mastery.summary.clone().unwrap_or_default();

        let studied_tags: Vec<String> = mastery
            .topics
            .iter()
            .filter(|topic| topic.studied_cards > 0)
            .map(|topic| topic.topic_id.clone())
            .collect();
        let studied_tag_refs: Vec<&str> = studied_tags.iter().map(String::as_str).collect();
        let coverage = compute_coverage(&studied_tag_refs);

        let memory = compute_memory_score(&MemoryInputs {
            fsrs_enabled: mastery.fsrs_enabled,
            overall_retrievability: summary.overall_avg_retrievability,
            coverage_ratio: summary.coverage_ratio,
            studied_cards: summary.studied_cards,
            topics: mastery.topics.clone(),
        });

        let storage = brainlift_storage(self)?;
        let (correct, total) = storage.performance_summary()?;
        let performance = compute_performance_score(&PerformanceInputs { correct, total });
        let practice_by_topic: HashMap<String, (u32, u32)> = storage
            .performance_by_topic()?
            .into_iter()
            .map(|(topic, correct, total)| (topic, (correct, total)))
            .collect();
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
        let calibration = maintain_readiness_calibration(
            storage,
            snapshot.as_ref(),
            &outcome_inputs,
        )?;
        let readiness = apply_calibration_honesty(readiness, &calibration);

        Ok(BrainliftSignals {
            memory,
            performance,
            readiness,
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
