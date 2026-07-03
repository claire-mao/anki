// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Static excerpts from the single named generation source bundled in-repo.
//!
//! Source: **ETS Official GRE Prep Material** (public ETS quantitative and
//! verbal overview excerpts; not retrieved at runtime).

/// The one named source used for all AI-assisted question generation.
pub const GENERATION_SOURCE_NAME: &str = "ETS Official GRE Prep Material";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSection {
    /// Section title within the named source (e.g. chapter heading).
    pub section: &'static str,
    pub topic_id: &'static str,
    pub gre_section: &'static str,
    pub keywords: &'static [&'static str],
    pub excerpt: &'static str,
}

/// Canonical source sections indexed by GRE catalog leaf topic.
pub const SOURCE_SECTIONS: &[SourceSection] = &[
    SourceSection {
        section: "Quantitative Reasoning — Percents and ratios",
        topic_id: "gre::quant::arithmetic::percent",
        gre_section: "quant",
        keywords: &["percent", "ratio", "fraction", "discount", "increase"],
        excerpt: "Percent problems express a part of a whole out of 100. A percent increase or decrease is applied to an original value.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Percents and ratios",
        topic_id: "gre::quant::arithmetic::ratio",
        gre_section: "quant",
        keywords: &["ratio", "proportion", "part", "scale", "equivalent"],
        excerpt: "Ratios compare two quantities. Equivalent ratios scale both parts by the same factor.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Linear equations",
        topic_id: "gre::quant::algebra::linear",
        gre_section: "quant",
        keywords: &["linear", "equation", "variable", "solve", "coefficient"],
        excerpt: "A linear equation in one variable has the form ax + b = c. Isolate the variable with inverse operations.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Quadratic equations",
        topic_id: "gre::quant::algebra::quadratic",
        gre_section: "quant",
        keywords: &["quadratic", "factor", "root", "parabola", "square"],
        excerpt: "Quadratic equations may be solved by factoring, completing the square, or the quadratic formula.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Triangles",
        topic_id: "gre::quant::geometry::triangles",
        gre_section: "quant",
        keywords: &["triangle", "angle", "hypotenuse", "pythagorean", "area"],
        excerpt: "Triangle angle sums are 180 degrees. Right triangles satisfy a² + b² = c² for the hypotenuse c.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Circles",
        topic_id: "gre::quant::geometry::circles",
        gre_section: "quant",
        keywords: &["circle", "radius", "diameter", "circumference", "area"],
        excerpt: "A circle's circumference is 2πr and area is πr² where r is the radius.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Data interpretation",
        topic_id: "gre::quant::data_interpretation",
        gre_section: "quant",
        keywords: &["table", "chart", "percent", "increase", "compare"],
        excerpt: "Data interpretation items ask you to read tables or charts and compare values or compute percent change.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Probability",
        topic_id: "gre::quant::statistics::probability",
        gre_section: "quant",
        keywords: &["probability", "outcome", "event", "independent", "fraction"],
        excerpt: "Probability of an event is favorable outcomes divided by total equally likely outcomes.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Data analysis",
        topic_id: "gre::quant::statistics::data_analysis",
        gre_section: "quant",
        keywords: &["mean", "median", "range", "standard", "distribution"],
        excerpt: "Basic statistics on the GRE include mean, median, mode, and range of a data set.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Word problems",
        topic_id: "gre::quant::word_problems",
        gre_section: "quant",
        keywords: &["rate", "distance", "time", "work", "combined"],
        excerpt: "Word problems translate a scenario into an equation using rates, distances, or combined work.",
    },
    SourceSection {
        section: "Quantitative Reasoning — Number properties",
        topic_id: "gre::quant::number_properties",
        gre_section: "quant",
        keywords: &["integer", "prime", "factor", "divisible", "remainder"],
        excerpt: "Number properties include divisibility, prime factorization, and remainders.",
    },
    SourceSection {
        section: "Verbal Reasoning — Text completion",
        topic_id: "gre::verbal::text_completion",
        gre_section: "verbal",
        keywords: &["context", "blank", "contrast", "support", "logic"],
        excerpt: "Text completion items require choosing words that fit the sentence's logical structure.",
    },
    SourceSection {
        section: "Verbal Reasoning — Sentence equivalence",
        topic_id: "gre::verbal::sentence_equivalence",
        gre_section: "verbal",
        keywords: &["equivalent", "synonym", "tone", "meaning", "pair"],
        excerpt: "Sentence equivalence selects two answer choices that complete the sentence with equivalent meaning.",
    },
    SourceSection {
        section: "Verbal Reasoning — Reading comprehension: inference",
        topic_id: "gre::verbal::reading::inference",
        gre_section: "verbal",
        keywords: &["inference", "passage", "imply", "support", "conclude"],
        excerpt: "Inference questions ask what the passage most strongly supports without stating explicitly.",
    },
    SourceSection {
        section: "Verbal Reasoning — Reading comprehension: main idea",
        topic_id: "gre::verbal::reading::main_idea",
        gre_section: "verbal",
        keywords: &["main", "idea", "purpose", "passage", "central"],
        excerpt: "Main idea questions identify the central claim or purpose of a short passage.",
    },
    SourceSection {
        section: "Verbal Reasoning — Reading comprehension: detail",
        topic_id: "gre::verbal::reading::detail",
        gre_section: "verbal",
        keywords: &["detail", "passage", "states", "according", "specific"],
        excerpt: "Detail questions locate information explicitly stated in the passage.",
    },
    SourceSection {
        section: "Verbal Reasoning — Vocabulary: context clues",
        topic_id: "gre::verbal::vocabulary::context",
        gre_section: "verbal",
        keywords: &["context", "meaning", "clue", "surrounding", "word"],
        excerpt: "Unfamiliar words can often be inferred from surrounding context clues in the sentence.",
    },
    SourceSection {
        section: "Verbal Reasoning — Vocabulary: advanced",
        topic_id: "gre::verbal::vocabulary::advanced",
        gre_section: "verbal",
        keywords: &["vocabulary", "precise", "nuance", "register", "connotation"],
        excerpt: "Advanced vocabulary items test precise word choice and connotation in academic prose.",
    },
    SourceSection {
        section: "Verbal Reasoning — Reading comprehension: function",
        topic_id: "gre::verbal::reading::function",
        gre_section: "verbal",
        keywords: &["function", "sentence", "role", "passage", "structure"],
        excerpt: "Function questions ask why the author included a particular sentence in the passage structure.",
    },
    SourceSection {
        section: "Analytical Writing — Analyze an Issue",
        topic_id: "gre::awa::issue",
        gre_section: "awa",
        keywords: &["issue", "claim", "position", "evidence", "critique"],
        excerpt: "Issue tasks present a claim; a strong response evaluates the claim and considers counterarguments.",
    },
    SourceSection {
        section: "Analytical Writing — Analyze an Argument",
        topic_id: "gre::awa::argument",
        gre_section: "awa",
        keywords: &["argument", "assumption", "evidence", "flaw", "alternative"],
        excerpt: "Argument tasks critique unstated assumptions and weak evidence in a brief argument passage.",
    },
];

pub fn source_section_for_topic(topic_id: &str) -> Option<&'static SourceSection> {
    SOURCE_SECTIONS
        .iter()
        .find(|section| section.topic_id == topic_id)
}

pub fn all_source_sections() -> &'static [SourceSection] {
    SOURCE_SECTIONS
}
