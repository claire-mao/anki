// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/// Official GRE General Test sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GreSection {
    QuantitativeReasoning,
    VerbalReasoning,
    AnalyticalWriting,
}

impl GreSection {
    pub const ALL: [GreSection; 3] = [
        GreSection::QuantitativeReasoning,
        GreSection::VerbalReasoning,
        GreSection::AnalyticalWriting,
    ];

    /// Short slug used in storage and legacy seed data (`quant`, `verbal`, `awa`).
    pub fn slug(self) -> &'static str {
        match self {
            GreSection::QuantitativeReasoning => "quant",
            GreSection::VerbalReasoning => "verbal",
            GreSection::AnalyticalWriting => "awa",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            GreSection::QuantitativeReasoning => "Quantitative Reasoning",
            GreSection::VerbalReasoning => "Verbal Reasoning",
            GreSection::AnalyticalWriting => "Analytical Writing",
        }
    }

    /// Share of overall scored GRE prep weight (quant + verbal + AWA sum to 1.0).
    pub fn official_section_weight(self) -> f32 {
        match self {
            GreSection::QuantitativeReasoning => 0.47,
            GreSection::VerbalReasoning => 0.47,
            GreSection::AnalyticalWriting => 0.06,
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "quant" => Some(GreSection::QuantitativeReasoning),
            "verbal" => Some(GreSection::VerbalReasoning),
            "awa" => Some(GreSection::AnalyticalWriting),
            _ => None,
        }
    }

    /// Root topic id for this section in the canonical catalog.
    pub fn root_topic_id(self) -> &'static str {
        match self {
            GreSection::QuantitativeReasoning => "gre::quant",
            GreSection::VerbalReasoning => "gre::verbal",
            GreSection::AnalyticalWriting => "gre::awa",
        }
    }
}
