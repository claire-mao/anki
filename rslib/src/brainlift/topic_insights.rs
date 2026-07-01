// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::collections::HashSet;

use anki_proto::brainlift::DashboardTopicInsight;
use anki_proto::stats::TopicMasteryEntry;

pub(crate) const LOW_MEMORY_RETRIEVABILITY: f32 = 0.7;
pub(crate) const LOW_PRACTICE_ACCURACY: f32 = 0.5;
pub(crate) const MIN_PRACTICE_ATTEMPTS_FOR_WEAK: u32 = 3;
pub(crate) const HIGH_EXAM_WEIGHT: f32 = 0.10;

#[derive(Clone)]
pub(crate) struct TopicInsightCandidate {
    pub topic_id: String,
    pub display_name: String,
    pub section: String,
    pub exam_weight: f32,
    pub memory_score: Option<f32>,
    pub practice_accuracy: Option<f32>,
    pub studied_cards: u32,
    pub covered: bool,
    pub reason: String,
    pub rank_score: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct LeafMetrics {
    pub studied_cards: u32,
    pub covered: bool,
    pub memory_score: Option<f32>,
    pub practice_accuracy: Option<f32>,
    pub memory_retrievability: Option<f32>,
    pub practice_correct: Option<u32>,
    pub practice_total: Option<u32>,
}

pub(crate) fn leaf_metrics(
    topic_id: &str,
    mastery_by_id: &HashMap<String, TopicMasteryEntry>,
    observed_tags: &[&str],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> LeafMetrics {
    let mastery = mastery_by_id.get(topic_id);
    let studied_cards = mastery.map(|entry| entry.studied_cards).unwrap_or(0);
    let covered = super::signals::is_leaf_covered(topic_id, observed_tags, mastery_by_id);
    let memory_retrievability = mastery.and_then(|entry| {
        if entry.studied_cards > 0 {
            Some(entry.avg_retrievability)
        } else {
            None
        }
    });
    let memory_score = memory_retrievability.map(|value| value * 100.0);
    let practice_stats = super::signals::practice_stats_for_topic(topic_id, practice_by_topic);
    let practice_accuracy = practice_stats
        .map(|(correct, total)| correct as f32 / total as f32 * 100.0);

    LeafMetrics {
        studied_cards,
        covered,
        memory_score,
        practice_accuracy,
        memory_retrievability,
        practice_correct: practice_stats.map(|(correct, _)| correct),
        practice_total: practice_stats.map(|(_, total)| total),
    }
}

pub(crate) fn build_candidate(
    topic_id: &str,
    display_name: &str,
    section: &str,
    exam_weight: f32,
    mastery_by_id: &HashMap<String, TopicMasteryEntry>,
    observed_tags: &[&str],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> TopicInsightCandidate {
    let metrics = leaf_metrics(topic_id, mastery_by_id, observed_tags, practice_by_topic);
    let mastery = mastery_by_id.get(topic_id);

    let mut rank_score = 0.0f32;
    let mut reason = String::new();

    if !metrics.covered {
        rank_score = exam_weight * 1.5;
        reason = "Not yet covered in your GRE deck".into();
    } else if let Some(entry) = mastery {
        if entry.studied_cards > 0 && entry.avg_retrievability < LOW_MEMORY_RETRIEVABILITY {
            rank_score = exam_weight * (1.0 - entry.avg_retrievability);
            reason = format!(
                "Low memory strength ({:.0}% FSRS retrievability)",
                entry.avg_retrievability * 100.0
            );
        }
    }

    if let Some((correct, total)) = super::signals::practice_stats_for_topic(topic_id, practice_by_topic)
    {
        if total >= MIN_PRACTICE_ATTEMPTS_FOR_WEAK {
            let accuracy = correct as f32 / total as f32;
            if accuracy < LOW_PRACTICE_ACCURACY {
                let practice_rank = exam_weight * (1.0 - accuracy);
                if practice_rank > rank_score {
                    rank_score = practice_rank;
                    reason = format!("Low practice accuracy ({correct}/{total} correct)");
                }
            }
        }
    }

    TopicInsightCandidate {
        topic_id: topic_id.to_string(),
        display_name: display_name.to_string(),
        section: section.to_string(),
        exam_weight,
        memory_score: metrics.memory_score,
        practice_accuracy: metrics.practice_accuracy,
        studied_cards: metrics.studied_cards,
        covered: metrics.covered,
        reason,
        rank_score,
    }
}

pub(crate) fn into_insight(candidate: TopicInsightCandidate) -> DashboardTopicInsight {
    DashboardTopicInsight {
        topic_id: candidate.topic_id,
        display_name: candidate.display_name,
        section: candidate.section,
        exam_weight: candidate.exam_weight,
        memory_score: candidate.memory_score,
        practice_accuracy: candidate.practice_accuracy,
        studied_cards: candidate.studied_cards,
        covered: candidate.covered,
        reason: candidate.reason,
    }
}

pub(crate) fn select_weak_topics(
    candidates: &[TopicInsightCandidate],
    limit: u32,
) -> Vec<DashboardTopicInsight> {
    let mut ranked: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.rank_score > 0.0)
        .cloned()
        .collect();
    ranked.sort_by(|a, b| {
        b.rank_score
            .partial_cmp(&a.rank_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked.truncate(limit as usize);
    ranked.into_iter().map(into_insight).collect()
}

pub(crate) fn select_recommended_topics(
    candidates: &[TopicInsightCandidate],
    weak_topics: &[DashboardTopicInsight],
    limit: u32,
) -> Vec<DashboardTopicInsight> {
    let weak_ids: HashSet<_> = weak_topics
        .iter()
        .map(|topic| topic.topic_id.as_str())
        .collect();

    let mut uncovered: Vec<_> = candidates
        .iter()
        .filter(|candidate| !candidate.covered)
        .cloned()
        .collect();
    uncovered.sort_by(|a, b| {
        b.exam_weight
            .partial_cmp(&a.exam_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut recommended = Vec::new();
    for mut candidate in uncovered {
        if recommended.len() >= limit as usize {
            break;
        }
        candidate.reason = "High-yield topic not yet in your deck".into();
        recommended.push(into_insight(candidate));
    }

    if recommended.len() < limit as usize {
        let mut studied_gaps: Vec<_> = candidates
            .iter()
            .filter(|candidate| {
                candidate.covered
                    && !weak_ids.contains(candidate.topic_id.as_str())
                    && candidate
                        .memory_score
                        .is_some_and(|score| score < LOW_MEMORY_RETRIEVABILITY * 100.0)
            })
            .cloned()
            .collect();
        studied_gaps.sort_by(|a, b| {
            b.exam_weight
                .partial_cmp(&a.exam_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for mut candidate in studied_gaps {
            if recommended.len() >= limit as usize {
                break;
            }
            candidate.reason = "Review this studied topic to raise memory strength".into();
            recommended.push(into_insight(candidate));
        }
    }

    recommended
}
