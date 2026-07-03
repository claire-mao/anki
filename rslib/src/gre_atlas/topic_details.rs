// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GetTopicDetailsRequest;
use anki_proto::brainlift::MemoryScore;
use anki_proto::brainlift::PerformanceScore;
use anki_proto::brainlift::ReadinessScore;
use anki_proto::brainlift::TopicDetailsResponse;
use anki_proto::brainlift::TopicReadinessContribution;
use anki_proto::stats::TopicMasteryEntry;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::is_topic_covered;
use crate::gre_atlas::questions::stored_question_to_proto;
use crate::gre_atlas::readiness;
use crate::gre_atlas::signals::attempt_row_to_proto;
use crate::gre_atlas::signals::is_leaf_covered;
use crate::gre_atlas::signals::mastery_map;
use crate::gre_atlas::signals::observed_tags_from_mastery;
use crate::gre_atlas::signals::practice_stats_for_topic;
use crate::gre_atlas::GreCatalog;
use crate::gre_atlas::TopicDef;
use crate::invalid_input;

impl Collection {
    pub fn gre_atlas_get_topic_details(
        &mut self,
        input: GetTopicDetailsRequest,
    ) -> Result<TopicDetailsResponse> {
        let topic_id = input.topic_id.trim();
        if topic_id.is_empty() {
            invalid_input!("topic_id is required");
        }
        let topic_def = GreCatalog::topic_by_id(topic_id).ok_or_else(|| {
            crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!("unknown GRE topic: {topic_id}")),
            }
        })?;

        let signals = self.load_gre_atlas_signals(1)?;
        let mastery_by_id = mastery_map(&signals.mastery.topics);
        let observed_tags = observed_tags_from_mastery(&signals.mastery.topics);
        let observed_refs: Vec<&str> = observed_tags.iter().map(String::as_str).collect();

        let covered = if topic_def.is_leaf() {
            is_leaf_covered(topic_id, &observed_refs, &mastery_by_id)
        } else {
            is_topic_covered(topic_id, &observed_refs)
        };

        let mastery_entry = mastery_by_id.get(topic_id);
        let practice_stats = practice_stats_for_topic(topic_id, &signals.practice_by_topic);
        let (practice_correct, practice_total) = practice_stats.unwrap_or((0, 0));
        let practice_accuracy = if practice_total > 0 {
            Some(practice_correct as f32 / practice_total as f32 * 100.0)
        } else {
            None
        };

        let question_limit = if input.practice_question_limit == 0 {
            10
        } else {
            input.practice_question_limit
        };
        let attempt_limit = if input.recent_attempt_limit == 0 {
            10
        } else {
            input.recent_attempt_limit
        };

        let storage = gre_atlas_storage(self)?;
        let practice_questions = storage
            .list_questions(topic_id, question_limit)?
            .into_iter()
            .map(stored_question_to_proto)
            .collect();
        let recent_attempts = storage
            .recent_attempts(topic_id, attempt_limit)?
            .into_iter()
            .map(|row| attempt_row_to_proto(&row))
            .collect();

        let total_studied = signals
            .mastery
            .summary
            .as_ref()
            .map(|summary| summary.studied_cards)
            .unwrap_or(0);
        let (_, global_total) = storage.performance_summary()?;

        let readiness_contribution = compute_topic_readiness_contribution(
            topic_def,
            covered,
            mastery_entry,
            practice_correct,
            practice_total,
            total_studied,
            global_total,
            &signals.memory,
            &signals.performance,
            &signals.readiness,
        );

        Ok(TopicDetailsResponse {
            topic_id: topic_id.to_string(),
            display_name: topic_def.display_name.to_string(),
            section: topic_def.section.display_name().to_string(),
            exam_weight: topic_def.exam_weight,
            is_leaf: topic_def.is_leaf(),
            studied_cards: mastery_entry.map(|entry| entry.studied_cards).unwrap_or(0),
            total_cards: mastery_entry.map(|entry| entry.total_cards).unwrap_or(0),
            mastered_cards: mastery_entry.map(|entry| entry.mastered_cards).unwrap_or(0),
            memory_score: mastery_entry.and_then(|entry| {
                if entry.studied_cards > 0 {
                    Some(entry.avg_retrievability * 100.0)
                } else {
                    None
                }
            }),
            avg_retrievability_low: mastery_entry.and_then(|entry| {
                if entry.studied_cards > 0 {
                    Some(entry.avg_retrievability_low * 100.0)
                } else {
                    None
                }
            }),
            avg_retrievability_high: mastery_entry.and_then(|entry| {
                if entry.studied_cards > 0 {
                    Some(entry.avg_retrievability_high * 100.0)
                } else {
                    None
                }
            }),
            total_reviews: mastery_entry.map(|entry| entry.total_reviews).unwrap_or(0),
            practice_accuracy,
            practice_correct,
            practice_total,
            covered,
            readiness_contribution: Some(readiness_contribution),
            practice_questions,
            recent_attempts,
            global_readiness_score: signals.readiness.projected_score,
            global_readiness_summary: signals.readiness.evidence_summary,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_topic_readiness_contribution(
    topic_def: &TopicDef,
    covered: bool,
    mastery_entry: Option<&TopicMasteryEntry>,
    practice_correct: u32,
    practice_total: u32,
    total_studied_cards: u32,
    global_practice_total: u32,
    global_memory: &MemoryScore,
    global_performance: &PerformanceScore,
    global_readiness: &ReadinessScore,
) -> TopicReadinessContribution {
    let section_weight = topic_def.section.official_section_weight();
    let exam_weight = topic_def.exam_weight;

    let coverage_contribution = if topic_def.is_leaf() {
        Some(if covered {
            readiness::COVERAGE_WEIGHT * 100.0 * section_weight * exam_weight
        } else {
            0.0
        })
    } else {
        None
    };

    let memory_contribution = mastery_entry.and_then(|entry| {
        if entry.studied_cards == 0 || total_studied_cards == 0 {
            return None;
        }
        let card_share = entry.studied_cards as f32 / total_studied_cards as f32;
        Some(readiness::MEMORY_WEIGHT * 100.0 * card_share * entry.avg_retrievability)
    });

    let performance_contribution = if practice_total > 0 && global_practice_total > 0 {
        let attempt_share = practice_total as f32 / global_practice_total as f32;
        let accuracy = practice_correct as f32 / practice_total as f32;
        Some(readiness::PERFORMANCE_WEIGHT * 100.0 * attempt_share * accuracy)
    } else {
        None
    };

    let estimated_total_contribution = {
        let mut sum = 0.0f32;
        let mut has_component = false;
        if let Some(value) = coverage_contribution {
            sum += value;
            has_component = true;
        }
        if let Some(value) = memory_contribution {
            sum += value;
            has_component = true;
        }
        if let Some(value) = performance_contribution {
            sum += value;
            has_component = true;
        }
        if has_component {
            Some(sum)
        } else {
            None
        }
    };

    let summary = build_contribution_summary(
        topic_def,
        covered,
        coverage_contribution,
        memory_contribution,
        performance_contribution,
        estimated_total_contribution,
        global_memory,
        global_performance,
        global_readiness,
    );

    TopicReadinessContribution {
        exam_weight,
        section_weight,
        covered,
        coverage_contribution,
        memory_contribution,
        performance_contribution,
        estimated_total_contribution,
        summary,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_contribution_summary(
    topic_def: &TopicDef,
    covered: bool,
    coverage_contribution: Option<f32>,
    memory_contribution: Option<f32>,
    performance_contribution: Option<f32>,
    estimated_total_contribution: Option<f32>,
    global_memory: &MemoryScore,
    global_performance: &PerformanceScore,
    global_readiness: &ReadinessScore,
) -> String {
    let mut parts = Vec::new();

    if topic_def.is_leaf() {
        parts.push(format!(
            "Exam weight {:.0}% of {}",
            topic_def.exam_weight * 100.0,
            topic_def.section.display_name()
        ));
        if covered {
            if let Some(points) = coverage_contribution {
                parts.push(format!(
                    "Coverage adds up to {:.1} readiness points when this leaf is covered",
                    points
                ));
            }
        } else {
            parts.push("Not covered yet — add flashcards or parent tags for this topic".into());
        }
    } else {
        parts
            .push("Parent topic — coverage and exam weight are rolled up from child topics".into());
    }

    if let Some(points) = memory_contribution {
        parts.push(format!(
            "Memory contributes ~{:.1} points via studied cards and FSRS retrievability",
            points
        ));
    } else if global_memory.sufficient_data {
        parts.push("No studied cards tagged for this topic yet".into());
    }

    if let Some(points) = performance_contribution {
        parts.push(format!(
            "Practice contributes ~{:.1} points based on attempt share and accuracy",
            points
        ));
    } else if global_performance.sufficient_data {
        parts.push("No practice attempts recorded for this topic yet".into());
    }

    if let Some(total) = estimated_total_contribution {
        if global_readiness.sufficient_data {
            parts.push(format!(
                "Estimated ~{:.1} of {:.0} overall readiness points from this topic",
                total,
                global_readiness.projected_score.unwrap_or_default()
            ));
        } else {
            parts.push(format!(
                "Estimated ~{:.1} readiness points from this topic once global readiness unlocks",
                total
            ));
        }
    }

    parts.join(". ")
}

#[cfg(test)]
mod test {
    use anki_proto::brainlift::MemoryScore;
    use anki_proto::brainlift::PerformanceScore;
    use anki_proto::brainlift::ReadinessScore;
    use anki_proto::stats::TopicMasteryEntry;

    use super::*;
    use crate::gre_atlas::GreSection;

    fn leaf_topic() -> &'static TopicDef {
        GreCatalog::topic_by_id("gre::quant::algebra::linear").unwrap()
    }

    fn global_scores() -> (MemoryScore, PerformanceScore, ReadinessScore) {
        let memory = MemoryScore {
            sufficient_data: true,
            value: Some(80.0),
            studied_cards: 200,
            coverage_ratio: 0.5,
            ..Default::default()
        };
        let performance = PerformanceScore {
            sufficient_data: true,
            value: Some(75.0),
            attempt_count: 40,
            ..Default::default()
        };
        let readiness = ReadinessScore {
            sufficient_data: true,
            projected_score: Some(72.0),
            ..Default::default()
        };
        (memory, performance, readiness)
    }

    #[test]
    fn covered_leaf_has_coverage_contribution() {
        let topic = leaf_topic();
        let (memory, performance, readiness) = global_scores();
        let contribution = compute_topic_readiness_contribution(
            topic,
            true,
            None,
            0,
            0,
            0,
            0,
            &memory,
            &performance,
            &readiness,
        );
        let expected = readiness::COVERAGE_WEIGHT
            * 100.0
            * GreSection::QuantitativeReasoning.official_section_weight()
            * topic.exam_weight;
        assert!((contribution.coverage_contribution.unwrap() - expected).abs() < 0.001);
        assert!(contribution.summary.contains("Coverage adds"));
    }

    #[test]
    fn memory_and_performance_contributions_scale_with_evidence() {
        let topic = leaf_topic();
        let (memory, performance, readiness) = global_scores();
        let mastery = TopicMasteryEntry {
            topic_id: topic.id.to_string(),
            display_name: topic.display_name.to_string(),
            total_cards: 50,
            studied_cards: 50,
            mastered_cards: 10,
            avg_retrievability: 0.8,
            avg_retrievability_low: 0.75,
            avg_retrievability_high: 0.85,
            total_reviews: 100,
        };
        let contribution = compute_topic_readiness_contribution(
            topic,
            true,
            Some(&mastery),
            8,
            10,
            200,
            40,
            &memory,
            &performance,
            &readiness,
        );
        let memory_points = contribution.memory_contribution.unwrap();
        assert!((memory_points - readiness::MEMORY_WEIGHT * 100.0 * 0.25 * 0.8).abs() < 0.01);
        let performance_points = contribution.performance_contribution.unwrap();
        assert!(
            (performance_points - readiness::PERFORMANCE_WEIGHT * 100.0 * 0.25 * 0.8).abs() < 0.01
        );
        assert!(contribution.estimated_total_contribution.unwrap() > memory_points);
    }
}
