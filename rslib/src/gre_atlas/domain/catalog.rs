// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use super::section::GreSection;

pub const TOPIC_TAG_PREFIX: &str = "gre::";

/// One node in the canonical GRE topic hierarchy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopicDef {
    pub id: &'static str,
    pub display_name: &'static str,
    pub section: GreSection,
    pub parent_id: Option<&'static str>,
    /// Official exam weight for leaf topics within their section (section
    /// leaves sum to 1.0).
    pub exam_weight: f32,
}

impl TopicDef {
    pub fn is_leaf(&self) -> bool {
        self.exam_weight > 0.0
    }
}

/// Canonical GRE topic catalog. Parent nodes organize the tree; leaves carry
/// exam weights.
const TOPIC_CATALOG: &[TopicDef] = &[
    // --- Quantitative Reasoning ---
    TopicDef {
        id: "gre::quant",
        display_name: "Quantitative Reasoning",
        section: GreSection::QuantitativeReasoning,
        parent_id: None,
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::quant::arithmetic",
        display_name: "Arithmetic",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::quant::arithmetic::percent",
        display_name: "Percents",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::arithmetic"),
        exam_weight: 0.08,
    },
    TopicDef {
        id: "gre::quant::arithmetic::ratio",
        display_name: "Ratios & proportions",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::arithmetic"),
        exam_weight: 0.07,
    },
    TopicDef {
        id: "gre::quant::algebra",
        display_name: "Algebra",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::quant::algebra::linear",
        display_name: "Linear equations",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::algebra"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::quant::algebra::quadratic",
        display_name: "Quadratic equations",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::algebra"),
        exam_weight: 0.08,
    },
    TopicDef {
        id: "gre::quant::geometry",
        display_name: "Geometry",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::quant::geometry::triangles",
        display_name: "Triangles",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::geometry"),
        exam_weight: 0.07,
    },
    TopicDef {
        id: "gre::quant::geometry::circles",
        display_name: "Circles",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::geometry"),
        exam_weight: 0.07,
    },
    TopicDef {
        id: "gre::quant::data_interpretation",
        display_name: "Data interpretation",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.15,
    },
    TopicDef {
        id: "gre::quant::statistics",
        display_name: "Statistics & probability",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::quant::statistics::probability",
        display_name: "Probability",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::statistics"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::quant::statistics::data_analysis",
        display_name: "Data analysis",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant::statistics"),
        exam_weight: 0.08,
    },
    TopicDef {
        id: "gre::quant::word_problems",
        display_name: "Word problems",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::quant::number_properties",
        display_name: "Number properties",
        section: GreSection::QuantitativeReasoning,
        parent_id: Some("gre::quant"),
        exam_weight: 0.10,
    },
    // --- Verbal Reasoning ---
    TopicDef {
        id: "gre::verbal",
        display_name: "Verbal Reasoning",
        section: GreSection::VerbalReasoning,
        parent_id: None,
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::verbal::reading",
        display_name: "Reading comprehension",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::verbal::reading::inference",
        display_name: "Inference",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::reading"),
        exam_weight: 0.12,
    },
    TopicDef {
        id: "gre::verbal::reading::main_idea",
        display_name: "Main idea",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::reading"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::verbal::reading::detail",
        display_name: "Supporting detail",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::reading"),
        exam_weight: 0.08,
    },
    TopicDef {
        id: "gre::verbal::text_completion",
        display_name: "Text completion",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal"),
        exam_weight: 0.20,
    },
    TopicDef {
        id: "gre::verbal::sentence_equivalence",
        display_name: "Sentence equivalence",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal"),
        exam_weight: 0.15,
    },
    TopicDef {
        id: "gre::verbal::vocabulary",
        display_name: "Vocabulary",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal"),
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::verbal::vocabulary::context",
        display_name: "Context clues",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::vocabulary"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::verbal::vocabulary::advanced",
        display_name: "Advanced vocabulary",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::vocabulary"),
        exam_weight: 0.10,
    },
    TopicDef {
        id: "gre::verbal::reading::function",
        display_name: "Function of a sentence",
        section: GreSection::VerbalReasoning,
        parent_id: Some("gre::verbal::reading"),
        exam_weight: 0.15,
    },
    // --- Analytical Writing ---
    TopicDef {
        id: "gre::awa",
        display_name: "Analytical Writing",
        section: GreSection::AnalyticalWriting,
        parent_id: None,
        exam_weight: 0.0,
    },
    TopicDef {
        id: "gre::awa::issue",
        display_name: "Analyze an Issue",
        section: GreSection::AnalyticalWriting,
        parent_id: Some("gre::awa"),
        exam_weight: 0.50,
    },
    TopicDef {
        id: "gre::awa::argument",
        display_name: "Analyze an Argument",
        section: GreSection::AnalyticalWriting,
        parent_id: Some("gre::awa"),
        exam_weight: 0.50,
    },
];

pub struct GreCatalog;

impl GreCatalog {
    pub fn topics() -> &'static [TopicDef] {
        TOPIC_CATALOG
    }

    pub fn topic_by_id(id: &str) -> Option<&'static TopicDef> {
        TOPIC_CATALOG.iter().find(|topic| topic.id == id)
    }

    pub fn section_for_topic_id(id: &str) -> Option<GreSection> {
        Self::topic_by_id(id).map(|topic| topic.section)
    }

    pub fn children(parent_id: &str) -> Vec<&'static TopicDef> {
        TOPIC_CATALOG
            .iter()
            .filter(|topic| topic.parent_id == Some(parent_id))
            .collect()
    }

    pub fn leaf_topics() -> impl Iterator<Item = &'static TopicDef> {
        TOPIC_CATALOG.iter().filter(|topic| topic.is_leaf())
    }

    pub fn leaf_topics_for_section(section: GreSection) -> Vec<&'static TopicDef> {
        Self::leaf_topics()
            .filter(|topic| topic.section == section)
            .collect()
    }

    pub fn ancestors(id: &str) -> Vec<&'static TopicDef> {
        let mut out = Vec::new();
        let mut current = Self::topic_by_id(id);
        while let Some(topic) = current {
            out.push(topic);
            current = topic.parent_id.and_then(Self::topic_by_id);
        }
        out.reverse();
        out
    }

    pub fn is_known_topic(id: &str) -> bool {
        Self::topic_by_id(id).is_some()
    }

    /// Nearest catalog topic for an observed tag (exact id or longest prefix
    /// match).
    pub fn nearest_topic_for_tag(tag: &str) -> Option<&'static TopicDef> {
        if let Some(exact) = Self::topic_by_id(tag) {
            return Some(exact);
        }
        let mut best: Option<&'static TopicDef> = None;
        for topic in TOPIC_CATALOG {
            if tag.starts_with(topic.id)
                && (tag.len() == topic.id.len() || tag[topic.id.len()..].starts_with("::"))
                && (best.is_none() || topic.id.len() > best.unwrap().id.len())
            {
                best = Some(topic);
            }
        }
        best
    }

    pub fn display_name_for_tag(tag: &str) -> String {
        Self::nearest_topic_for_tag(tag)
            .map(|topic| topic.display_name.to_string())
            .unwrap_or_else(|| {
                tag.strip_prefix(TOPIC_TAG_PREFIX)
                    .unwrap_or(tag)
                    .replace("::", " / ")
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn section_weights_sum_to_one() {
        let sum: f32 = GreSection::ALL
            .iter()
            .map(|section| section.official_section_weight())
            .sum();
        assert!((sum - 1.0).abs() < 0.001, "section weights sum to {sum}");
    }

    #[test]
    fn leaf_weights_sum_to_one_per_section() {
        for section in GreSection::ALL {
            let sum: f32 = GreCatalog::leaf_topics_for_section(section)
                .iter()
                .map(|topic| topic.exam_weight)
                .sum();
            assert!(
                (sum - 1.0).abs() < 0.001,
                "{:?} leaf weights sum to {sum}",
                section
            );
        }
    }

    #[test]
    fn parent_links_are_valid() {
        for topic in GreCatalog::topics() {
            if let Some(parent_id) = topic.parent_id {
                let parent = GreCatalog::topic_by_id(parent_id)
                    .unwrap_or_else(|| panic!("missing parent {parent_id} for {}", topic.id));
                assert_eq!(parent.section, topic.section);
            }
        }
    }

    #[test]
    fn topic_lookup_and_hierarchy() {
        let leaf = GreCatalog::topic_by_id("gre::quant::algebra::linear").unwrap();
        assert!(leaf.is_leaf());
        assert_eq!(leaf.section, GreSection::QuantitativeReasoning);

        let ancestors = GreCatalog::ancestors("gre::quant::algebra::linear");
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0].id, "gre::quant");
        assert_eq!(ancestors[1].id, "gre::quant::algebra");
        assert_eq!(ancestors[2].id, "gre::quant::algebra::linear");

        let children = GreCatalog::children("gre::quant::algebra");
        assert!(children
            .iter()
            .any(|t| t.id == "gre::quant::algebra::linear"));
    }

    #[test]
    fn nearest_topic_for_observed_tags() {
        assert_eq!(
            GreCatalog::nearest_topic_for_tag("gre::quant::arithmetic::percent")
                .unwrap()
                .id,
            "gre::quant::arithmetic::percent"
        );
        assert_eq!(
            GreCatalog::nearest_topic_for_tag("gre::quant::algebra")
                .unwrap()
                .id,
            "gre::quant::algebra"
        );
        assert!(GreCatalog::nearest_topic_for_tag("gre::unknown::topic").is_none());
    }

    #[test]
    fn seed_question_topics_exist_in_catalog() {
        let seed_topics = [
            "gre::quant::arithmetic::percent",
            "gre::quant::algebra::linear",
            "gre::verbal::text_completion",
            "gre::verbal::reading::inference",
            "gre::quant::data_interpretation",
        ];
        for id in seed_topics {
            assert!(
                GreCatalog::topic_by_id(id).is_some(),
                "seed topic missing from catalog: {id}"
            );
        }
    }
}
