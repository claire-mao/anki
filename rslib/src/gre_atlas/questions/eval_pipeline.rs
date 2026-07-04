// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Pre-exposure evaluation gate for generated questions.
//!
//! Every candidate question — whether produced by the real LLM path or the
//! deterministic template path — passes through [`evaluate_draft`] before it is
//! allowed to reach a learner. The gate enforces four rules:
//!
//! 1. **Hallucination**: the answer must be derivable and present among the
//!    choices, and the item must be structurally well-formed.
//! 2. **Unsupported**: the stem/explanation must be grounded in the bundled
//!    source material (and, when available, the held-out gold set) rather than
//!    inventing claims.
//! 3. **Duplicate**: the candidate must not be a near-duplicate of an existing
//!    bank question.
//! 4. Anything that passes all three is **approved**.
//!
//! Rejected candidates are never persisted into `bl_question`; instead the
//! generator falls back to a deterministic template and logs the rejection via
//! [`GenerationEvalRecord`].

use std::collections::HashSet;

use crate::gre_atlas::questions::ai_gen::keyword_overlap;
use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
use crate::gre_atlas::questions::ai_gen::GoldEvalSet;
use crate::gre_atlas::questions::metadata::EvaluationStatus;
use crate::gre_atlas::questions::metadata::Provenance;
use crate::gre_atlas::questions::source::SourceSection;
use crate::gre_atlas::questions::variants::correct_answer_in_choices;

/// Minimum grounding score (keyword overlap with the source excerpt / gold set)
/// required to accept a candidate. Below this the item is treated as making
/// unsupported claims.
pub const GROUNDING_MIN: f32 = 0.15;

/// Maximum Jaccard token similarity a candidate may have with any existing bank
/// question before it is treated as a near-duplicate.
pub const DUPLICATE_SIMILARITY_MAX: f32 = 0.85;

/// Result of running the evaluation gate on a single candidate.
#[derive(Debug, Clone)]
pub struct EvaluationReport {
    pub status: EvaluationStatus,
    pub reason: String,
    pub grounding_score: f32,
    pub max_similarity: f32,
}

impl EvaluationReport {
    pub fn approved(grounding_score: f32, max_similarity: f32) -> Self {
        EvaluationReport {
            status: EvaluationStatus::Approved,
            reason: String::new(),
            grounding_score,
            max_similarity,
        }
    }
}

/// A row logged to `bl_generation_eval` recording one candidate's gate outcome.
#[derive(Debug, Clone)]
pub struct GenerationEvalRecord {
    pub candidate_id: String,
    pub topic: String,
    pub model_version: String,
    pub provenance: Provenance,
    pub status: EvaluationStatus,
    pub reason: String,
    pub confidence: Option<f32>,
}

/// Run all four gates against a candidate draft. Returns the first failing gate,
/// or an approval if every gate passes.
pub fn evaluate_draft(
    draft: &GeneratedQuestionDraft,
    source: &SourceSection,
    gold: &GoldEvalSet,
    existing_normalized: &[String],
) -> EvaluationReport {
    // 1. Hallucination / structural validity.
    if let Some(reason) = hallucination_reason(draft) {
        return EvaluationReport {
            status: EvaluationStatus::RejectedHallucination,
            reason,
            grounding_score: 0.0,
            max_similarity: 0.0,
        };
    }

    // 2. Grounding: supported by the bundled source and/or held-out gold set.
    let grounding_score = grounding_score(draft, source, gold);
    if grounding_score < GROUNDING_MIN {
        return EvaluationReport {
            status: EvaluationStatus::RejectedUnsupported,
            reason: format!(
                "grounding score {grounding_score:.3} below minimum {GROUNDING_MIN:.3}; \
                 claims not supported by `{}`",
                source.section
            ),
            grounding_score,
            max_similarity: 0.0,
        };
    }

    // 3. Duplicate: near-duplicate of an existing bank question.
    let candidate_tokens = token_set(&draft.stem);
    let mut max_similarity = 0.0f32;
    for existing in existing_normalized {
        let similarity = jaccard(&candidate_tokens, &token_set(existing));
        if similarity > max_similarity {
            max_similarity = similarity;
        }
    }
    if max_similarity >= DUPLICATE_SIMILARITY_MAX {
        return EvaluationReport {
            status: EvaluationStatus::RejectedDuplicate,
            reason: format!(
                "near-duplicate of an existing question (similarity {max_similarity:.3} >= \
                 {DUPLICATE_SIMILARITY_MAX:.3})"
            ),
            grounding_score,
            max_similarity,
        };
    }

    EvaluationReport::approved(grounding_score, max_similarity)
}

/// Structural / hallucination checks. Returns a reason string when the item is
/// malformed or its answer is not derivable from the presented choices.
fn hallucination_reason(draft: &GeneratedQuestionDraft) -> Option<String> {
    if draft.stem.trim().is_empty() {
        return Some("empty stem".into());
    }
    if draft.correct_answer.trim().is_empty() {
        return Some("empty correct answer".into());
    }
    let non_empty_choices = draft
        .choices
        .iter()
        .filter(|c| !c.trim().is_empty())
        .count();
    if non_empty_choices < 2 {
        return Some(format!(
            "too few answer choices ({non_empty_choices}); need at least 2"
        ));
    }
    if !correct_answer_in_choices(&draft.correct_answer, &draft.choices) {
        return Some(
            "correct answer is not among the presented choices (answer not derivable)".into(),
        );
    }
    None
}

/// Grounding score = best keyword overlap between the candidate's stem +
/// explanation and either the bundled source excerpt or the held-out gold
/// questions for the same topic.
fn grounding_score(
    draft: &GeneratedQuestionDraft,
    source: &SourceSection,
    gold: &GoldEvalSet,
) -> f32 {
    let haystack = format!("{} {}", draft.stem, draft.explanation);
    let source_keywords: Vec<String> = source.keywords.iter().map(|k| (*k).to_string()).collect();
    let source_score = keyword_overlap(&haystack, &source_keywords);

    let gold_keywords: Vec<String> = gold
        .questions
        .iter()
        .filter(|q| q.topic == draft.topic)
        .flat_map(|q| q.keywords.clone())
        .collect();
    let gold_score = if gold_keywords.is_empty() {
        0.0
    } else {
        keyword_overlap(&haystack, &gold_keywords)
    };

    source_score.max(gold_score)
}

/// Normalize a stem for duplicate comparison: lowercase, drop punctuation,
/// collapse whitespace.
pub fn normalize_stem(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let mut prev_space = false;
    for ch in stem.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            prev_space = false;
        } else if !prev_space {
            out.push(' ');
            prev_space = true;
        }
    }
    out.trim().to_string()
}

fn token_set(text: &str) -> HashSet<String> {
    normalize_stem(text)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;
    use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
    use crate::gre_atlas::questions::source::source_section_for_topic;
    use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;

    fn draft(stem: &str, choices: &[&str], correct: &str, explanation: &str) -> GeneratedQuestionDraft {
        GeneratedQuestionDraft {
            id: "cand-1".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: stem.into(),
            choices: choices.iter().map(|c| c.to_string()).collect(),
            correct_answer: correct.into(),
            explanation: explanation.into(),
            difficulty: Some(0.4),
            confidence: 0.8,
            attribution: QuestionAttribution {
                source_name: GENERATION_SOURCE_NAME.into(),
                source_section: "Quantitative Reasoning — Linear equations".into(),
                generated_at_secs: 1,
            },
        }
    }

    fn source() -> &'static SourceSection {
        source_section_for_topic("gre::quant::algebra::linear").unwrap()
    }

    #[test]
    fn approves_a_grounded_unique_valid_question() {
        let gold = load_gold_eval_set().unwrap();
        let d = draft(
            "Solve the linear equation 2x + 3 = 11 for the variable x.",
            &["3", "4", "5", "6"],
            "4",
            "Subtract 3 then divide by 2: x = 4.",
        );
        let report = evaluate_draft(&d, source(), &gold, &[]);
        assert_eq!(report.status, EvaluationStatus::Approved, "{}", report.reason);
    }

    #[test]
    fn rejects_answer_not_in_choices_as_hallucination() {
        let gold = load_gold_eval_set().unwrap();
        let d = draft(
            "Solve the linear equation 2x + 3 = 11 for x.",
            &["3", "4", "5", "6"],
            "99",
            "x = 4.",
        );
        let report = evaluate_draft(&d, source(), &gold, &[]);
        assert_eq!(report.status, EvaluationStatus::RejectedHallucination);
    }

    #[test]
    fn rejects_ungrounded_claims_as_unsupported() {
        let gold = load_gold_eval_set().unwrap();
        let d = draft(
            "Which planet is closest to a purple banana on Tuesdays?",
            &["Mars", "Venus", "Jupiter", "Saturn"],
            "Venus",
            "Bananas orbit quickly.",
        );
        let report = evaluate_draft(&d, source(), &gold, &[]);
        assert_eq!(report.status, EvaluationStatus::RejectedUnsupported);
    }

    #[test]
    fn rejects_near_duplicate() {
        let gold = load_gold_eval_set().unwrap();
        let stem = "Solve the linear equation 2x + 3 = 11 for the variable x.";
        let d = draft(stem, &["3", "4", "5", "6"], "4", "x = 4 by isolating the variable.");
        let existing = vec![normalize_stem(stem)];
        let report = evaluate_draft(&d, source(), &gold, &existing);
        assert_eq!(report.status, EvaluationStatus::RejectedDuplicate);
    }

    #[test]
    fn normalize_is_case_and_punctuation_insensitive() {
        assert_eq!(
            normalize_stem("Solve:  2x + 3 = 11?!"),
            normalize_stem("solve 2x 3 11")
        );
    }
}
