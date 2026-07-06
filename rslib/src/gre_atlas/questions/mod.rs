// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod ai_gen;
pub mod bank;
pub mod eval_pipeline;
pub mod explanation;
pub mod foundation;
pub mod generator;
pub mod llm;
pub mod metadata;
pub mod retrieval;
pub mod source;
pub mod variants;

use anki_proto::brainlift::AnswerChoiceExplanation;
use anki_proto::brainlift::AnswerExplanation;
use anki_proto::brainlift::SolutionExplanation;
use anki_proto::brainlift::ExplainAnswerResponse;
use anki_proto::brainlift::GenerateQuestionResponse;
use anki_proto::brainlift::Question;
use anki_proto::brainlift::QuestionAttribution;
pub use bank::assert_practice_bank_listable;
pub use bank::ensure_exam_length_bank;
pub use bank::ensure_foundation_questions_present;
pub use bank::exam_bank_question_count;
pub use bank::load_practice_bank;
pub use bank::MIN_PRACTICE_BANK_PER_TOPIC;
pub use bank::PRACTICE_BANK_QUESTION_TOTAL;
pub use bank::PRACTICE_QUESTION_LIST_LIMIT;
pub use bank::TARGET_PRACTICE_BANK_AWA;
pub use bank::TARGET_PRACTICE_BANK_QUANT;
pub use bank::TARGET_PRACTICE_BANK_VERBAL;
pub use bank::practice_bank_ids;
pub use bank::purge_invalid_questions;
pub use bank::repair_sync_question_stubs;
pub use bank::target_count_for_topic;
pub use bank::validate_practice_bank;

use crate::collection::Collection;
use crate::error::OrInvalid;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;
use crate::gre_atlas::questions::eval_pipeline::normalize_stem;
use crate::gre_atlas::questions::explanation::build_answer_explanation;
use crate::gre_atlas::questions::explanation::AnswerExplanationData;
use crate::gre_atlas::questions::explanation::ChoiceExplanation;
use crate::gre_atlas::questions::explanation::SolutionExplanationData;
use crate::gre_atlas::questions::generator::generate_with_fallback;
use crate::gre_atlas::questions::generator::GeneratedQuestion;
use crate::gre_atlas::questions::llm::GreAtlasAiConfig;
use crate::gre_atlas::questions::llm::LlmClient;
use crate::gre_atlas::questions::llm::OpenAiLlmClient;
use crate::gre_atlas::questions::metadata::OFFLINE_TEMPLATE_NOTE;
use crate::gre_atlas::storage::StoredQuestion;
use crate::timestamp::TimestampSecs;

/// Map a stored question row to the public protobuf (never includes answers).
pub(crate) fn stored_question_to_proto(q: StoredQuestion) -> Question {
    let attribution = match (
        q.source_name,
        q.source_section,
        q.generated_at_secs,
        q.generation_confidence,
    ) {
        (Some(source_name), Some(source_section), Some(generated_at_secs), confidence) => {
            Some(QuestionAttribution {
                source_name,
                source_section,
                generated_at_secs,
                confidence,
                provenance: q.provenance.clone().unwrap_or_default(),
                model_version: q.model_version.clone().unwrap_or_default(),
                evaluation_status: q.evaluation_status.clone().unwrap_or_default(),
                source_document: q.source_document.clone().unwrap_or_default(),
            })
        }
        _ => None,
    };
    Question {
        id: q.id,
        topic: q.topic,
        section: q.section,
        format: q.format,
        stem: q.stem,
        choices: q.choices,
        attribution,
    }
}

impl Collection {
    pub fn gre_atlas_generate_question(
        &mut self,
        topic_id: &str,
        persist: bool,
    ) -> Result<GenerateQuestionResponse> {
        let now = TimestampSecs::now();
        // Optional, env-gated AI client. `None` (default) => deterministic path.
        let ai_client = build_ai_client(self);
        let gold = load_gold_eval_set()?;

        // Existing normalized stems power the duplicate-rejection gate.
        let existing_normalized: Vec<String> = {
            let storage = gre_atlas_storage(self)?;
            storage
                .list_questions("", u32::MAX)?
                .into_iter()
                .map(|q| normalize_stem(&q.stem))
                .collect()
        };

        let attempt = generate_with_fallback(
            topic_id,
            0,
            now,
            ai_client.as_deref(),
            &gold,
            &existing_normalized,
        );

        // Persist eval metrics + the approved question (if any).
        {
            let storage = gre_atlas_storage(self)?;
            for record in &attempt.eval_records {
                storage.record_generation_eval(record)?;
            }
            if persist {
                if let Some(question) = &attempt.question {
                    storage
                        .insert_generated_question_with_meta(&question.draft, &question.metadata)?;
                }
            }
        }

        match attempt.question {
            Some(question) => Ok(generated_question_response(question)),
            None => Ok(GenerateQuestionResponse {
                accepted: false,
                confidence: 0.0,
                rejection_reason: attempt
                    .fallback_reason
                    .unwrap_or_else(|| format!("unable to generate a question for {topic_id}")),
                question: None,
                attribution: None,
                provenance: String::new(),
                model_version: String::new(),
                evaluation_status: String::new(),
                provenance_note: String::new(),
            }),
        }
    }

    /// Build a post-answer explanation for a stored question. Uses the optional
    /// LLM path when enabled/reachable, otherwise a deterministic templated
    /// explanation. Never errors for AI-unavailability.
    pub fn gre_atlas_explain_answer(
        &mut self,
        question_id: &str,
        selected_answer: &str,
    ) -> Result<ExplainAnswerResponse> {
        let ai_client = build_ai_client(self);
        let storage = gre_atlas_storage(self)?;
        let question = storage
            .get_question(question_id)?
            .or_invalid("question not found")?;
        let data = build_answer_explanation(&question, selected_answer, ai_client.as_deref());
        let mut explanation = answer_explanation_to_proto(data);
        enrich_answer_explanation(&question, &mut explanation);
        Ok(ExplainAnswerResponse {
            explanation: Some(explanation),
        })
    }
}

/// Fill grounded, stored-data-derived fields the deterministic generator does
/// not already set: per-choice trap recognition and the solution's difficulty,
/// estimated time, related topics, and alternative method. Idempotent — only
/// fills values that are still empty (so an AI response can supply its own).
fn enrich_answer_explanation(
    question: &crate::gre_atlas::storage::StoredQuestion,
    explanation: &mut AnswerExplanation,
) {
    use crate::gre_atlas::questions::explanation::{
        alternative_method_for, concept_from_topic, difficulty_label, estimated_time_label,
        generic_trap_recognition, related_topics_for,
    };

    let concept = explanation
        .solution
        .as_ref()
        .map(|s| s.concept.clone())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| concept_from_topic(&question.topic));

    if let Some(solution) = explanation.solution.as_mut() {
        if solution.difficulty.is_empty() {
            solution.difficulty = difficulty_label(question.difficulty);
        }
        if solution.estimated_time.is_empty() {
            solution.estimated_time = estimated_time_label(question);
        }
        if solution.related_topics.is_empty() {
            solution.related_topics = related_topics_for(&question.topic);
        }
        if solution.alternative_method.is_empty() {
            solution.alternative_method = alternative_method_for(&solution.concept);
        }
    }

    for choice in explanation.choices.iter_mut() {
        if !choice.is_correct && choice.trap_recognition.is_empty() {
            choice.trap_recognition = generic_trap_recognition(&concept);
        }
    }
}

/// Construct the optional LLM client from the environment. Returns `None`
/// (feature disabled) unless `GRE_ATLAS_OPENAI_API_KEY` is set — this is what
/// keeps the default build/test path fully offline.
/// Collection-config key persisting the user's AI explanation toggle.
pub(crate) const GRE_ATLAS_AI_ENABLED_KEY: &str = "gre_atlas_ai_enabled";

/// User toggle for AI explanations (defaults on). Independent of whether an API
/// key is actually configured.
pub(crate) fn gre_atlas_ai_enabled(col: &Collection) -> bool {
    col.get_config_optional::<bool, _>(GRE_ATLAS_AI_ENABLED_KEY)
        .unwrap_or(true)
}

/// Whether an API key is configured so AI can run on this machine.
pub(crate) fn gre_atlas_ai_available() -> bool {
    GreAtlasAiConfig::from_env().is_some()
}

fn build_ai_client(col: &Collection) -> Option<Box<dyn LlmClient>> {
    if !gre_atlas_ai_enabled(col) {
        return None;
    }
    GreAtlasAiConfig::from_env()
        .map(|config| Box::new(OpenAiLlmClient::new(config)) as Box<dyn LlmClient>)
}

/// Map an approved generated question into the RPC response, including
/// provenance metadata and the offline-template note when applicable.
fn generated_question_response(question: GeneratedQuestion) -> GenerateQuestionResponse {
    // `evaluation` is captured for auditing; the eval grounding score is already
    // reflected in `draft.confidence` for AI items.
    let GeneratedQuestion {
        draft, metadata, ..
    } = question;
    let provenance = metadata.provenance.as_str().to_string();
    let evaluation_status = metadata.evaluation_status.as_str().to_string();
    let provenance_note = if metadata.provenance.is_ai() {
        String::new()
    } else {
        OFFLINE_TEMPLATE_NOTE.to_string()
    };

    let stored = StoredQuestion {
        id: draft.id.clone(),
        topic: draft.topic.clone(),
        section: draft.section.clone(),
        format: draft.format.clone(),
        stem: draft.stem.clone(),
        choices: draft.choices.clone(),
        correct_answer: draft.correct_answer.clone(),
        explanation: draft.explanation.clone(),
        difficulty: draft.difficulty,
        source_name: Some(draft.attribution.source_name.clone()),
        source_section: Some(draft.attribution.source_section.clone()),
        generated_at_secs: Some(draft.attribution.generated_at_secs),
        generation_confidence: Some(draft.confidence),
        source_document: Some(metadata.source_document.clone()),
        model_version: Some(metadata.model_version.clone()),
        provenance: Some(provenance.clone()),
        evaluation_status: Some(evaluation_status.clone()),
    };

    GenerateQuestionResponse {
        accepted: true,
        confidence: draft.confidence,
        rejection_reason: String::new(),
        question: Some(stored_question_to_proto(stored)),
        attribution: Some(QuestionAttribution {
            source_name: draft.attribution.source_name,
            source_section: draft.attribution.source_section,
            generated_at_secs: draft.attribution.generated_at_secs,
            confidence: Some(draft.confidence),
            provenance: provenance.clone(),
            model_version: metadata.model_version.clone(),
            evaluation_status: evaluation_status.clone(),
            source_document: metadata.source_document.clone(),
        }),
        provenance,
        model_version: metadata.model_version,
        evaluation_status,
        provenance_note,
    }
}

/// Map the internal explanation data to the protobuf message.
fn answer_explanation_to_proto(data: AnswerExplanationData) -> AnswerExplanation {
    AnswerExplanation {
        summary: data.summary,
        choices: data.choices.into_iter().map(choice_explanation_to_proto).collect(),
        correct_answer: data.correct_answer,
        citation_source_name: data.citation_source_name,
        citation_source_section: data.citation_source_section,
        citation_excerpt: data.citation_excerpt,
        provenance: data.provenance.as_str().to_string(),
        provenance_note: data.provenance_note,
        model_version: data.model_version,
        solution: data.solution.map(solution_explanation_to_proto),
    }
}

fn choice_explanation_to_proto(c: ChoiceExplanation) -> AnswerChoiceExplanation {
    AnswerChoiceExplanation {
        choice: c.choice,
        is_correct: c.is_correct,
        reasoning: c.reasoning,
        label: c.label,
        reason: c.reason,
        likely_misconception: c.likely_misconception,
        student_reasoning: c.student_reasoning,
        correct_reasoning: c.correct_reasoning,
        difference: c.difference,
        // Filled by enrich_answer_explanation (needs the question for grounding).
        trap_recognition: String::new(),
    }
}

fn solution_explanation_to_proto(s: SolutionExplanationData) -> SolutionExplanation {
    SolutionExplanation {
        concept: s.concept,
        formula: s.formula,
        steps: s.steps,
        final_answer: s.final_answer,
        common_mistake: s.common_mistake,
        key_takeaways: s.key_takeaways,
        citation: s.citation,
        // Filled by enrich_answer_explanation from the stored question.
        alternative_method: String::new(),
        difficulty: String::new(),
        estimated_time: String::new(),
        related_topics: Vec::new(),
    }
}

/// Whether `topic` equals `prefix` or is a descendant in the GRE tag hierarchy.
pub fn topic_matches_prefix(topic: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    topic == prefix
        || topic
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with("::"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topic_prefix_matching() {
        assert!(topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::quant"
        ));
        assert!(topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::quant::algebra::linear"
        ));
        assert!(!topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::verbal"
        ));
        assert!(!topic_matches_prefix(
            "gre::quant::algebra",
            "gre::quant::arithmetic"
        ));
        assert!(topic_matches_prefix("gre::quant", ""));
    }

    #[test]
    fn generate_question_persists_attribution() -> Result<()> {
        use crate::collection::CollectionBuilder;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let mut col = CollectionBuilder::new(&col_path).build()?;
        let resp = col.gre_atlas_generate_question("gre::quant::algebra::linear", true)?;
        assert!(resp.accepted);
        let attr = resp.attribution.unwrap();
        assert!(!attr.source_name.is_empty());
        assert!(!attr.source_section.is_empty());
        assert!(attr.generated_at_secs > 0);

        let storage = gre_atlas_storage(&mut col)?;
        let stored = storage.get_question(&resp.question.unwrap().id)?.unwrap();
        assert_eq!(
            stored.source_name.as_deref(),
            Some(attr.source_name.as_str())
        );
        assert_eq!(
            stored.source_section.as_deref(),
            Some(attr.source_section.as_str())
        );
        assert_eq!(stored.generated_at_secs, Some(attr.generated_at_secs));
        Ok(())
    }

    /// With no API key configured (the default), generation must silently use
    /// the deterministic offline-template path and surface the exact note.
    #[test]
    fn generate_question_defaults_to_offline_template_when_ai_disabled() -> Result<()> {
        use crate::collection::CollectionBuilder;

        // Guard: this integration test asserts the offline path, which requires
        // no API key. Skip the note assertion if a developer has one set.
        let ai_enabled = GreAtlasAiConfig::from_env().is_some();

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let mut col = CollectionBuilder::new(&col_path).build()?;
        let resp = col.gre_atlas_generate_question("gre::quant::algebra::linear", true)?;

        assert!(resp.accepted);
        assert_eq!(resp.evaluation_status, "approved");
        if !ai_enabled {
            assert_eq!(resp.provenance, "offline_template");
            assert_eq!(resp.provenance_note, OFFLINE_TEMPLATE_NOTE);
            assert_eq!(resp.model_version, "template_v1");
        }
        Ok(())
    }

    /// Explanations must always be produced (never error) and, in the offline
    /// default, carry per-choice reasoning, a citation, and the offline note.
    #[test]
    fn explain_answer_offline_has_citation_and_per_choice_reasoning() -> Result<()> {
        use crate::collection::CollectionBuilder;

        let ai_enabled = GreAtlasAiConfig::from_env().is_some();

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let mut col = CollectionBuilder::new(&col_path).build()?;

        let question = {
            let storage = gre_atlas_storage(&mut col)?;
            storage
                .list_questions("gre::quant::algebra::linear", 1)?
                .pop()
                .unwrap()
        };

        let resp = col.gre_atlas_explain_answer(&question.id, &question.choices[0])?;
        let explanation = resp.explanation.unwrap();

        // One reasoning entry per presented choice; exactly one marked correct.
        assert_eq!(explanation.choices.len(), question.choices.len());
        assert_eq!(
            explanation.choices.iter().filter(|c| c.is_correct).count(),
            1
        );
        assert!(!explanation.summary.is_empty());
        assert!(!explanation.correct_answer.is_empty());
        assert!(!explanation.citation_source_name.is_empty());
        // Foundation bank items cite the practice bank; generated items cite ETS.
        let is_generated = question.provenance.as_deref() == Some("offline_template")
            || question.provenance.as_deref() == Some("ai_generated")
            || question.source_name.as_deref() == Some("ETS Official GRE Prep Material");
        if is_generated {
            assert_eq!(
                explanation.citation_source_name,
                "ETS Official GRE Prep Material"
            );
        } else {
            assert_eq!(explanation.citation_source_name, "GRE Atlas Practice Bank");
        }
        if !ai_enabled {
            assert_eq!(explanation.provenance, "offline_template");
            assert_eq!(explanation.provenance_note, OFFLINE_TEMPLATE_NOTE);
        }
        Ok(())
    }
}
