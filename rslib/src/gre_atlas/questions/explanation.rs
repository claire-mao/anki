// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Post-answer explanations.
//!
//! Given a stored question and the learner's selected answer, produce a
//! structured explanation that (a) says why the correct answer is correct,
//! (b) says why *each* distractor is wrong, and (c) cites the grounding source.
//!
//! The optional LLM path generates this on demand; the deterministic fallback
//! builds it from the stored explanation plus per-distractor templated
//! reasoning. The fallback is always available and never errors — when it is
//! used, [`OFFLINE_TEMPLATE_NOTE`] is attached so the UI can surface exactly
//! "Generated using offline templates.".

use crate::gre_atlas::questions::foundation::FOUNDATION_SOURCE_NAME;
use crate::gre_atlas::questions::llm::LlmChatRequest;
use crate::gre_atlas::questions::llm::LlmClient;
use crate::gre_atlas::questions::metadata::Provenance;
use crate::gre_atlas::questions::metadata::OFFLINE_TEMPLATE_NOTE;
use crate::gre_atlas::questions::metadata::PROVENANCE_AI;
use crate::gre_atlas::questions::metadata::PROVENANCE_TEMPLATE;
use crate::gre_atlas::questions::source::source_section_for_topic;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::gre_atlas::storage::StoredQuestion;

/// Reasoning for a single answer choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceExplanation {
    pub choice: String,
    pub is_correct: bool,
    pub reasoning: String,
}

/// A complete, structured post-answer explanation.
#[derive(Debug, Clone)]
pub struct AnswerExplanationData {
    pub summary: String,
    pub choices: Vec<ChoiceExplanation>,
    pub correct_answer: String,
    pub citation_source_name: String,
    pub citation_source_section: String,
    pub citation_excerpt: String,
    pub provenance: Provenance,
    pub provenance_note: String,
    pub model_version: String,
}

/// Build an explanation for `question` given the learner's `selected_answer`.
///
/// Tries the LLM path when `ai` is `Some`; on any failure falls back to the
/// deterministic templated explanation. Never returns an error.
pub fn build_answer_explanation(
    question: &StoredQuestion,
    selected_answer: &str,
    ai: Option<&dyn LlmClient>,
) -> AnswerExplanationData {
    if let Some(client) = ai {
        if let Some(explanation) = try_llm_explanation(question, selected_answer, client) {
            return explanation;
        }
    }
    build_template_explanation(question)
}

/// Deterministic explanation from stored data + per-distractor templates.
pub fn build_template_explanation(question: &StoredQuestion) -> AnswerExplanationData {
    let summary = clean_explanation(&question.explanation);
    let citation = citation_for(question);

    let choices = question
        .choices
        .iter()
        .map(|choice| {
            let is_correct = choice.trim() == question.correct_answer.trim();
            let reasoning = if is_correct {
                if summary.is_empty() {
                    "This is the correct answer.".to_string()
                } else {
                    format!("Correct. {summary}")
                }
            } else {
                distractor_reasoning(&question.correct_answer, &summary)
            };
            ChoiceExplanation {
                choice: choice.clone(),
                is_correct,
                reasoning,
            }
        })
        .collect();

    AnswerExplanationData {
        summary: if summary.is_empty() {
            format!("The correct answer is \"{}\".", question.correct_answer)
        } else {
            summary
        },
        choices,
        correct_answer: question.correct_answer.clone(),
        citation_source_name: citation.0,
        citation_source_section: citation.1,
        citation_excerpt: citation.2,
        provenance: Provenance::OfflineTemplate,
        provenance_note: OFFLINE_TEMPLATE_NOTE.to_string(),
        model_version: crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION.to_string(),
    }
}

/// Templated reasoning for a distractor. References the correct answer and the
/// grounding explanation so the learner sees *why* it is wrong.
fn distractor_reasoning(correct_answer: &str, summary: &str) -> String {
    if summary.is_empty() {
        format!(
            "Incorrect. This option does not follow from the question; the correct answer is \"{correct_answer}\"."
        )
    } else {
        format!(
            "Incorrect. This option does not match the reasoning that yields the correct answer, \
             \"{correct_answer}\": {summary}"
        )
    }
}

fn try_llm_explanation(
    question: &StoredQuestion,
    selected_answer: &str,
    client: &dyn LlmClient,
) -> Option<AnswerExplanationData> {
    let prompt = build_explanation_prompt(question, selected_answer);
    let request = LlmChatRequest {
        system: EXPLANATION_SYSTEM_PROMPT.to_string(),
        user: prompt,
        temperature: 0.3,
    };
    let content = client.complete(&request).ok()?;
    let parsed = parse_explanation_json(&content)?;

    // Require that the model addressed the correct answer; otherwise fall back.
    if parsed.summary.trim().is_empty() || parsed.choices.is_empty() {
        return None;
    }

    let citation = citation_for(question);
    let choices = question
        .choices
        .iter()
        .map(|choice| {
            let is_correct = choice.trim() == question.correct_answer.trim();
            let reasoning = parsed
                .choices
                .iter()
                .find(|c| c.choice.trim() == choice.trim())
                .map(|c| c.reasoning.clone())
                .filter(|r| !r.trim().is_empty())
                .unwrap_or_else(|| {
                    distractor_reasoning(&question.correct_answer, parsed.summary.trim())
                });
            ChoiceExplanation {
                choice: choice.clone(),
                is_correct,
                reasoning,
            }
        })
        .collect();

    Some(AnswerExplanationData {
        summary: parsed.summary.trim().to_string(),
        choices,
        correct_answer: question.correct_answer.clone(),
        citation_source_name: citation.0,
        citation_source_section: citation.1,
        citation_excerpt: citation.2,
        provenance: Provenance::AiGenerated,
        provenance_note: format!("Explained by {}.", client.model_version()),
        model_version: client.model_version().to_string(),
    })
}

/// Whether this row was produced by the template/LLM generation pipeline and
/// should cite the bundled ETS excerpts — not manually authored foundation
/// items.
fn question_uses_ets_grounding(question: &StoredQuestion) -> bool {
    matches!(
        question.provenance.as_deref(),
        Some(PROVENANCE_AI) | Some(PROVENANCE_TEMPLATE)
    ) || question.source_name.as_deref() == Some(GENERATION_SOURCE_NAME)
}

/// Citation triple: (source_name, source_section, excerpt).
///
/// Generated items cite the bundled ETS excerpt they were grounded in.
/// Foundation bank items cite their stored named source and never fabricate an
/// ETS excerpt.
fn citation_for(question: &StoredQuestion) -> (String, String, String) {
    if question_uses_ets_grounding(question) {
        if let Some(source) = source_section_for_topic(&question.topic) {
            let section = question
                .source_section
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(source.section);
            return (
                GENERATION_SOURCE_NAME.to_string(),
                section.to_string(),
                source.excerpt.to_string(),
            );
        }
    }

    let name = question
        .source_name
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| FOUNDATION_SOURCE_NAME.to_string());
    let section = question
        .source_section
        .clone()
        .or_else(|| question.source_document.clone())
        .unwrap_or_default();
    (name, section, String::new())
}

/// Strip the embedded `<!-- meta: ... -->` block foundation questions append to
/// their explanation, plus surrounding whitespace.
fn clean_explanation(explanation: &str) -> String {
    let without_meta = match explanation.find("<!-- meta:") {
        Some(idx) => &explanation[..idx],
        None => explanation,
    };
    without_meta.trim().to_string()
}

const EXPLANATION_SYSTEM_PROMPT: &str = "You are a patient GRE tutor. Explain why the correct \
    answer is correct and why every other option is wrong, grounded in the provided source \
    excerpt. Reply with strict JSON and nothing else.";

fn build_explanation_prompt(question: &StoredQuestion, selected_answer: &str) -> String {
    let (source_name, source_section, excerpt) = citation_for(question);
    let choices = question
        .choices
        .iter()
        .map(|c| format!("- {c}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {source_name}\nSection: {source_section}\nExcerpt: {excerpt}\n\n\
         Question: {stem}\nChoices:\n{choices}\nCorrect answer: {correct}\n\
         Learner selected: {selected}\n\n\
         Return JSON: {{\"summary\": string, \"choices\": [{{\"choice\": string, \
         \"is_correct\": boolean, \"reasoning\": string}}]}}. Include one entry per choice.",
        stem = question.stem,
        correct = question.correct_answer,
        selected = selected_answer,
    )
}

#[derive(serde::Deserialize)]
struct ExplanationJson {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    choices: Vec<ExplanationChoiceJson>,
}

#[derive(serde::Deserialize)]
struct ExplanationChoiceJson {
    #[serde(default)]
    choice: String,
    #[serde(default)]
    reasoning: String,
}

fn parse_explanation_json(content: &str) -> Option<ExplanationJson> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&content[start..=end]).ok()
}

#[cfg(test)]
mod test {
    use super::*;

    fn stored(explanation: &str) -> StoredQuestion {
        StoredQuestion {
            id: "q1".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: "If 4x + 8 = 20, what is x?".into(),
            choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            correct_answer: "3".into(),
            explanation: explanation.into(),
            difficulty: Some(0.4),
            source_name: None,
            source_section: None,
            generated_at_secs: None,
            generation_confidence: None,
            source_document: None,
            model_version: None,
            provenance: None,
            evaluation_status: None,
        }
    }

    #[test]
    fn template_explanation_covers_every_choice_and_cites_source() {
        let mut q = stored("Subtract 8 then divide by 4: x = 3.");
        q.source_name = Some(GENERATION_SOURCE_NAME.to_string());
        q.provenance = Some(PROVENANCE_TEMPLATE.to_string());
        let out = build_template_explanation(&q);
        assert_eq!(out.provenance, Provenance::OfflineTemplate);
        assert_eq!(out.provenance_note, OFFLINE_TEMPLATE_NOTE);
        assert_eq!(out.choices.len(), 4);
        let correct: Vec<_> = out.choices.iter().filter(|c| c.is_correct).collect();
        assert_eq!(correct.len(), 1);
        assert_eq!(correct[0].choice, "3");
        assert!(correct[0].reasoning.contains("Subtract 8"));
        // every distractor names the correct answer
        for c in out.choices.iter().filter(|c| !c.is_correct) {
            assert!(c.reasoning.contains("\"3\""), "{}", c.reasoning);
        }
        assert_eq!(out.citation_source_name, GENERATION_SOURCE_NAME);
        assert!(!out.citation_excerpt.is_empty());
    }

    #[test]
    fn template_explanation_strips_meta_comment() {
        let q = stored("x = 3.\n\n<!-- meta: {\"subtopic\":\"linear\"} -->");
        let out = build_template_explanation(&q);
        assert!(!out.summary.contains("meta"));
        assert_eq!(out.summary, "x = 3.");
    }

    #[test]
    fn build_uses_template_when_ai_absent() {
        let q = stored("x = 3.");
        let out = build_answer_explanation(&q, "1", None);
        assert_eq!(out.provenance, Provenance::OfflineTemplate);
    }

    #[test]
    fn foundation_question_cites_practice_bank_not_ets() {
        let mut q = stored("20% of $120 is $24; $120 − $24 = $96.");
        q.source_name = Some(FOUNDATION_SOURCE_NAME.to_string());
        q.source_section = Some("percent_discount".into());
        let out = build_template_explanation(&q);
        assert_eq!(out.citation_source_name, FOUNDATION_SOURCE_NAME);
        assert_eq!(out.citation_source_section, "percent_discount");
        assert!(out.citation_excerpt.is_empty());
        assert_ne!(out.citation_source_name, GENERATION_SOURCE_NAME);
    }

    #[test]
    fn generated_question_cites_ets_excerpt() {
        let mut q = stored("Subtract 8 then divide by 4: x = 3.");
        q.source_name = Some(GENERATION_SOURCE_NAME.to_string());
        q.source_section = Some("Quantitative Reasoning — Linear equations".into());
        q.provenance = Some(PROVENANCE_TEMPLATE.to_string());
        let out = build_template_explanation(&q);
        assert_eq!(out.citation_source_name, GENERATION_SOURCE_NAME);
        assert!(!out.citation_excerpt.is_empty());
    }
}
