// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::Result;
use crate::gre_atlas::domain::GreCatalog;
use crate::gre_atlas::domain::GreSection;
use crate::gre_atlas::questions::source::source_section_for_topic;
use crate::gre_atlas::questions::source::SourceSection;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::gre_atlas::questions::source::SOURCE_SECTIONS;
use crate::timestamp::TimestampSecs;

/// Minimum confidence required to accept a generated question.
/// Below this threshold the pipeline rejects the output rather than persisting
/// it.
pub const MIN_GENERATION_CONFIDENCE: f32 = 0.55;

pub const AI_GENERATION_MODEL_VERSION: &str = "template_v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionAttribution {
    pub source_name: String,
    pub source_section: String,
    pub generated_at_secs: i64,
}

#[derive(Debug, Clone)]
pub struct GeneratedQuestionDraft {
    pub id: String,
    pub topic: String,
    pub section: String,
    pub format: String,
    pub stem: String,
    pub choices: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    pub difficulty: Option<f32>,
    pub confidence: f32,
    pub attribution: QuestionAttribution,
}

#[derive(Debug, Clone)]
pub enum GenerationOutcome {
    Accepted(GeneratedQuestionDraft),
    Rejected {
        confidence: f32,
        reason: String,
        attribution: QuestionAttribution,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoldEvalQuestion {
    pub id: String,
    pub topic: String,
    pub section: String,
    pub stem: String,
    pub keywords: Vec<String>,
    pub correct_answer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoldEvalSet {
    pub label: String,
    pub verified: bool,
    pub questions: Vec<GoldEvalQuestion>,
}

#[derive(Debug, Clone)]
pub struct KeywordRetrievalResult {
    pub gold_id: String,
    pub matched_topic: String,
    pub matched_section: String,
    pub keyword_recall: f32,
    pub topic_match: bool,
}

pub fn load_gold_eval_set() -> Result<GoldEvalSet> {
    serde_json::from_str(include_str!("gold_eval_set.json")).map_err(|err| {
        crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!("gold eval set: {err}")),
        }
    })
}

/// Deterministic template-based generation from the named ETS source section.
/// Optional LLM enhancement is env-gated; eval uses this path only for
/// reproducibility.
pub fn generate_question_for_topic(topic_id: &str, now: TimestampSecs) -> GenerationOutcome {
    let Some(catalog_topic) = GreCatalog::topic_by_id(topic_id) else {
        return GenerationOutcome::Rejected {
            confidence: 0.0,
            reason: format!("unknown topic: {topic_id}"),
            attribution: empty_attribution(now),
        };
    };

    let Some(source) = source_section_for_topic(topic_id) else {
        return GenerationOutcome::Rejected {
            confidence: 0.0,
            reason: format!("no source section mapped for topic: {topic_id}"),
            attribution: empty_attribution(now),
        };
    };

    let draft = build_template_draft(catalog_topic.id, catalog_topic.section, source, now);
    let confidence = score_confidence(&draft, source);

    if confidence < MIN_GENERATION_CONFIDENCE {
        return GenerationOutcome::Rejected {
            confidence,
            reason: format!(
                "confidence {confidence:.3} below threshold {MIN_GENERATION_CONFIDENCE}"
            ),
            attribution: draft.attribution.clone(),
        };
    }

    GenerationOutcome::Accepted(GeneratedQuestionDraft {
        confidence,
        ..draft
    })
}

/// Keyword retrieval baseline: pick the source section with highest keyword
/// overlap.
pub fn keyword_retrieve(gold: &GoldEvalQuestion) -> KeywordRetrievalResult {
    let query = normalize_keywords(&gold.keywords);
    let mut best: Option<(&SourceSection, f32)> = None;

    for section in SOURCE_SECTIONS {
        let section_keywords: HashSet<String> = section
            .keywords
            .iter()
            .map(|kw| kw.to_ascii_lowercase())
            .collect();
        if section_keywords.is_empty() {
            continue;
        }
        let overlap = query.intersection(&section_keywords).count() as f32;
        let recall = overlap / query.len().max(1) as f32;
        if best.map_or(true, |(_, best_recall)| recall > best_recall) {
            best = Some((section, recall));
        }
    }

    let (section, keyword_recall) = best.unwrap_or((&SOURCE_SECTIONS[0], 0.0));
    KeywordRetrievalResult {
        gold_id: gold.id.clone(),
        matched_topic: section.topic_id.to_string(),
        matched_section: section.section.to_string(),
        keyword_recall,
        topic_match: section.topic_id == gold.topic,
    }
}

pub fn keyword_overlap(stem: &str, keywords: &[String]) -> f32 {
    let stem_lower = stem.to_ascii_lowercase();
    let hits = keywords
        .iter()
        .filter(|kw| stem_lower.contains(&kw.to_ascii_lowercase()))
        .count();
    hits as f32 / keywords.len().max(1) as f32
}

fn empty_attribution(now: TimestampSecs) -> QuestionAttribution {
    QuestionAttribution {
        source_name: GENERATION_SOURCE_NAME.into(),
        source_section: "unknown".into(),
        generated_at_secs: now.0,
    }
}

fn normalize_keywords(keywords: &[String]) -> HashSet<String> {
    keywords.iter().map(|kw| kw.to_ascii_lowercase()).collect()
}

fn build_template_draft(
    topic_id: &str,
    gre_section: GreSection,
    source: &SourceSection,
    now: TimestampSecs,
) -> GeneratedQuestionDraft {
    let section = gre_section.slug();
    let attribution = QuestionAttribution {
        source_name: GENERATION_SOURCE_NAME.into(),
        source_section: source.section.into(),
        generated_at_secs: now.0,
    };

    match topic_id {
        "gre::quant::arithmetic::percent" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): A laptop priced at $200 \
                 is discounted by 15%. What is the sale price?",
                section_title = source.section
            ),
            vec!["$170", "$185", "$190", "$230"],
            "$170",
            "15% of $200 is $30; $200 − $30 = $170.",
            0.35,
            attribution,
        ),
        "gre::quant::arithmetic::ratio" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): If the ratio of red to blue \
                 marbles is 2:3 and there are 12 red marbles, how many blue marbles are there?",
                section_title = source.section
            ),
            vec!["15", "18", "20", "24"],
            "18",
            "12 red is 2 parts, so one part is 6. Three parts of blue gives 18.",
            0.4,
            attribution,
        ),
        "gre::quant::algebra::linear" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): If 4x − 9 = 11, what is x?",
                section_title = source.section
            ),
            vec!["3", "4", "5", "6"],
            "5",
            "Add 9 to both sides: 4x = 20. Divide by 4: x = 5.",
            0.3,
            attribution,
        ),
        "gre::quant::algebra::quadratic" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): What is the positive root \
                 of x² − 5x + 6 = 0?",
                section_title = source.section
            ),
            vec!["2", "3", "5", "6"],
            "3",
            "Factor: (x − 2)(x − 3) = 0. Positive root is 3.",
            0.45,
            attribution,
        ),
        "gre::quant::geometry::triangles" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): A right triangle has legs 5 \
                 and 12. What is the hypotenuse?",
                section_title = source.section
            ),
            vec!["13", "14", "15", "17"],
            "13",
            "√(5² + 12²) = √169 = 13.",
            0.4,
            attribution,
        ),
        "gre::quant::geometry::circles" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): A circle has radius 4. \
                 What is its area?",
                section_title = source.section
            ),
            vec!["8π", "12π", "16π", "20π"],
            "16π",
            "Area = πr² = π(4)² = 16π.",
            0.4,
            attribution,
        ),
        "gre::quant::data_interpretation" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Revenue rose from $50M to \
                 $65M. What is the percent increase?",
                section_title = source.section
            ),
            vec!["15%", "25%", "30%", "35%"],
            "30%",
            "Increase is $15M on $50M → 15/50 = 30%.",
            0.5,
            attribution,
        ),
        "gre::quant::statistics::probability" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): A bag has 3 red and 7 blue \
                 marbles. If one is chosen at random, what is P(red)?",
                section_title = source.section
            ),
            vec!["3/10", "3/7", "7/10", "1/3"],
            "3/10",
            "3 favorable out of 10 total → 3/10.",
            0.35,
            attribution,
        ),
        "gre::quant::statistics::data_analysis" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): What is the median of \
                 4, 9, 11, 15, 22?",
                section_title = source.section
            ),
            vec!["9", "11", "12", "15"],
            "11",
            "The middle value of the sorted list is 11.",
            0.35,
            attribution,
        ),
        "gre::quant::word_problems" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): A train travels 120 miles \
                 in 2 hours. What is its average speed?",
                section_title = source.section
            ),
            vec!["40 mph", "50 mph", "60 mph", "70 mph"],
            "60 mph",
            "Speed = distance / time = 120 / 2 = 60 mph.",
            0.35,
            attribution,
        ),
        "gre::quant::number_properties" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): What is the remainder when \
                 47 is divided by 6?",
                section_title = source.section
            ),
            vec!["3", "4", "5", "6"],
            "5",
            "47 = 6×7 + 5, so the remainder is 5.",
            0.35,
            attribution,
        ),
        "gre::verbal::text_completion" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): The committee's report was \
                 so ______ that even dissenting members accepted its conclusions.",
                section_title = source.section
            ),
            vec!["equivocal", "persuasive", "opaque", "fragmentary"],
            "persuasive",
            "Dissenters accepting conclusions implies the report was convincing.",
            0.5,
            attribution,
        ),
        "gre::verbal::sentence_equivalence" => GeneratedQuestionDraft {
            id: new_generated_id(topic_id, now),
            topic: topic_id.into(),
            section: section.into(),
            format: "sentence_equivalence".into(),
            stem: format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): The historian's account was \
                 surprisingly ______, given the contentious subject matter.",
                section_title = source.section
            ),
            choices: vec![
                "dispassionate".into(),
                "inflammatory".into(),
                "neutral".into(),
                "biased".into(),
                "polemical".into(),
                "temperate".into(),
            ],
            correct_answer: "dispassionate".into(),
            explanation: "Surprisingly calm tone fits dispassionate (or neutral/temperate).".into(),
            difficulty: Some(0.55),
            confidence: 0.0,
            attribution,
        },
        "gre::verbal::reading::inference" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Passage: Cities that expand \
                 transit see fewer solo car commutes, but housing near stations often becomes more \
                 expensive. Which inference is best supported?",
                section_title = source.section
            ),
            vec![
                "Transit expansion always lowers housing costs.",
                "Convenience may trade off with affordability.",
                "Commutes are unaffected by transit.",
                "Housing prices are unrelated to transit.",
            ],
            "Convenience may trade off with affordability.",
            "The passage links reduced commutes with higher nearby housing costs.",
            0.55,
            attribution,
        ),
        "gre::verbal::reading::main_idea" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Passage: Coral reefs support \
                 diverse fisheries, but warming oceans cause bleaching that collapses those food \
                 webs. What is the main idea?",
                section_title = source.section
            ),
            vec![
                "Fisheries are unrelated to coral health.",
                "Ocean warming threatens reef ecosystems and dependent fisheries.",
                "Bleaching improves biodiversity.",
                "Coral reefs exist only in cold water.",
            ],
            "Ocean warming threatens reef ecosystems and dependent fisheries.",
            "The passage connects warming, bleaching, and fishery collapse.",
            0.55,
            attribution,
        ),
        "gre::verbal::reading::detail" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Passage: The trial lasted \
                 twelve weeks and included 240 participants. According to the passage, how long \
                 did the trial last?",
                section_title = source.section
            ),
            vec!["eight weeks", "ten weeks", "twelve weeks", "twenty weeks"],
            "twelve weeks",
            "The passage explicitly states twelve weeks.",
            0.4,
            attribution,
        ),
        "gre::verbal::vocabulary::context" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Although the instructions \
                 were ______, the team completed the assembly without errors.",
                section_title = source.section
            ),
            vec!["ambiguous", "lucid", "cryptic", "obscure"],
            "lucid",
            "Successful completion implies the instructions were clear — lucid.",
            0.45,
            attribution,
        ),
        "gre::verbal::vocabulary::advanced" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): The CEO's ______ apology \
                 failed to reassure investors who wanted concrete reforms.",
                section_title = source.section
            ),
            vec!["abject", "perfunctory", "sincere", "heartfelt"],
            "perfunctory",
            "Investors wanted substance; a perfunctory apology is superficial.",
            0.55,
            attribution,
        ),
        "gre::verbal::reading::function" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Passage: Many firms adopt \
                 remote work. However, collaboration costs can rise without in-person contact. \
                 What is the function of the second sentence?",
                section_title = source.section
            ),
            vec![
                "It introduces a counterpoint to the first sentence.",
                "It summarizes the entire passage.",
                "It defines remote work.",
                "It provides unrelated historical background.",
            ],
            "It introduces a counterpoint to the first sentence.",
            "However signals a contrast with the benefit stated first.",
            0.55,
            attribution,
        ),
        "gre::awa::issue" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Issue: \"Success is \
                 determined solely by financial wealth.\" What is the strongest critique?",
                section_title = source.section
            ),
            vec![
                "Wealth is the only measurable outcome.",
                "Success can include non-financial contributions and well-being.",
                "Financial wealth is impossible to define.",
                "Critiques of wealth are always invalid.",
            ],
            "Success can include non-financial contributions and well-being.",
            "The claim overgeneralizes; success has multiple dimensions.",
            0.6,
            attribution,
        ),
        "gre::awa::argument" => mcq(
            topic_id,
            section,
            &format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): Argument: \"Our app \
                 downloads increased, so customer satisfaction must have improved.\" What is the \
                 main flaw?",
                section_title = source.section
            ),
            vec![
                "Downloads may not reflect satisfaction.",
                "Satisfaction always equals downloads.",
                "Apps cannot be measured.",
                "Customers never use downloaded apps.",
            ],
            "Downloads may not reflect satisfaction.",
            "The argument equates usage metrics with satisfaction without evidence.",
            0.6,
            attribution,
        ),
        _ => GeneratedQuestionDraft {
            id: new_generated_id(topic_id, now),
            topic: topic_id.into(),
            section: section.into(),
            format: "mcq".into(),
            stem: format!(
                "Based on {GENERATION_SOURCE_NAME} ({section_title}): {excerpt} Which statement \
                 best reflects this section?",
                section_title = source.section,
                excerpt = source.excerpt
            ),
            choices: vec![
                source
                    .keywords
                    .first()
                    .copied()
                    .unwrap_or("concept")
                    .to_string(),
                "unrelated detail".into(),
                "unsupported claim".into(),
                "contradictory idea".into(),
            ],
            correct_answer: source.keywords.first().copied().unwrap_or("concept").into(),
            explanation: "The correct choice aligns with the source excerpt keywords.".into(),
            difficulty: Some(0.5),
            confidence: 0.0,
            attribution,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn mcq(
    topic_id: &str,
    section: &str,
    stem: &str,
    choices: Vec<&str>,
    correct: &str,
    explanation: &str,
    difficulty: f32,
    attribution: QuestionAttribution,
) -> GeneratedQuestionDraft {
    GeneratedQuestionDraft {
        id: new_generated_id(topic_id, TimestampSecs(attribution.generated_at_secs)),
        topic: topic_id.into(),
        section: section.into(),
        format: "mcq".into(),
        stem: stem.into(),
        choices: choices.into_iter().map(str::to_string).collect(),
        correct_answer: correct.into(),
        explanation: explanation.into(),
        difficulty: Some(difficulty),
        confidence: 0.0,
        attribution,
    }
}

fn new_generated_id(topic_id: &str, now: TimestampSecs) -> String {
    let slug = topic_id
        .strip_prefix("gre::")
        .unwrap_or(topic_id)
        .replace("::", "-");
    format!("ai-{slug}-{now}", now = now.0)
}

fn score_confidence(draft: &GeneratedQuestionDraft, source: &SourceSection) -> f32 {
    let topic_match = if draft.topic == source.topic_id {
        1.0
    } else {
        0.0
    };
    let keyword_coverage = keyword_overlap(&draft.stem, &source_keywords(source));
    let template_validity =
        if draft.stem.is_empty() || draft.choices.is_empty() || draft.correct_answer.is_empty() {
            0.0
        } else {
            1.0
        };
    0.3 * topic_match + 0.4 * keyword_coverage + 0.3 * template_validity
}

fn source_keywords(source: &SourceSection) -> Vec<String> {
    source.keywords.iter().map(|kw| (*kw).to_string()).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gold_set_has_fifty_verified_questions() {
        let gold = load_gold_eval_set().unwrap();
        assert!(gold.verified);
        assert_eq!(gold.questions.len(), 50);
    }

    #[test]
    fn generation_records_source_attribution() {
        let now = TimestampSecs(1_700_000_000);
        let outcome = generate_question_for_topic("gre::quant::algebra::linear", now);
        match outcome {
            GenerationOutcome::Accepted(draft) => {
                assert_eq!(draft.attribution.source_name, GENERATION_SOURCE_NAME);
                assert!(!draft.attribution.source_section.is_empty());
                assert_eq!(draft.attribution.generated_at_secs, now.0);
                assert!(draft.confidence >= MIN_GENERATION_CONFIDENCE);
            }
            GenerationOutcome::Rejected { reason, .. } => panic!("unexpected rejection: {reason}"),
        }
    }

    #[test]
    fn rejects_unknown_topic() {
        let outcome = generate_question_for_topic("gre::unknown::topic", TimestampSecs(1));
        assert!(matches!(outcome, GenerationOutcome::Rejected { .. }));
    }

    #[test]
    fn keyword_retrieval_finds_matching_topic() {
        let gold = GoldEvalQuestion {
            id: "test".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            stem: "solve linear equation".into(),
            keywords: vec!["linear".into(), "equation".into(), "solve".into()],
            correct_answer: "5".into(),
        };
        let result = keyword_retrieve(&gold);
        assert!(result.topic_match);
        assert!(result.keyword_recall > 0.0);
    }

    #[test]
    fn confidence_threshold_is_documented_minimum() {
        assert!((MIN_GENERATION_CONFIDENCE - 0.55).abs() < f32::EPSILON);
    }
}
