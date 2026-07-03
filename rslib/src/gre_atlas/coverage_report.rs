// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashSet;

use anki_proto::brainlift::DashboardCoverage;
use anki_proto::brainlift::DashboardSectionCoverage;

use crate::gre_atlas::abstention::MIN_COVERAGE_RATIO;
use crate::gre_atlas::topic_insights::into_insight;
use crate::gre_atlas::topic_insights::TopicInsightCandidate;
use crate::gre_atlas::GreCatalog;
use crate::gre_atlas::GreCoverage;
use crate::gre_atlas::GreSection;

pub fn section_ui_label(section: GreSection) -> &'static str {
    match section {
        GreSection::QuantitativeReasoning => "Quant",
        GreSection::VerbalReasoning => "Verbal",
        GreSection::AnalyticalWriting => "AWA",
    }
}

pub fn study_recommendation_label(topic_id: &str, display_name: &str) -> String {
    let Some(topic) = GreCatalog::topic_by_id(topic_id) else {
        return format!("Study {display_name}");
    };
    let Some(parent_id) = topic.parent_id else {
        return format!("Study {display_name}");
    };
    let Some(parent) = GreCatalog::topic_by_id(parent_id) else {
        return format!("Study {display_name}");
    };

    if parent_id.contains("::reading") {
        return format!("Study Reading {display_name}");
    }

    let sibling_leaves: Vec<_> = GreCatalog::leaf_topics()
        .filter(|leaf| leaf.parent_id == Some(parent_id))
        .collect();
    if sibling_leaves.len() > 1 && parent.parent_id.is_some() {
        if parent.display_name.contains('&') {
            return format!("Study {display_name}");
        }
        return format!("Study {}", parent.display_name);
    }

    format!("Study {display_name}")
}

pub fn build_dashboard_coverage(
    coverage: &GreCoverage,
    candidates: &[TopicInsightCandidate],
) -> DashboardCoverage {
    let sections = coverage
        .sections
        .iter()
        .map(|section| DashboardSectionCoverage {
            section: section.section.slug().into(),
            label: section_ui_label(section.section).into(),
            covered_exam_weight: section.covered_exam_weight,
            catalog_leaf_count: section.catalog_leaf_count,
            covered_leaf_count: section.covered_leaf_count,
        })
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

    let mut seen = HashSet::new();
    let mut uncovered_topics = Vec::new();
    for candidate in uncovered {
        let study_label = study_recommendation_label(&candidate.topic_id, &candidate.display_name);
        if !seen.insert(study_label.clone()) {
            continue;
        }
        let mut insight = into_insight(candidate);
        insight.study_label = study_label;
        uncovered_topics.push(insight);
    }

    DashboardCoverage {
        weighted_ratio: coverage.weighted_ratio,
        unweighted_ratio: coverage.unweighted_ratio,
        catalog_leaf_count: coverage.catalog_leaf_count,
        covered_leaf_count: coverage.covered_leaf_count,
        sections,
        uncovered_topics,
        coverage_threshold: MIN_COVERAGE_RATIO,
        readiness_available: coverage.weighted_ratio >= MIN_COVERAGE_RATIO,
    }
}

#[cfg(test)]
fn coverage_readiness_reason(coverage: &DashboardCoverage) -> String {
    let pct = (coverage.weighted_ratio * 100.0).round() as u32;
    format!("Only {pct}% of the GRE has evidence.")
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::gre_atlas::compute_coverage;
    use crate::gre_atlas::topic_insights::build_candidate;

    fn candidate(
        topic_id: &str,
        display_name: &str,
        covered: bool,
        exam_weight: f32,
    ) -> TopicInsightCandidate {
        let observed: Vec<&str> = if covered { vec![topic_id] } else { Vec::new() };
        build_candidate(
            topic_id,
            display_name,
            "quant",
            exam_weight,
            &HashMap::new(),
            &observed,
            &HashMap::new(),
        )
    }

    #[test]
    fn study_label_groups_geometry_leaves() {
        assert_eq!(
            study_recommendation_label("gre::quant::geometry::triangles", "Triangles"),
            "Study Geometry"
        );
        assert_eq!(
            study_recommendation_label("gre::quant::statistics::probability", "Probability"),
            "Study Probability"
        );
        assert_eq!(
            study_recommendation_label("gre::verbal::reading::inference", "Inference"),
            "Study Reading Inference"
        );
    }

    #[test]
    fn build_dashboard_coverage_includes_sections_and_uncovered() {
        let coverage = compute_coverage(&[
            "gre::quant::geometry::triangles",
            "gre::verbal::reading::inference",
            "gre::awa::issue",
        ]);
        let candidates = vec![
            candidate("gre::quant::geometry::triangles", "Triangles", true, 0.07),
            candidate(
                "gre::quant::statistics::probability",
                "Probability",
                false,
                0.10,
            ),
            candidate("gre::verbal::reading::inference", "Inference", true, 0.12),
        ];
        let report = build_dashboard_coverage(&coverage, &candidates);
        assert_eq!(report.sections.len(), 3);
        assert!(!report.readiness_available);
        assert_eq!(report.coverage_threshold, MIN_COVERAGE_RATIO);
        assert!(report
            .uncovered_topics
            .iter()
            .any(|topic| topic.study_label == "Study Probability"));
    }

    #[test]
    fn coverage_readiness_reason_uses_weighted_percent() {
        let report = DashboardCoverage {
            weighted_ratio: 0.42,
            ..Default::default()
        };
        assert_eq!(
            coverage_readiness_reason(&report),
            "Only 42% of the GRE has evidence."
        );
    }
}
