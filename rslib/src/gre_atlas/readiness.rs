// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::MemoryScore;
use anki_proto::brainlift::PerformanceScore;
use anki_proto::brainlift::ReadinessScore;
use anki_proto::stats::TopicMasteryEntry;

use crate::gre_atlas::abstention::abstain_reason_from_requirements;
use crate::gre_atlas::abstention::memory_requirements;
use crate::gre_atlas::abstention::performance_requirements;
use crate::gre_atlas::abstention::readiness_requirements;
use crate::gre_atlas::abstention::sufficient_from_requirements;

pub(crate) const MEMORY_WEIGHT: f32 = 0.45;
pub(crate) const PERFORMANCE_WEIGHT: f32 = 0.45;
pub(crate) const COVERAGE_WEIGHT: f32 = 0.10;

const CONFIDENCE_HIGH_MAX_WIDTH: f32 = 8.0;
const CONFIDENCE_MEDIUM_MAX_WIDTH: f32 = 15.0;

pub(crate) struct MemoryInputs<'a> {
    pub fsrs_enabled: bool,
    pub overall_retrievability: f32,
    pub coverage_ratio: f32,
    pub studied_cards: u32,
    pub topics: &'a [TopicMasteryEntry],
}

pub(crate) struct PerformanceInputs {
    pub correct: u32,
    pub total: u32,
}

pub(crate) fn compute_memory_score(input: &MemoryInputs) -> MemoryScore {
    let requirements = memory_requirements(
        input.fsrs_enabled,
        input.studied_cards,
        input.coverage_ratio,
    );
    let sufficient_data = sufficient_from_requirements(&requirements);
    let abstain_reason = abstain_reason_from_requirements(&requirements);
    let (value_low, value_high) =
        memory_retrievability_ci(input.topics, input.overall_retrievability);
    let value = input.overall_retrievability * 100.0;
    if sufficient_data {
        let covered_leaves = input
            .topics
            .iter()
            .filter(|topic| {
                topic.studied_cards > 0
                    && crate::gre_atlas::GreCatalog::topic_by_id(&topic.topic_id)
                        .is_some_and(|def| def.is_leaf())
            })
            .count();
        MemoryScore {
            value: Some(value),
            value_low: Some(value_low * 100.0),
            value_high: Some(value_high * 100.0),
            sufficient_data: true,
            detail: format!(
                "{} studied cards · {}% catalog coverage ({} leaf topics with data)",
                input.studied_cards,
                (input.coverage_ratio * 100.0).round() as u32,
                covered_leaves,
            ),
            coverage_ratio: input.coverage_ratio,
            abstain_reason: String::new(),
            studied_cards: input.studied_cards,
            abstention_requirements: requirements,
        }
    } else {
        MemoryScore {
            value: None,
            value_low: None,
            value_high: None,
            sufficient_data: false,
            detail: abstain_reason.clone(),
            coverage_ratio: input.coverage_ratio,
            abstain_reason,
            studied_cards: input.studied_cards,
            abstention_requirements: requirements,
        }
    }
}

pub(crate) fn compute_performance_score(input: &PerformanceInputs) -> PerformanceScore {
    let requirements = performance_requirements(input.total);
    let sufficient_data = sufficient_from_requirements(&requirements);
    let abstain_reason = abstain_reason_from_requirements(&requirements);
    if !sufficient_data {
        return PerformanceScore {
            value: None,
            value_low: None,
            value_high: None,
            sufficient_data: false,
            detail: abstain_reason.clone(),
            attempt_count: input.total,
            abstain_reason,
            abstention_requirements: requirements,
        };
    }
    let (low, high) = wilson_ci(input.correct, input.total, 1.96);
    let accuracy = input.correct as f32 / input.total as f32;
    PerformanceScore {
        value: Some(accuracy * 100.0),
        value_low: Some(low * 100.0),
        value_high: Some(high * 100.0),
        sufficient_data: true,
        detail: format!(
            "{}/{} practice questions correct ({:.0}%)",
            input.correct,
            input.total,
            accuracy * 100.0
        ),
        attempt_count: input.total,
        abstain_reason: String::new(),
        abstention_requirements: requirements,
    }
}

pub(crate) fn compute_readiness_score(
    memory: &MemoryScore,
    performance: &PerformanceScore,
    last_updated_millis: i64,
) -> ReadinessScore {
    let coverage_ratio = memory.coverage_ratio;
    let requirements = readiness_requirements(
        &memory.abstention_requirements,
        &performance.abstention_requirements,
    );
    let sufficient_data = sufficient_from_requirements(&requirements);
    let evidence_summary = build_evidence_summary(memory, performance);

    if !sufficient_data {
        return ReadinessScore {
            projected_score: None,
            projected_score_low: None,
            projected_score_high: None,
            confidence_level: String::new(),
            coverage_ratio,
            last_updated_millis,
            evidence_summary,
            sufficient_data: false,
            abstain_reason: abstain_reason_from_requirements(&requirements),
            calibration_note: String::new(),
            calibration_brier_score: None,
            calibration_sufficient_data: false,
            calibration_well_calibrated: false,
            abstention_requirements: requirements,
        };
    }

    let memory_norm = memory.value.unwrap_or_default() / 100.0;
    let perf_norm = performance.value.unwrap_or_default() / 100.0;
    let projected = (MEMORY_WEIGHT * memory_norm
        + PERFORMANCE_WEIGHT * perf_norm
        + COVERAGE_WEIGHT * coverage_ratio)
        * 100.0;

    let memory_width =
        memory.value_high.unwrap_or(projected) - memory.value_low.unwrap_or(projected);
    let perf_width =
        performance.value_high.unwrap_or(projected) - performance.value_low.unwrap_or(projected);
    let combined_margin = (memory_width * memory_width + perf_width * perf_width).sqrt() / 2.0;
    let projected_low = (projected - combined_margin).clamp(0.0, 100.0);
    let projected_high = (projected + combined_margin).clamp(0.0, 100.0);

    let confidence_level = confidence_level(
        memory.studied_cards,
        performance.attempt_count,
        coverage_ratio,
        projected_high - projected_low,
    );

    ReadinessScore {
        projected_score: Some(projected),
        projected_score_low: Some(projected_low),
        projected_score_high: Some(projected_high),
        confidence_level,
        coverage_ratio,
        last_updated_millis,
        evidence_summary,
        sufficient_data: true,
        abstain_reason: String::new(),
        calibration_note: String::new(),
        calibration_brier_score: None,
        calibration_sufficient_data: false,
        calibration_well_calibrated: false,
        abstention_requirements: requirements,
    }
}

fn build_evidence_summary(memory: &MemoryScore, performance: &PerformanceScore) -> String {
    let memory_part = if memory.sufficient_data {
        format!(
            "Memory {:.0}% ({} studied cards, {:.0}% catalog coverage)",
            memory.value.unwrap_or_default(),
            memory.studied_cards,
            memory.coverage_ratio * 100.0
        )
    } else if memory.studied_cards > 0 {
        format!(
            "Memory insufficient ({} studied cards, {:.0}% catalog coverage)",
            memory.studied_cards,
            memory.coverage_ratio * 100.0
        )
    } else {
        "Memory insufficient (no studied GRE cards)".into()
    };

    let performance_part = if performance.sufficient_data {
        format!(
            "Performance {:.0}% ({} attempts)",
            performance.value.unwrap_or_default(),
            performance.attempt_count
        )
    } else {
        format!(
            "Performance insufficient ({} attempts)",
            performance.attempt_count
        )
    };

    format!("{memory_part} · {performance_part}")
}

fn confidence_level(
    studied_cards: u32,
    attempt_count: u32,
    coverage_ratio: f32,
    interval_width: f32,
) -> String {
    if studied_cards >= 400
        && attempt_count >= 50
        && coverage_ratio >= 0.7
        && interval_width <= CONFIDENCE_HIGH_MAX_WIDTH
    {
        "high".into()
    } else if interval_width <= CONFIDENCE_MEDIUM_MAX_WIDTH {
        "medium".into()
    } else {
        "low".into()
    }
}

fn memory_retrievability_ci(topics: &[TopicMasteryEntry], overall_avg: f32) -> (f32, f32) {
    let mut total_weight = 0u32;
    let mut weighted_low = 0f64;
    let mut weighted_high = 0f64;
    for topic in topics {
        if topic.studied_cards == 0 {
            continue;
        }
        let weight = topic.studied_cards as f64;
        weighted_low += f64::from(topic.avg_retrievability_low) * weight;
        weighted_high += f64::from(topic.avg_retrievability_high) * weight;
        total_weight += topic.studied_cards;
    }
    if total_weight == 0 {
        return (overall_avg, overall_avg);
    }
    (
        (weighted_low / f64::from(total_weight)) as f32,
        (weighted_high / f64::from(total_weight)) as f32,
    )
}

pub(crate) fn wilson_ci(correct: u32, total: u32, z: f32) -> (f32, f32) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let n = total as f32;
    let p = correct as f32 / n;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let margin = (z / denom) * ((p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt());
    (
        (center - margin).clamp(0.0, 1.0),
        (center + margin).clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod test {
    use anki_proto::stats::TopicMasteryEntry;

    use super::*;
    use crate::gre_atlas::abstention::REQ_PRACTICE_ATTEMPTS;
    use crate::gre_atlas::abstention::REQ_STUDIED_CARDS;
    use crate::gre_atlas::abstention::REQ_TOPIC_COVERAGE;

    fn topic(id: &str, studied: u32, avg: f32, low: f32, high: f32) -> TopicMasteryEntry {
        TopicMasteryEntry {
            topic_id: id.to_string(),
            display_name: id.to_string(),
            total_cards: studied,
            studied_cards: studied,
            mastered_cards: 0,
            avg_retrievability: avg,
            avg_retrievability_low: low,
            avg_retrievability_high: high,
            total_reviews: studied,
            ..Default::default()
        }
    }

    fn sufficient_memory() -> MemoryScore {
        compute_memory_score(&MemoryInputs {
            fsrs_enabled: true,
            overall_retrievability: 0.8,
            coverage_ratio: 0.6,
            studied_cards: 250,
            topics: &[topic("gre::quant::algebra", 250, 0.8, 0.75, 0.85)],
        })
    }

    fn sufficient_performance() -> PerformanceScore {
        compute_performance_score(&PerformanceInputs {
            correct: 40,
            total: 50,
        })
    }

    #[test]
    fn readiness_weights_sum_to_one() {
        assert!((MEMORY_WEIGHT + PERFORMANCE_WEIGHT + COVERAGE_WEIGHT - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn performance_abstains_below_minimum_attempts() {
        let score = compute_performance_score(&PerformanceInputs {
            correct: 1,
            total: 5,
        });
        assert!(!score.sufficient_data);
        assert!(score.value.is_none());
        assert_eq!(score.abstention_requirements.len(), 1);
        let req = &score.abstention_requirements[0];
        assert_eq!(req.id, REQ_PRACTICE_ATTEMPTS);
        assert!(!req.met);
        assert!(req.next_step.contains("50"));
    }

    #[test]
    fn memory_abstains_without_fsrs() {
        let score = compute_memory_score(&MemoryInputs {
            fsrs_enabled: false,
            overall_retrievability: 0.8,
            coverage_ratio: 0.6,
            studied_cards: 250,
            topics: &[],
        });
        assert!(!score.sufficient_data);
        assert!(score
            .abstention_requirements
            .iter()
            .any(|req| req.id == "fsrs_enabled" && !req.met));
    }

    #[test]
    fn readiness_abstains_without_sufficient_inputs() {
        let memory = sufficient_memory();
        let performance = compute_performance_score(&PerformanceInputs {
            correct: 1,
            total: 3,
        });
        let readiness = compute_readiness_score(&memory, &performance, 1_700_000_000_000);
        assert!(!readiness.sufficient_data);
        assert!(readiness.projected_score.is_none());
        assert!(!readiness.abstain_reason.is_empty());
        assert!(readiness
            .evidence_summary
            .contains("Performance insufficient"));
        let unmet: Vec<_> = readiness
            .abstention_requirements
            .iter()
            .filter(|req| !req.met)
            .map(|req| req.id.as_str())
            .collect();
        assert_eq!(unmet, vec![REQ_PRACTICE_ATTEMPTS]);
        assert!(!readiness.abstention_requirements.is_empty());
    }

    #[test]
    fn readiness_abstains_when_memory_and_performance_insufficient() {
        let memory = compute_memory_score(&MemoryInputs {
            fsrs_enabled: true,
            overall_retrievability: 0.5,
            coverage_ratio: 0.2,
            studied_cards: 10,
            topics: &[],
        });
        let performance = compute_performance_score(&PerformanceInputs {
            correct: 0,
            total: 0,
        });
        let readiness = compute_readiness_score(&memory, &performance, 1);
        assert!(!readiness.sufficient_data);
        let unmet: Vec<_> = readiness
            .abstention_requirements
            .iter()
            .filter(|req| !req.met)
            .map(|req| req.id.as_str())
            .collect();
        assert_eq!(
            unmet,
            vec![REQ_STUDIED_CARDS, REQ_TOPIC_COVERAGE, REQ_PRACTICE_ATTEMPTS]
        );
        for req in readiness
            .abstention_requirements
            .iter()
            .filter(|req| !req.met)
        {
            assert!(!req.next_step.is_empty());
        }
    }

    #[test]
    fn readiness_projects_score_when_evidence_is_sufficient() {
        let memory = sufficient_memory();
        let performance = sufficient_performance();
        let readiness = compute_readiness_score(&memory, &performance, 1_700_000_000_000);
        assert!(readiness.sufficient_data);
        let projected = readiness.projected_score.unwrap();
        assert!(projected > 0.0 && projected <= 100.0);
        assert!(readiness.projected_score_low.unwrap() <= projected);
        assert!(readiness.projected_score_high.unwrap() >= projected);
        assert_eq!(readiness.coverage_ratio, 0.6);
        assert!(!readiness.confidence_level.is_empty());
        assert!(readiness.evidence_summary.contains("Memory"));
        assert!(readiness.evidence_summary.contains("Performance"));
        assert!(readiness.abstain_reason.is_empty());
        assert!(readiness.abstention_requirements.iter().all(|req| req.met));
    }

    #[test]
    fn wilson_interval_is_bounded() {
        let (low, high) = wilson_ci(8, 10, 1.96);
        assert!(low <= 0.8);
        assert!(high >= 0.8);
        assert!(low >= 0.0 && high <= 1.0);
    }
}
