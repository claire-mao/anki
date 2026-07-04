// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Provider abstraction for question generation plus the fallback orchestrator.
//!
//! Two generators implement [`QuestionGenerator`]:
//!
//! * [`TemplateQuestionGenerator`] — the always-available deterministic
//!   fallback that wraps the existing template/variant path.
//! * [`LlmQuestionGenerator`] — the optional, env-gated real LLM path.
//!
//! [`generate_with_fallback`] composes them: it tries the LLM path (when
//! enabled), runs the eval gate, and — on *any* failure or rejection — falls
//! back to the deterministic template. It never returns an error for
//! AI-unavailability.

use crate::gre_atlas::questions::ai_gen::generate_question_for_topic_variant;
use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::questions::ai_gen::GoldEvalSet;
use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
use crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION;
use crate::gre_atlas::questions::eval_pipeline::evaluate_draft;
use crate::gre_atlas::questions::eval_pipeline::EvaluationReport;
use crate::gre_atlas::questions::eval_pipeline::GenerationEvalRecord;
use crate::gre_atlas::questions::foundation::exemplars_for_topic;
use crate::gre_atlas::questions::llm::LlmChatRequest;
use crate::gre_atlas::questions::llm::LlmClient;
use crate::gre_atlas::questions::metadata::EvaluationStatus;
use crate::gre_atlas::questions::metadata::Provenance;
use crate::gre_atlas::questions::metadata::QuestionMetadata;
use crate::gre_atlas::questions::source::source_section_for_topic;
use crate::gre_atlas::questions::source::SourceSection;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::timestamp::TimestampSecs;

/// A generated question that passed the eval gate and is ready to persist/serve.
#[derive(Debug, Clone)]
pub struct GeneratedQuestion {
    pub draft: GeneratedQuestionDraft,
    pub metadata: QuestionMetadata,
    pub evaluation: EvaluationReport,
}

/// Outcome of [`generate_with_fallback`], including audit records for metrics.
#[derive(Debug, Clone)]
pub struct GenerationAttempt {
    /// The question to persist/serve. `None` only when the topic is unknown or
    /// has no template mapping (nothing can be served, not an AI failure).
    pub question: Option<GeneratedQuestion>,
    /// Per-candidate eval records to log (e.g. a rejected AI candidate).
    pub eval_records: Vec<GenerationEvalRecord>,
    /// Whether the LLM path was attempted (client present).
    pub ai_attempted: bool,
    /// Whether the served question came from the deterministic fallback.
    pub used_fallback: bool,
    /// Human-readable reason the fallback was used, when applicable.
    pub fallback_reason: Option<String>,
}

/// A source of generated questions (deterministic template or real LLM).
pub trait QuestionGenerator {
    /// Attempt to generate a draft for the given topic/variant.
    fn generate(&self, topic_id: &str, variant: u32, now: TimestampSecs) -> GenerationOutcome;
    fn model_version(&self) -> &str;
    fn provenance(&self) -> Provenance;
}

/// The always-available deterministic generator. Wraps the existing
/// template/variant path unchanged.
pub struct TemplateQuestionGenerator;

impl QuestionGenerator for TemplateQuestionGenerator {
    fn generate(&self, topic_id: &str, variant: u32, now: TimestampSecs) -> GenerationOutcome {
        generate_question_for_topic_variant(topic_id, variant, now)
    }

    fn model_version(&self) -> &str {
        AI_GENERATION_MODEL_VERSION
    }

    fn provenance(&self) -> Provenance {
        Provenance::OfflineTemplate
    }
}

/// The optional real LLM generator. Builds a grounded prompt from the bundled
/// source material and manually authored exemplars, then parses the model's
/// JSON reply into a draft. Any transport/parse failure becomes a
/// [`GenerationOutcome::Rejected`] so the orchestrator can fall back.
pub struct LlmQuestionGenerator<'a> {
    client: &'a dyn LlmClient,
    /// Sampling temperature; low for reproducible, grounded items.
    temperature: f32,
}

impl<'a> LlmQuestionGenerator<'a> {
    pub fn new(client: &'a dyn LlmClient) -> Self {
        LlmQuestionGenerator {
            client,
            temperature: 0.4,
        }
    }
}

impl QuestionGenerator for LlmQuestionGenerator<'_> {
    fn generate(&self, topic_id: &str, variant: u32, now: TimestampSecs) -> GenerationOutcome {
        let Some(source) = source_section_for_topic(topic_id) else {
            return GenerationOutcome::Rejected {
                confidence: 0.0,
                reason: format!("no source section mapped for topic: {topic_id}"),
                attribution: empty_attribution(now),
            };
        };
        let prompt = build_generation_prompt(topic_id, source);
        let request = LlmChatRequest {
            system: GENERATION_SYSTEM_PROMPT.to_string(),
            user: prompt,
            temperature: self.temperature,
        };
        match self.client.complete(&request) {
            Ok(content) => match parse_generated_question_json(&content, topic_id, source, now, variant) {
                Ok(draft) => GenerationOutcome::Accepted(draft),
                Err(reason) => GenerationOutcome::Rejected {
                    confidence: 0.0,
                    reason,
                    attribution: source_attribution(source, now),
                },
            },
            Err(err) => GenerationOutcome::Rejected {
                confidence: 0.0,
                reason: err.to_string(),
                attribution: source_attribution(source, now),
            },
        }
    }

    fn model_version(&self) -> &str {
        self.client.model_version()
    }

    fn provenance(&self) -> Provenance {
        Provenance::AiGenerated
    }
}

/// Orchestrate generation with automatic, silent fallback to templates.
///
/// * When `ai` is `Some`, try the LLM path and run the eval gate. On approval,
///   return the AI question. On rejection or any error, log the outcome and fall
///   back.
/// * When `ai` is `None` (feature disabled) or the LLM path fails, return the
///   deterministic template question with offline-template provenance.
pub fn generate_with_fallback(
    topic_id: &str,
    variant: u32,
    now: TimestampSecs,
    ai: Option<&dyn LlmClient>,
    gold: &GoldEvalSet,
    existing_normalized: &[String],
) -> GenerationAttempt {
    let mut eval_records = Vec::new();
    let ai_attempted = ai.is_some();
    let mut fallback_reason = None;

    let source = source_section_for_topic(topic_id);

    // Try the real LLM path first when enabled.
    if let (Some(client), Some(source)) = (ai, source) {
        let generator = LlmQuestionGenerator::new(client);
        match generator.generate(topic_id, variant, now) {
            GenerationOutcome::Accepted(mut draft) => {
                let report = evaluate_draft(&draft, source, gold, existing_normalized);
                eval_records.push(GenerationEvalRecord {
                    candidate_id: draft.id.clone(),
                    topic: topic_id.to_string(),
                    model_version: generator.model_version().to_string(),
                    provenance: Provenance::AiGenerated,
                    status: report.status,
                    reason: report.reason.clone(),
                    confidence: Some(report.grounding_score),
                });
                if report.status.is_approved() {
                    // Persist the grounding score as the stored confidence.
                    draft.confidence = report.grounding_score;
                    return GenerationAttempt {
                        question: Some(GeneratedQuestion {
                            metadata: QuestionMetadata {
                                provenance: Provenance::AiGenerated,
                                model_version: generator.model_version().to_string(),
                                source_document: source.section.to_string(),
                                evaluation_status: EvaluationStatus::Approved,
                            },
                            draft,
                            evaluation: report,
                        }),
                        eval_records,
                        ai_attempted,
                        used_fallback: false,
                        fallback_reason: None,
                    };
                }
                fallback_reason = Some(report.reason);
            }
            GenerationOutcome::Rejected { reason, .. } => {
                fallback_reason = Some(reason);
            }
        }
    } else if ai_attempted {
        fallback_reason = Some(format!("no source section mapped for topic: {topic_id}"));
    }

    // Deterministic fallback (also the default path when AI is disabled).
    let template = TemplateQuestionGenerator;
    match template.generate(topic_id, variant, now) {
        GenerationOutcome::Accepted(draft) => {
            let source_document = source
                .map(|s| s.section.to_string())
                .unwrap_or_else(|| draft.attribution.source_section.clone());
            let metadata = QuestionMetadata::offline_template(
                template.model_version(),
                source_document,
            );
            let evaluation = EvaluationReport::approved(draft.confidence, 0.0);
            GenerationAttempt {
                question: Some(GeneratedQuestion {
                    draft,
                    metadata,
                    evaluation,
                }),
                eval_records,
                ai_attempted,
                used_fallback: true,
                fallback_reason,
            }
        }
        GenerationOutcome::Rejected { reason, .. } => GenerationAttempt {
            question: None,
            eval_records,
            ai_attempted,
            used_fallback: true,
            fallback_reason: fallback_reason.or(Some(reason)),
        },
    }
}

const GENERATION_SYSTEM_PROMPT: &str = "You are an expert GRE item writer. Produce a single \
    exam-quality multiple-choice practice question grounded ONLY in the provided source excerpt. \
    Do not invent facts beyond the excerpt. Reply with strict JSON and nothing else.";

/// Build the grounded user prompt from the source excerpt + exemplar stems.
fn build_generation_prompt(topic_id: &str, source: &SourceSection) -> String {
    let exemplars = exemplars_for_topic(topic_id);
    let exemplar_block = exemplars
        .iter()
        .take(2)
        .map(|q| format!("- {}", q.stem_text()))
        .collect::<Vec<_>>()
        .join("\n");
    let keywords = source.keywords.join(", ");

    format!(
        "Source: {source_name}\n\
         Section: {section}\n\
         Excerpt: {excerpt}\n\
         Key concepts: {keywords}\n\
         GRE section: {gre_section}\n\
         Topic id: {topic}\n\
         Example stems for this topic (style reference only):\n{exemplars}\n\n\
         Write ONE new multiple-choice question grounded in the excerpt above. \
         Provide exactly 4 answer choices, exactly one correct. The correct answer \
         string MUST appear verbatim in the choices array. Return JSON with this shape:\n\
         {{\"stem\": string, \"choices\": [string, string, string, string], \
         \"correct_answer\": string, \"explanation\": string}}",
        source_name = GENERATION_SOURCE_NAME,
        section = source.section,
        excerpt = source.excerpt,
        keywords = keywords,
        gre_section = source.gre_section,
        topic = topic_id,
        exemplars = if exemplar_block.is_empty() {
            "(none)".to_string()
        } else {
            exemplar_block
        },
    )
}

#[derive(serde::Deserialize)]
struct LlmQuestionJson {
    #[serde(default)]
    stem: String,
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    correct_answer: String,
    #[serde(default)]
    explanation: String,
}

/// Parse the model's JSON reply into a draft. Tolerates fenced code blocks and
/// surrounding prose by extracting the first JSON object.
pub fn parse_generated_question_json(
    content: &str,
    topic_id: &str,
    source: &SourceSection,
    now: TimestampSecs,
    variant: u32,
) -> Result<GeneratedQuestionDraft, String> {
    let json_slice = extract_json_object(content).ok_or_else(|| "no JSON object in reply".to_string())?;
    let parsed: LlmQuestionJson =
        serde_json::from_str(json_slice).map_err(|e| format!("invalid JSON: {e}"))?;

    if parsed.stem.trim().is_empty() {
        return Err("model reply missing stem".into());
    }
    if parsed.choices.len() < 2 {
        return Err("model reply has fewer than 2 choices".into());
    }
    if parsed.correct_answer.trim().is_empty() {
        return Err("model reply missing correct_answer".into());
    }

    let section = source.gre_section;
    Ok(GeneratedQuestionDraft {
        id: new_llm_id(topic_id, variant, now),
        topic: topic_id.to_string(),
        section: section.to_string(),
        format: "mcq".to_string(),
        stem: parsed.stem.trim().to_string(),
        choices: parsed
            .choices
            .into_iter()
            .map(|c| c.trim().to_string())
            .collect(),
        correct_answer: parsed.correct_answer.trim().to_string(),
        explanation: parsed.explanation.trim().to_string(),
        difficulty: Some(0.5),
        confidence: 0.0,
        attribution: source_attribution(source, now),
    })
}

/// Extract the first balanced `{...}` object from arbitrary text.
fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in text[start..].char_indices() {
        match ch {
            '"' if !escaped => in_string = !in_string,
            '\\' if in_string => {
                escaped = !escaped;
                continue;
            }
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..start + offset + 1]);
                }
            }
            _ => {}
        }
        escaped = false;
    }
    None
}

fn source_attribution(source: &SourceSection, now: TimestampSecs) -> QuestionAttribution {
    QuestionAttribution {
        source_name: GENERATION_SOURCE_NAME.to_string(),
        source_section: source.section.to_string(),
        generated_at_secs: now.0,
    }
}

fn empty_attribution(now: TimestampSecs) -> QuestionAttribution {
    QuestionAttribution {
        source_name: GENERATION_SOURCE_NAME.to_string(),
        source_section: "unknown".to_string(),
        generated_at_secs: now.0,
    }
}

fn new_llm_id(topic_id: &str, variant: u32, now: TimestampSecs) -> String {
    let slug = topic_id
        .strip_prefix("gre::")
        .unwrap_or(topic_id)
        .replace("::", "-");
    format!("ai-llm-{slug}-v{variant}-{}", now.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;
    use crate::gre_atlas::questions::llm::LlmError;
    use std::sync::Mutex;

    /// Stub client returning a fixed reply or a fixed error — no network.
    struct StubClient {
        reply: Result<String, LlmError>,
        seen: Mutex<Vec<LlmChatRequest>>,
    }

    impl StubClient {
        fn ok(json: &str) -> Self {
            StubClient {
                reply: Ok(json.to_string()),
                seen: Mutex::new(Vec::new()),
            }
        }
        fn err(err: LlmError) -> Self {
            StubClient {
                reply: Err(err),
                seen: Mutex::new(Vec::new()),
            }
        }
    }

    impl LlmClient for StubClient {
        fn complete(&self, request: &LlmChatRequest) -> Result<String, LlmError> {
            self.seen.lock().unwrap().push(request.clone());
            self.reply.clone()
        }
        fn model_version(&self) -> &str {
            "stub-model-v1"
        }
    }

    fn gold() -> GoldEvalSet {
        load_gold_eval_set().unwrap()
    }

    #[test]
    fn disabled_ai_uses_offline_template() {
        let attempt = generate_with_fallback(
            "gre::quant::algebra::linear",
            0,
            TimestampSecs(1_700_000_000),
            None,
            &gold(),
            &[],
        );
        let q = attempt.question.expect("template question present");
        assert_eq!(q.metadata.provenance, Provenance::OfflineTemplate);
        assert_eq!(q.metadata.model_version, AI_GENERATION_MODEL_VERSION);
        assert!(!attempt.ai_attempted);
        assert!(attempt.used_fallback);
    }

    #[test]
    fn approved_ai_reply_is_used() {
        let json = r#"{"stem":"Solve the linear equation 4x + 8 = 20 for the variable x.",
            "choices":["1","2","3","4"],"correct_answer":"3",
            "explanation":"Subtract 8 to get 4x = 12, then divide by 4 so x = 3."}"#;
        let client = StubClient::ok(json);
        let attempt = generate_with_fallback(
            "gre::quant::algebra::linear",
            0,
            TimestampSecs(1_700_000_000),
            Some(&client),
            &gold(),
            &[],
        );
        let q = attempt.question.expect("ai question present");
        assert_eq!(q.metadata.provenance, Provenance::AiGenerated);
        assert_eq!(q.metadata.model_version, "stub-model-v1");
        assert!(!attempt.used_fallback);
        assert_eq!(q.metadata.evaluation_status, EvaluationStatus::Approved);
        assert!(q.draft.id.starts_with("ai-llm-"));
    }

    #[test]
    fn hallucinated_ai_reply_falls_back_to_template() {
        // correct_answer not among the choices → hallucination rejection.
        let json = r#"{"stem":"Solve 2x + 1 = 7 for x.","choices":["1","2","4","5"],
            "correct_answer":"3","explanation":"x = 3."}"#;
        let client = StubClient::ok(json);
        let attempt = generate_with_fallback(
            "gre::quant::algebra::linear",
            0,
            TimestampSecs(1_700_000_000),
            Some(&client),
            &gold(),
            &[],
        );
        let q = attempt.question.expect("fallback question present");
        assert_eq!(q.metadata.provenance, Provenance::OfflineTemplate);
        assert!(attempt.used_fallback);
        assert!(attempt.ai_attempted);
        assert_eq!(attempt.eval_records.len(), 1);
        assert_eq!(
            attempt.eval_records[0].status,
            EvaluationStatus::RejectedHallucination
        );
    }

    #[test]
    fn transport_error_falls_back_silently() {
        let client = StubClient::err(LlmError::Transport("connection refused".into()));
        let attempt = generate_with_fallback(
            "gre::quant::algebra::linear",
            0,
            TimestampSecs(1_700_000_000),
            Some(&client),
            &gold(),
            &[],
        );
        let q = attempt.question.expect("fallback question present");
        assert_eq!(q.metadata.provenance, Provenance::OfflineTemplate);
        assert!(attempt.used_fallback);
        assert!(attempt.fallback_reason.is_some());
    }

    #[test]
    fn parses_json_wrapped_in_prose() {
        let content = "Here you go:\n```json\n{\"stem\":\"x\",\"choices\":[\"a\",\"b\"],\
            \"correct_answer\":\"a\",\"explanation\":\"because\"}\n```";
        let source = source_section_for_topic("gre::quant::algebra::linear").unwrap();
        let draft = parse_generated_question_json(
            content,
            "gre::quant::algebra::linear",
            source,
            TimestampSecs(1),
            0,
        )
        .unwrap();
        assert_eq!(draft.stem, "x");
        assert_eq!(draft.choices, vec!["a", "b"]);
    }
}
