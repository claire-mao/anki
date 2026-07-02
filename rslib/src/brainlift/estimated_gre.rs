// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::EstimatedGreScore;
use anki_proto::brainlift::MemoryScore;
use anki_proto::brainlift::PerformanceScore;
use anki_proto::brainlift::ReadinessScore;
use anki_proto::stats::TopicMasteryEntry;

use crate::brainlift::readiness;
use crate::brainlift::signals::practice_stats_for_topic;
use crate::brainlift::GreCoverage;
use crate::brainlift::GreSection;

const GRE_COMBINED_MIN: u32 = 260;
const GRE_COMBINED_MAX: u32 = 340;
const GRE_SECTION_MIN: u32 = 130;
const GRE_SECTION_MAX: u32 = 170;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReadinessIndexEstimate {
    index: f32,
    index_low: f32,
    index_high: f32,
}

pub(crate) fn compute_estimated_gre_score(
    memory: &MemoryScore,
    performance: &PerformanceScore,
    readiness: &ReadinessScore,
    coverage: &GreCoverage,
    mastery_topics: &[TopicMasteryEntry],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> EstimatedGreScore {
    if let Some(estimate) = readiness_index_from_readiness(readiness) {
        return build_score_from_index(
            estimate,
            readiness.confidence_level.as_str(),
            memory,
            performance,
            coverage,
            mastery_topics,
            practice_by_topic,
            false,
        );
    }

    if let Some(estimate) = partial_readiness_index(memory, performance) {
        return build_score_from_index(
            estimate,
            "preliminary",
            memory,
            performance,
            coverage,
            mastery_topics,
            practice_by_topic,
            true,
        );
    }

    EstimatedGreScore {
        sufficient_data: false,
        preliminary: false,
        detail: String::new(),
        abstain_reason: build_abstain_reason(memory, performance, readiness),
        ..Default::default()
    }
}

fn build_score_from_index(
    estimate: ReadinessIndexEstimate,
    confidence_label: &str,
    memory: &MemoryScore,
    performance: &PerformanceScore,
    coverage: &GreCoverage,
    mastery_topics: &[TopicMasteryEntry],
    practice_by_topic: &HashMap<String, (u32, u32)>,
    preliminary: bool,
) -> EstimatedGreScore {
    let combined_score = Some(index_to_combined(estimate.index));
    let combined_score_low = Some(index_to_combined(estimate.index_low));
    let combined_score_high = Some(index_to_combined(estimate.index_high));

    let quant = section_score(
        GreSection::QuantitativeReasoning,
        memory,
        performance,
        coverage,
        mastery_topics,
        practice_by_topic,
    );
    let verbal = section_score(
        GreSection::VerbalReasoning,
        memory,
        performance,
        coverage,
        mastery_topics,
        practice_by_topic,
    );

    let detail = if preliminary {
        format!(
            "Preliminary Quant + Verbal {} mapped from partial memory, performance, and coverage (readiness index {:.0}, {:.0}–{:.0})",
            format_combined_range(combined_score, combined_score_low, combined_score_high),
            estimate.index,
            estimate.index_low,
            estimate.index_high,
        )
    } else {
        format!(
            "Estimated Quant + Verbal {} from readiness index {:.0} ({:.0}–{:.0}, {confidence_label} confidence)",
            format_combined_range(combined_score, combined_score_low, combined_score_high),
            estimate.index,
            estimate.index_low,
            estimate.index_high,
        )
    };

    EstimatedGreScore {
        combined_score,
        combined_score_low,
        combined_score_high,
        quant_score: quant.map(|value| index_to_section(value.index)),
        quant_score_low: quant.map(|value| index_to_section(value.index_low)),
        quant_score_high: quant.map(|value| index_to_section(value.index_high)),
        verbal_score: verbal.map(|value| index_to_section(value.index)),
        verbal_score_low: verbal.map(|value| index_to_section(value.index_low)),
        verbal_score_high: verbal.map(|value| index_to_section(value.index_high)),
        sufficient_data: !preliminary,
        preliminary,
        detail,
        abstain_reason: String::new(),
    }
}

fn readiness_index_from_readiness(readiness: &ReadinessScore) -> Option<ReadinessIndexEstimate> {
    if !readiness.sufficient_data {
        return None;
    }
    Some(ReadinessIndexEstimate {
        index: readiness.projected_score?,
        index_low: readiness.projected_score_low.unwrap_or(readiness.projected_score?),
        index_high: readiness.projected_score_high.unwrap_or(readiness.projected_score?),
    })
}

fn partial_readiness_index(
    memory: &MemoryScore,
    performance: &PerformanceScore,
) -> Option<ReadinessIndexEstimate> {
    let mut weighted_sum = 0.0f32;
    let mut weight_total = 0.0f32;
    let mut low_sum = 0.0f32;
    let mut high_sum = 0.0f32;

    if memory.sufficient_data {
        let value = memory.value?;
        weighted_sum += readiness::MEMORY_WEIGHT * value;
        weight_total += readiness::MEMORY_WEIGHT;
        low_sum += readiness::MEMORY_WEIGHT * memory.value_low.unwrap_or(value);
        high_sum += readiness::MEMORY_WEIGHT * memory.value_high.unwrap_or(value);
    }

    if performance.sufficient_data {
        let value = performance.value?;
        weighted_sum += readiness::PERFORMANCE_WEIGHT * value;
        weight_total += readiness::PERFORMANCE_WEIGHT;
        low_sum += readiness::PERFORMANCE_WEIGHT * performance.value_low.unwrap_or(value);
        high_sum += readiness::PERFORMANCE_WEIGHT * performance.value_high.unwrap_or(value);
    }

    if memory.coverage_ratio > 0.0 {
        let coverage_pct = memory.coverage_ratio * 100.0;
        weighted_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
        weight_total += readiness::COVERAGE_WEIGHT;
        low_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
        high_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
    }

    if weight_total <= 0.0 {
        return None;
    }

    let scale = 100.0 / weight_total;
    Some(ReadinessIndexEstimate {
        index: (weighted_sum * scale).clamp(0.0, 100.0),
        index_low: (low_sum * scale).clamp(0.0, 100.0),
        index_high: (high_sum * scale).clamp(0.0, 100.0),
    })
}

fn section_score(
    section: GreSection,
    memory: &MemoryScore,
    performance: &PerformanceScore,
    coverage: &GreCoverage,
    mastery_topics: &[TopicMasteryEntry],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> Option<ReadinessIndexEstimate> {
    let section_cov_pct = coverage
        .sections
        .iter()
        .find(|entry| entry.section == section)
        .map(|entry| entry.covered_exam_weight.clamp(0.0, 1.0))
        .unwrap_or(0.0);

    let topic_prefix = section.root_topic_id();
    let memory_stats = section_memory_stats(topic_prefix, mastery_topics);
    let practice_stats = practice_stats_for_topic(topic_prefix, practice_by_topic);

    let mut weighted_sum = 0.0f32;
    let mut weight_total = 0.0f32;
    let mut low_sum = 0.0f32;
    let mut high_sum = 0.0f32;

    if let Some((mean, low, high)) = memory_stats {
        let value = mean * 100.0;
        weighted_sum += readiness::MEMORY_WEIGHT * value;
        weight_total += readiness::MEMORY_WEIGHT;
        low_sum += readiness::MEMORY_WEIGHT * low * 100.0;
        high_sum += readiness::MEMORY_WEIGHT * high * 100.0;
    } else if memory.sufficient_data {
        let value = memory.value?;
        weighted_sum += readiness::MEMORY_WEIGHT * value;
        weight_total += readiness::MEMORY_WEIGHT;
        low_sum += readiness::MEMORY_WEIGHT * memory.value_low.unwrap_or(value);
        high_sum += readiness::MEMORY_WEIGHT * memory.value_high.unwrap_or(value);
    }

    if let Some((correct, total)) = practice_stats {
        let accuracy = correct as f32 / total as f32 * 100.0;
        let (low, high) = wilson_ci(correct, total, 1.96);
        weighted_sum += readiness::PERFORMANCE_WEIGHT * accuracy;
        weight_total += readiness::PERFORMANCE_WEIGHT;
        low_sum += readiness::PERFORMANCE_WEIGHT * low * 100.0;
        high_sum += readiness::PERFORMANCE_WEIGHT * high * 100.0;
    } else if performance.sufficient_data {
        let value = performance.value?;
        weighted_sum += readiness::PERFORMANCE_WEIGHT * value;
        weight_total += readiness::PERFORMANCE_WEIGHT;
        low_sum += readiness::PERFORMANCE_WEIGHT * performance.value_low.unwrap_or(value);
        high_sum += readiness::PERFORMANCE_WEIGHT * performance.value_high.unwrap_or(value);
    }

    let coverage_pct = section_cov_pct * 100.0;
    if coverage_pct > 0.0 {
        weighted_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
        weight_total += readiness::COVERAGE_WEIGHT;
        low_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
        high_sum += readiness::COVERAGE_WEIGHT * coverage_pct;
    }

    if weight_total <= 0.0 {
        return None;
    }

    let scale = 100.0 / weight_total;
    Some(ReadinessIndexEstimate {
        index: (weighted_sum * scale).clamp(0.0, 100.0),
        index_low: (low_sum * scale).clamp(0.0, 100.0),
        index_high: (high_sum * scale).clamp(0.0, 100.0),
    })
}

fn section_memory_stats(
    topic_prefix: &str,
    mastery_topics: &[TopicMasteryEntry],
) -> Option<(f32, f32, f32)> {
    let mut total_weight = 0u32;
    let mut weighted_mean = 0f64;
    let mut weighted_low = 0f64;
    let mut weighted_high = 0f64;

    for topic in mastery_topics {
        if topic.studied_cards == 0 || !topic_matches_prefix(&topic.topic_id, topic_prefix) {
            continue;
        }
        let weight = topic.studied_cards as f64;
        weighted_mean += f64::from(topic.avg_retrievability) * weight;
        weighted_low += f64::from(topic.avg_retrievability_low) * weight;
        weighted_high += f64::from(topic.avg_retrievability_high) * weight;
        total_weight += topic.studied_cards;
    }

    if total_weight == 0 {
        return None;
    }

    let denom = f64::from(total_weight);
    Some((
        (weighted_mean / denom) as f32,
        (weighted_low / denom) as f32,
        (weighted_high / denom) as f32,
    ))
}

fn topic_matches_prefix(topic_id: &str, prefix: &str) -> bool {
    topic_id == prefix || topic_id.starts_with(&format!("{prefix}::"))
}

fn index_to_combined(index: f32) -> u32 {
    map_index_to_score(index, GRE_COMBINED_MIN, GRE_COMBINED_MAX)
}

fn index_to_section(index: f32) -> u32 {
    map_index_to_score(index, GRE_SECTION_MIN, GRE_SECTION_MAX)
}

fn map_index_to_score(index: f32, min: u32, max: u32) -> u32 {
    let span = (max - min) as f32;
    let score = min as f32 + (index.clamp(0.0, 100.0) / 100.0) * span;
    score.round().clamp(min as f32, max as f32) as u32
}

fn format_combined_range(
    score: Option<u32>,
    low: Option<u32>,
    high: Option<u32>,
) -> String {
    match (score, low, high) {
        (Some(value), Some(low), Some(high)) if low != high => format!("{value} ({low}–{high})"),
        (Some(value), _, _) => value.to_string(),
        _ => "—".into(),
    }
}

fn build_abstain_reason(
    memory: &MemoryScore,
    performance: &PerformanceScore,
    readiness: &ReadinessScore,
) -> String {
    if !readiness.abstain_reason.is_empty() {
        return format!(
            "Need more GRE study evidence before estimating Quant + Verbal: {}",
            readiness.abstain_reason
        );
    }
    if !memory.abstain_reason.is_empty() && !performance.abstain_reason.is_empty() {
        return "Need studied GRE cards and practice attempts before estimating Quant + Verbal."
            .into();
    }
    if !memory.abstain_reason.is_empty() {
        return format!(
            "Need more GRE flashcard evidence before estimating Quant + Verbal: {}",
            memory.abstain_reason
        );
    }
    format!(
        "Need more GRE practice attempts before estimating Quant + Verbal: {}",
        performance.abstain_reason
    )
}

fn wilson_ci(correct: u32, total: u32, z: f32) -> (f32, f32) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let n = total as f32;
    let p = correct as f32 / n;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let margin = (z / denom) * ((p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt());
    ((center - margin).clamp(0.0, 1.0), (center + margin).clamp(0.0, 1.0))
}

#[cfg(test)]
mod test {
    use super::*;
    use anki_proto::brainlift::ReadinessScore;

    #[test]
    fn maps_readiness_index_to_combined_score() {
        assert_eq!(index_to_combined(0.0), 260);
        assert_eq!(index_to_combined(100.0), 340);
        assert_eq!(index_to_combined(50.0), 300);
        assert_eq!(index_to_combined(72.5), 318);
    }

    #[test]
    fn maps_readiness_index_to_section_score() {
        assert_eq!(index_to_section(0.0), 130);
        assert_eq!(index_to_section(100.0), 170);
        assert_eq!(index_to_section(50.0), 150);
    }

    #[test]
    fn uses_readiness_interval_when_available() {
        let memory = MemoryScore {
            sufficient_data: true,
            value: Some(80.0),
            value_low: Some(75.0),
            value_high: Some(85.0),
            coverage_ratio: 0.6,
            ..Default::default()
        };
        let performance = PerformanceScore {
            sufficient_data: true,
            value: Some(75.0),
            value_low: Some(60.0),
            value_high: Some(85.0),
            attempt_count: 25,
            ..Default::default()
        };
        let readiness = ReadinessScore {
            sufficient_data: true,
            projected_score: Some(72.0),
            projected_score_low: Some(65.0),
            projected_score_high: Some(79.0),
            confidence_level: "medium".into(),
            coverage_ratio: 0.6,
            ..Default::default()
        };
        let coverage = GreCoverage {
            catalog_leaf_count: 10,
            covered_leaf_count: 4,
            unweighted_ratio: 0.4,
            weighted_ratio: 0.6,
            sections: vec![],
        };
        let estimate = compute_estimated_gre_score(
            &memory,
            &performance,
            &readiness,
            &coverage,
            &[],
            &HashMap::new(),
        );
        assert!(estimate.sufficient_data);
        assert!(!estimate.preliminary);
        assert_eq!(estimate.combined_score, Some(318));
        assert_eq!(estimate.combined_score_low, Some(312));
        assert_eq!(estimate.combined_score_high, Some(323));
    }

    #[test]
    fn preliminary_estimate_when_readiness_abstains() {
        let memory = MemoryScore {
            sufficient_data: false,
            coverage_ratio: 0.2,
            studied_cards: 10,
            abstain_reason: "Need more cards".into(),
            ..Default::default()
        };
        let performance = PerformanceScore {
            sufficient_data: true,
            value: Some(80.0),
            value_low: Some(65.0),
            value_high: Some(90.0),
            attempt_count: 25,
            ..Default::default()
        };
        let readiness = ReadinessScore {
            sufficient_data: false,
            abstain_reason: "Need studied cards".into(),
            ..Default::default()
        };
        let coverage = GreCoverage {
            catalog_leaf_count: 10,
            covered_leaf_count: 2,
            unweighted_ratio: 0.2,
            weighted_ratio: 0.2,
            sections: vec![],
        };
        let estimate = compute_estimated_gre_score(
            &memory,
            &performance,
            &readiness,
            &coverage,
            &[],
            &HashMap::new(),
        );
        assert!(estimate.preliminary);
        assert!(!estimate.sufficient_data);
        assert!(estimate.combined_score.is_some());
        assert!(estimate.combined_score_low.is_some());
        assert!(estimate.combined_score_high.is_some());
    }
}
