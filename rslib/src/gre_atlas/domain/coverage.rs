// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use super::catalog::GreCatalog;
use super::catalog::TopicDef;
use super::section::GreSection;

/// Coverage of the canonical GRE topic catalog against observed study tags.
#[derive(Debug, Clone, PartialEq)]
pub struct GreCoverage {
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    /// Fraction of catalog leaf topics with at least one matching observed tag.
    pub unweighted_ratio: f32,
    /// Section-weighted coverage using official section and topic exam weights.
    pub weighted_ratio: f32,
    pub sections: Vec<GreSectionCoverage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreSectionCoverage {
    pub section: GreSection,
    pub section_weight: f32,
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    pub leaf_coverage_ratio: f32,
    /// `section_weight * leaf_coverage_ratio`.
    pub weighted_contribution: f32,
    /// Sum of exam weights for covered leaves in this section.
    pub covered_exam_weight: f32,
}

/// Returns true when an observed tag covers `catalog_topic_id`.
///
/// A catalog topic is covered if any observed tag equals it, is a descendant of
/// it, or is an ancestor of it (studying a parent implies partial coverage of
/// children).
pub fn is_topic_covered(catalog_topic_id: &str, observed_tags: &[&str]) -> bool {
    observed_tags
        .iter()
        .any(|observed| tags_mutually_cover(observed, catalog_topic_id))
}

fn tags_mutually_cover(observed: &str, catalog_topic_id: &str) -> bool {
    if observed == catalog_topic_id {
        return true;
    }
    is_strict_prefix(observed, catalog_topic_id) || is_strict_prefix(catalog_topic_id, observed)
}

fn is_strict_prefix(prefix: &str, value: &str) -> bool {
    value.len() > prefix.len()
        && value.starts_with(prefix)
        && value[prefix.len()..].starts_with("::")
}

/// Compute catalog coverage from observed Anki tags or practice topic ids.
pub fn compute_coverage(observed_tags: &[&str]) -> GreCoverage {
    let mut sections = Vec::with_capacity(GreSection::ALL.len());
    let mut total_leaves = 0u32;
    let mut total_covered = 0u32;
    let mut weighted_sum = 0.0f32;

    for section in GreSection::ALL {
        let leaves = GreCatalog::leaf_topics_for_section(section);
        let catalog_leaf_count = leaves.len() as u32;
        let covered_leaves: Vec<&TopicDef> = leaves
            .iter()
            .copied()
            .filter(|leaf| is_topic_covered(leaf.id, observed_tags))
            .collect();
        let covered_leaf_count = covered_leaves.len() as u32;
        let leaf_coverage_ratio = if catalog_leaf_count > 0 {
            covered_leaf_count as f32 / catalog_leaf_count as f32
        } else {
            0.0
        };
        let covered_exam_weight: f32 = covered_leaves.iter().map(|leaf| leaf.exam_weight).sum();
        let section_weight = section.official_section_weight();
        let weighted_contribution = section_weight * leaf_coverage_ratio;
        weighted_sum += weighted_contribution;

        total_leaves += catalog_leaf_count;
        total_covered += covered_leaf_count;

        sections.push(GreSectionCoverage {
            section,
            section_weight,
            catalog_leaf_count,
            covered_leaf_count,
            leaf_coverage_ratio,
            weighted_contribution,
            covered_exam_weight,
        });
    }

    let unweighted_ratio = if total_leaves > 0 {
        total_covered as f32 / total_leaves as f32
    } else {
        0.0
    };

    GreCoverage {
        catalog_leaf_count: total_leaves,
        covered_leaf_count: total_covered,
        unweighted_ratio,
        weighted_ratio: weighted_sum,
        sections,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_observations_have_zero_coverage() {
        let report = compute_coverage(&[]);
        assert!(report.catalog_leaf_count > 0);
        assert_eq!(report.covered_leaf_count, 0);
        assert_eq!(report.unweighted_ratio, 0.0);
        assert_eq!(report.weighted_ratio, 0.0);
    }

    #[test]
    fn exact_leaf_tag_covers_one_topic() {
        assert!(is_topic_covered(
            "gre::quant::algebra::linear",
            &["gre::quant::algebra::linear"]
        ));
        assert!(!is_topic_covered(
            "gre::quant::algebra::linear",
            &["gre::quant::algebra::quadratic"]
        ));
    }

    #[test]
    fn parent_tag_covers_child_catalog_leaves() {
        assert!(is_topic_covered(
            "gre::quant::algebra::linear",
            &["gre::quant::algebra"]
        ));
        assert!(is_topic_covered(
            "gre::verbal::reading::inference",
            &["gre::verbal::reading"]
        ));
    }

    #[test]
    fn child_tag_covers_ancestor_in_coverage() {
        assert!(is_topic_covered(
            "gre::quant::algebra",
            &["gre::quant::algebra::linear"]
        ));
    }

    #[test]
    fn full_section_coverage() {
        let quant_leaves: Vec<&str> =
            GreCatalog::leaf_topics_for_section(GreSection::QuantitativeReasoning)
                .into_iter()
                .map(|leaf| leaf.id)
                .collect();
        let report = compute_coverage(&quant_leaves);
        let quant = report
            .sections
            .iter()
            .find(|s| s.section == GreSection::QuantitativeReasoning)
            .unwrap();
        assert_eq!(quant.covered_leaf_count, quant.catalog_leaf_count);
        assert!((quant.leaf_coverage_ratio - 1.0).abs() < f32::EPSILON);
        assert!((quant.covered_exam_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn weighted_coverage_uses_leaf_topic_ratios() {
        let report = compute_coverage(&["gre::quant::data_interpretation"]);
        let quant = report
            .sections
            .iter()
            .find(|s| s.section == GreSection::QuantitativeReasoning)
            .unwrap();
        let catalog_leaf_count = quant.catalog_leaf_count as f32;
        let expected_ratio = 1.0 / catalog_leaf_count;
        let expected = GreSection::QuantitativeReasoning.official_section_weight() * expected_ratio;
        assert!((quant.leaf_coverage_ratio - expected_ratio).abs() < 0.0001);
        assert!((quant.weighted_contribution - expected).abs() < 0.0001);
        assert!((report.weighted_ratio - expected).abs() < 0.0001);
    }

    #[test]
    fn weighted_coverage_regression_8_of_21_topics() {
        // Mirrors dashboard fixture: 4/11 quant, 3/8 verbal, 1/2 awa (8/21 overall).
        let observed = [
            "gre::quant::arithmetic::percent",
            "gre::quant::algebra::linear",
            "gre::quant::data_interpretation",
            "gre::quant::geometry::triangles",
            "gre::verbal::text_completion",
            "gre::verbal::reading::inference",
            "gre::verbal::sentence_equivalence",
            "gre::awa::issue",
        ];
        let report = compute_coverage(&observed);
        assert_eq!(report.covered_leaf_count, 8);
        assert_eq!(report.catalog_leaf_count, 21);

        let quant = report
            .sections
            .iter()
            .find(|s| s.section == GreSection::QuantitativeReasoning)
            .unwrap();
        let verbal = report
            .sections
            .iter()
            .find(|s| s.section == GreSection::VerbalReasoning)
            .unwrap();
        let awa = report
            .sections
            .iter()
            .find(|s| s.section == GreSection::AnalyticalWriting)
            .unwrap();

        assert_eq!(quant.covered_leaf_count, 4);
        assert_eq!(quant.catalog_leaf_count, 11);
        assert_eq!(verbal.covered_leaf_count, 3);
        assert_eq!(verbal.catalog_leaf_count, 8);
        assert_eq!(awa.covered_leaf_count, 1);
        assert_eq!(awa.catalog_leaf_count, 2);

        let expected_weighted = GreSection::QuantitativeReasoning.official_section_weight()
            * (4.0 / 11.0)
            + GreSection::VerbalReasoning.official_section_weight() * (3.0 / 8.0)
            + GreSection::AnalyticalWriting.official_section_weight() * (1.0 / 2.0);
        assert!((report.weighted_ratio - expected_weighted).abs() < 0.0001);
        assert!((report.unweighted_ratio - (8.0 / 21.0)).abs() < 0.0001);

        let contribution_sum: f32 = report
            .sections
            .iter()
            .map(|section| section.weighted_contribution)
            .sum();
        assert!((contribution_sum - report.weighted_ratio).abs() < 0.0001);
    }

    #[test]
    fn seed_topics_partial_coverage() {
        let observed = [
            "gre::quant::arithmetic::percent",
            "gre::quant::algebra::linear",
            "gre::verbal::text_completion",
            "gre::verbal::reading::inference",
            "gre::quant::data_interpretation",
        ];
        let report = compute_coverage(&observed);
        assert_eq!(report.covered_leaf_count, 5);
        assert!(report.unweighted_ratio > 0.0 && report.unweighted_ratio < 1.0);
        assert!(report.weighted_ratio > 0.0 && report.weighted_ratio < 1.0);
    }

    #[test]
    fn section_breakdown_sums_weighted_contribution() {
        let observed = ["gre::awa::issue", "gre::verbal::text_completion"];
        let report = compute_coverage(&observed);
        let sum: f32 = report
            .sections
            .iter()
            .map(|s| s.weighted_contribution)
            .sum();
        assert!((sum - report.weighted_ratio).abs() < 0.0001);
    }
}
