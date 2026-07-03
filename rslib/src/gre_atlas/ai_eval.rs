// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::Serialize;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::questions::ai_gen::generate_question_for_topic;
use crate::gre_atlas::questions::ai_gen::keyword_overlap;
use crate::gre_atlas::questions::ai_gen::keyword_retrieve;
use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::questions::ai_gen::GoldEvalQuestion;
use crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION;
use crate::gre_atlas::questions::ai_gen::MIN_GENERATION_CONFIDENCE;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::timestamp::TimestampSecs;

#[derive(Debug, Clone, Serialize)]
pub struct GreAtlasAiEvalReport {
    pub generated_at_millis: i64,
    pub model_version: String,
    pub source_name: String,
    pub gold_set_label: String,
    pub gold_question_count: u32,
    pub confidence_threshold: f32,
    pub methodology: AiEvalMethodology,
    pub baseline_keyword_retrieval: KeywordRetrievalEval,
    pub template_generation: TemplateGenerationEval,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalMethodology {
    pub summary: String,
    pub generation_path: String,
    pub baseline_description: String,
    pub metrics: Vec<String>,
    pub reproducibility: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordRetrievalEval {
    pub topic_match_rate: f32,
    pub mean_keyword_recall: f32,
    pub evaluated_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateGenerationEval {
    pub acceptance_rate: f32,
    pub rejection_rate: f32,
    pub topic_match_rate: f32,
    pub mean_keyword_overlap: f32,
    pub evaluated_count: u32,
    pub accepted_count: u32,
    pub rejected_count: u32,
}

impl Collection {
    pub fn gre_atlas_generate_ai_eval_report(&mut self) -> Result<(String, String)> {
        let report = build_ai_eval_report()?;
        let json = serde_json::to_string_pretty(&report)?;
        let markdown = render_ai_eval_markdown(&report);
        Ok((json, markdown))
    }
}

pub fn build_ai_eval_report() -> Result<GreAtlasAiEvalReport> {
    let gold = load_gold_eval_set()?;
    let baseline = evaluate_keyword_baseline(&gold.questions);
    let generation = evaluate_template_generation(&gold.questions);

    Ok(GreAtlasAiEvalReport {
        generated_at_millis: TimestampSecs::now().0 * 1000,
        model_version: AI_GENERATION_MODEL_VERSION.into(),
        source_name: GENERATION_SOURCE_NAME.into(),
        gold_set_label: gold.label,
        gold_question_count: gold.questions.len() as u32,
        confidence_threshold: MIN_GENERATION_CONFIDENCE,
        methodology: ai_eval_methodology(),
        baseline_keyword_retrieval: baseline,
        template_generation: generation,
        limitations: ai_eval_limitations(),
    })
}

fn evaluate_keyword_baseline(gold: &[GoldEvalQuestion]) -> KeywordRetrievalEval {
    let mut topic_matches = 0u32;
    let mut recall_sum = 0.0f32;
    for question in gold {
        let result = keyword_retrieve(question);
        if result.topic_match {
            topic_matches += 1;
        }
        recall_sum += result.keyword_recall;
    }
    let count = gold.len() as u32;
    KeywordRetrievalEval {
        topic_match_rate: topic_matches as f32 / count as f32,
        mean_keyword_recall: recall_sum / count as f32,
        evaluated_count: count,
    }
}

fn evaluate_template_generation(gold: &[GoldEvalQuestion]) -> TemplateGenerationEval {
    let now = TimestampSecs(1_700_000_000);
    let mut accepted = 0u32;
    let mut rejected = 0u32;
    let mut topic_matches = 0u32;
    let mut overlap_sum = 0.0f32;

    for question in gold {
        match generate_question_for_topic(&question.topic, now) {
            GenerationOutcome::Accepted(draft) => {
                accepted += 1;
                if draft.topic == question.topic {
                    topic_matches += 1;
                }
                overlap_sum += keyword_overlap(&draft.stem, &question.keywords);
            }
            GenerationOutcome::Rejected { .. } => rejected += 1,
        }
    }

    let count = gold.len() as u32;
    TemplateGenerationEval {
        acceptance_rate: accepted as f32 / count as f32,
        rejection_rate: rejected as f32 / count as f32,
        topic_match_rate: topic_matches as f32 / accepted.max(1) as f32,
        mean_keyword_overlap: overlap_sum / accepted.max(1) as f32,
        evaluated_count: count,
        accepted_count: accepted,
        rejected_count: rejected,
    }
}

fn ai_eval_methodology() -> AiEvalMethodology {
    AiEvalMethodology {
        summary: "Compare keyword retrieval baseline against deterministic template generation on the static gold set.".into(),
        generation_path: format!(
            "Template generation from `{GENERATION_SOURCE_NAME}` with confidence threshold {MIN_GENERATION_CONFIDENCE}"
        ),
        baseline_description: "For each gold question, score every bundled source section by keyword overlap and pick the best match.".into(),
        metrics: vec![
            "topic_match_rate".into(),
            "mean_keyword_recall (baseline)".into(),
            "mean_keyword_overlap (generation)".into(),
            "acceptance_rate / rejection_rate".into(),
        ],
        reproducibility: "Gold eval uses a fixed timestamp (1700000000) for generation IDs and does not call external LLM APIs.".into(),
    }
}

fn ai_eval_limitations() -> Vec<String> {
    vec![
        "Template generation is not a large language model; optional LLM enhancement is env-gated and excluded from this eval.".into(),
        "Keyword baseline is a simple bag-of-words overlap, not semantic retrieval.".into(),
        "Gold questions are manually verified in-repo labels but do not measure live learner outcomes.".into(),
        "Topic match for generation is trivially high when a template exists for each gold topic.".into(),
    ]
}

pub fn render_ai_eval_markdown(report: &GreAtlasAiEvalReport) -> String {
    let mut out = String::new();
    out.push_str("# GRE Atlas AI evaluation report\n\n");
    out.push_str(&format!(
        "- Generated at (UTC millis): {}\n",
        report.generated_at_millis
    ));
    out.push_str(&format!("- Model version: `{}`\n", report.model_version));
    out.push_str(&format!("- Source: `{}`\n", report.source_name));
    out.push_str(&format!(
        "- Gold set: `{}` ({} questions)\n",
        report.gold_set_label, report.gold_question_count
    ));
    out.push_str(&format!(
        "- Confidence threshold: {:.2}\n\n",
        report.confidence_threshold
    ));

    out.push_str("## Methodology\n\n");
    out.push_str(&format!("{}\n\n", report.methodology.summary));
    out.push_str(&format!(
        "- Generation path: {}\n",
        report.methodology.generation_path
    ));
    out.push_str(&format!(
        "- Baseline: {}\n",
        report.methodology.baseline_description
    ));
    out.push_str(&format!(
        "- Reproducibility: {}\n\n",
        report.methodology.reproducibility
    ));

    out.push_str("## Baseline: keyword retrieval\n\n");
    out.push_str(&format!(
        "- Evaluated: {}\n",
        report.baseline_keyword_retrieval.evaluated_count
    ));
    out.push_str(&format!(
        "- Topic match rate: {:.1}%\n",
        report.baseline_keyword_retrieval.topic_match_rate * 100.0
    ));
    out.push_str(&format!(
        "- Mean keyword recall: {:.3}\n\n",
        report.baseline_keyword_retrieval.mean_keyword_recall
    ));

    out.push_str("## Template generation\n\n");
    out.push_str(&format!(
        "- Evaluated: {}\n",
        report.template_generation.evaluated_count
    ));
    out.push_str(&format!(
        "- Accepted: {} ({:.1}%)\n",
        report.template_generation.accepted_count,
        report.template_generation.acceptance_rate * 100.0
    ));
    out.push_str(&format!(
        "- Rejected: {} ({:.1}%)\n",
        report.template_generation.rejected_count,
        report.template_generation.rejection_rate * 100.0
    ));
    out.push_str(&format!(
        "- Topic match rate (accepted): {:.1}%\n",
        report.template_generation.topic_match_rate * 100.0
    ));
    out.push_str(&format!(
        "- Mean keyword overlap (accepted): {:.3}\n\n",
        report.template_generation.mean_keyword_overlap
    ));

    out.push_str("## Limitations\n\n");
    for limitation in &report.limitations {
        out.push_str(&format!("- {limitation}\n"));
    }

    out.push_str("\n## Reproducibility\n\n");
    out.push_str("Re-run with `just eval-gre_atlas-ai`.\n");
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ai_eval_report_is_deterministic_for_fixed_timestamp() {
        let first = build_ai_eval_report().unwrap();
        let second = build_ai_eval_report().unwrap();
        assert_eq!(
            first.template_generation.accepted_count,
            second.template_generation.accepted_count
        );
        assert_eq!(
            first.baseline_keyword_retrieval.topic_match_rate,
            second.baseline_keyword_retrieval.topic_match_rate
        );
    }

    #[test]
    fn markdown_includes_comparison_sections() {
        let report = build_ai_eval_report().unwrap();
        let md = render_ai_eval_markdown(&report);
        assert!(md.contains("Baseline: keyword retrieval"));
        assert!(md.contains("Template generation"));
        assert!(md.contains("Limitations"));
    }
}
