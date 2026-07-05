// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::Result;
use crate::gre_atlas::domain::GreCatalog;
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
///
/// Template variants follow patterns established in the manually authored
/// foundation bank ([`crate::gre_atlas::questions::foundation`]). When adding
/// LLM generation, condition prompts on `foundation::exemplars_for_topic`.
pub fn generate_question_for_topic(topic_id: &str, now: TimestampSecs) -> GenerationOutcome {
    generate_question_for_topic_variant(topic_id, 0, now)
}

/// Generate a specific template variant for a topic. Variants rotate through
/// parameterized stems derived from the seed question patterns.
pub fn generate_question_for_topic_variant(
    topic_id: &str,
    variant: u32,
    now: TimestampSecs,
) -> GenerationOutcome {
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

    let draft = crate::gre_atlas::questions::variants::build_variant_draft(
        catalog_topic.id,
        catalog_topic.section,
        source,
        variant,
        now,
    );

    if !crate::gre_atlas::questions::variants::correct_answer_in_choices(
        &draft.correct_answer,
        &draft.choices,
    ) {
        return GenerationOutcome::Rejected {
            confidence: 0.0,
            reason: "correct answer is not among the presented choices".into(),
            attribution: draft.attribution.clone(),
        };
    }

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

fn score_confidence(draft: &GeneratedQuestionDraft, source: &SourceSection) -> f32 {
    let topic_match = if draft.topic == source.topic_id {
        1.0
    } else {
        0.0
    };
    let keyword_coverage = keyword_overlap(
        &format!("{} {}", draft.stem, draft.explanation),
        &source_keywords(source),
    );
    let template_validity = if draft.stem.is_empty()
        || draft.choices.is_empty()
        || draft.correct_answer.is_empty()
        || !crate::gre_atlas::questions::variants::correct_answer_in_choices(
            &draft.correct_answer,
            &draft.choices,
        ) {
        0.0
    } else {
        1.0
    };
    let exemplar_bonus =
        if crate::gre_atlas::questions::foundation::exemplars_for_topic(&draft.topic).is_empty() {
            0.0
        } else {
            0.05
        };
    (0.3 * topic_match + 0.4 * keyword_coverage + 0.3 * template_validity + exemplar_bonus).min(1.0)
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
