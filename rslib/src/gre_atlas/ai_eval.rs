// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::collections::HashSet;

use serde::Serialize;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::questions::ai_gen::generate_question_for_topic_variant;
use crate::gre_atlas::questions::ai_gen::keyword_overlap;
use crate::gre_atlas::questions::ai_gen::keyword_retrieve;
use crate::gre_atlas::questions::ai_gen::load_gold_eval_set;
use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::questions::ai_gen::GoldEvalQuestion;
use crate::gre_atlas::questions::ai_gen::GoldEvalSet;
use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
use crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION;
use crate::gre_atlas::questions::ai_gen::MIN_GENERATION_CONFIDENCE;
use crate::gre_atlas::questions::eval_pipeline::evaluate_draft;
use crate::gre_atlas::questions::eval_pipeline::normalize_stem;
use crate::gre_atlas::questions::metadata::EvaluationStatus;
use crate::gre_atlas::questions::retrieval::retrieve;
use crate::gre_atlas::questions::retrieval::RetrievalMethod;
use crate::gre_atlas::questions::source::source_section_for_topic;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::timestamp::TimestampSecs;

const EVAL_TIMESTAMP: TimestampSecs = TimestampSecs(1_700_000_000);
const GENERATION_VARIANTS: u32 = 8;

/// Minimum share of held-out gold topics that must produce an approved
/// question.
pub const DEFAULT_MIN_ACCURACY: f32 = 0.95;
/// Maximum share of held-out gold topics with a wrong marked answer.
pub const DEFAULT_MAX_WRONG_ANSWER_RATE: f32 = 0.0;

#[derive(Debug, Clone, Serialize)]
pub struct GreAtlasAiEvalReport {
    pub generated_at_millis: i64,
    pub model_version: String,
    pub source_name: String,
    pub gold_set_label: String,
    pub gold_question_count: u32,
    pub confidence_threshold: f32,
    pub acceptance_criteria: AiEvalAcceptanceCriteria,
    pub methodology: AiEvalMethodology,
    pub query_mode: String,
    pub held_out_quality: HeldOutQualityEval,
    pub systems: Vec<SystemEvalMetrics>,
    pub comparison: BenchmarkComparison,
    pub baseline_keyword_retrieval: KeywordRetrievalEval,
    pub template_generation: TemplateGenerationEval,
    pub rejection_pipeline: RejectionPipelineEval,
    pub verdict: AiEvalVerdict,
    pub limitations: Vec<String>,
}

/// Configurable release thresholds. Override via environment variables
/// documented in `scripts/eval/README.md`.
#[derive(Debug, Clone, Serialize)]
pub struct AiEvalAcceptanceCriteria {
    pub min_accuracy: f32,
    pub max_wrong_answer_rate: f32,
    /// Minimum generation confidence required before the eval gate runs.
    pub acceptance_cutoff: f32,
}

/// Quality metrics computed only on the held-out gold set. Gold labels are
/// never fed into generation; only the topic id is used to pick a candidate.
#[derive(Debug, Clone, Serialize)]
pub struct HeldOutQualityEval {
    pub evaluated_count: u32,
    pub approved_count: u32,
    pub wrong_answer_count: u32,
    pub low_confidence_count: u32,
    pub other_rejection_count: u32,
    pub skipped_count: u32,
    pub accuracy: f32,
    pub wrong_answer_rate: f32,
    pub dataset_label: String,
    pub eval_rule: String,
    pub failing_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEvalVerdict {
    pub passed: bool,
    pub accuracy: f32,
    pub wrong_answer_rate: f32,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemEvalMetrics {
    pub system_id: String,
    pub system_label: String,
    pub role: String,
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub failure_rate: f32,
    pub mean_keyword_recall: f32,
    pub evaluated_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkComparison {
    pub primary_metric: String,
    pub ai_system_id: String,
    pub ai_beats_all_baselines: bool,
    pub ai_rank_by_accuracy: u32,
    pub baseline_ids: Vec<String>,
    pub winner_by_accuracy: String,
    pub deltas_vs_best_baseline: HashMap<String, f32>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectionPipelineEval {
    pub evaluated_count: u32,
    pub approved_count: u32,
    pub rejected_hallucination: u32,
    pub rejected_duplicate: u32,
    pub rejected_unsupported: u32,
    pub sample_reasons: Vec<String>,
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

pub fn acceptance_criteria_from_env() -> AiEvalAcceptanceCriteria {
    AiEvalAcceptanceCriteria {
        min_accuracy: env_f32("GRE_ATLAS_AI_EVAL_MIN_ACCURACY").unwrap_or(DEFAULT_MIN_ACCURACY),
        max_wrong_answer_rate: env_f32("GRE_ATLAS_AI_EVAL_MAX_WRONG_ANSWER_RATE")
            .unwrap_or(DEFAULT_MAX_WRONG_ANSWER_RATE),
        acceptance_cutoff: env_f32("GRE_ATLAS_AI_EVAL_ACCEPTANCE_CUTOFF")
            .unwrap_or(MIN_GENERATION_CONFIDENCE),
    }
}

pub fn build_ai_eval_report() -> Result<GreAtlasAiEvalReport> {
    build_ai_eval_report_with_criteria(acceptance_criteria_from_env())
}

pub fn build_ai_eval_report_with_criteria(
    criteria: AiEvalAcceptanceCriteria,
) -> Result<GreAtlasAiEvalReport> {
    let gold = load_gold_eval_set()?;
    let baseline = evaluate_keyword_baseline(&gold.questions);
    let generation = evaluate_template_generation(&gold.questions, &gold, &criteria);
    let rejection_pipeline = evaluate_rejection_pipeline(&gold);
    let held_out_quality = evaluate_held_out_quality(&gold, &criteria);

    let retrieval_systems = vec![
        evaluate_retrieval_system(
            &gold.questions,
            "baseline_keyword",
            "Keyword overlap baseline",
            "baseline",
            RetrievalMethod::Keyword,
        ),
        evaluate_retrieval_system(
            &gold.questions,
            "baseline_bm25",
            "BM25 baseline",
            "baseline",
            RetrievalMethod::Bm25,
        ),
        evaluate_retrieval_system(
            &gold.questions,
            "baseline_vector_tfidf",
            "TF-IDF vector baseline",
            "baseline",
            RetrievalMethod::VectorTfidf,
        ),
        evaluate_retrieval_system(
            &gold.questions,
            "ai_retrieval",
            "Catalog-aware AI retrieval",
            "ai",
            RetrievalMethod::AiEnhanced,
        ),
    ];
    let ai_generation = evaluate_ai_generation_pipeline(&gold);
    let mut systems = retrieval_systems;
    systems.push(ai_generation);

    let comparison = build_comparison(&systems);
    let verdict = compute_verdict(&held_out_quality, &rejection_pipeline, &criteria);

    Ok(GreAtlasAiEvalReport {
        generated_at_millis: TimestampSecs::now().0 * 1000,
        model_version: AI_GENERATION_MODEL_VERSION.into(),
        source_name: GENERATION_SOURCE_NAME.into(),
        gold_set_label: gold.label.clone(),
        gold_question_count: gold.questions.len() as u32,
        confidence_threshold: criteria.acceptance_cutoff,
        acceptance_criteria: criteria,
        methodology: ai_eval_methodology(),
        query_mode: "stem_only".into(),
        held_out_quality,
        systems,
        comparison,
        baseline_keyword_retrieval: baseline,
        template_generation: generation,
        rejection_pipeline,
        verdict,
        limitations: ai_eval_limitations(),
    })
}

fn env_f32(name: &str) -> Option<f32> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeldOutOutcome {
    Approved,
    WrongAnswer,
    LowConfidence,
    OtherRejection,
    Skipped,
}

fn evaluate_held_out_quality(
    gold: &GoldEvalSet,
    criteria: &AiEvalAcceptanceCriteria,
) -> HeldOutQualityEval {
    let mut approved = 0u32;
    let mut wrong_answer = 0u32;
    let mut low_confidence = 0u32;
    let mut other_rejection = 0u32;
    let mut skipped = 0u32;
    let mut failing_topics: Vec<String> = Vec::new();

    for question in &gold.questions {
        match classify_held_out_question(question, gold, criteria) {
            HeldOutOutcome::Approved => approved += 1,
            HeldOutOutcome::WrongAnswer => {
                wrong_answer += 1;
                push_failing_topic(&mut failing_topics, &question.topic);
            }
            HeldOutOutcome::LowConfidence => {
                low_confidence += 1;
                push_failing_topic(&mut failing_topics, &question.topic);
            }
            HeldOutOutcome::OtherRejection => {
                other_rejection += 1;
                push_failing_topic(&mut failing_topics, &question.topic);
            }
            HeldOutOutcome::Skipped => skipped += 1,
        }
    }

    let evaluated = gold.questions.len() as u32;
    let accuracy = approved as f32 / evaluated.max(1) as f32;
    let wrong_answer_rate = wrong_answer as f32 / evaluated.max(1) as f32;

    HeldOutQualityEval {
        evaluated_count: evaluated,
        approved_count: approved,
        wrong_answer_count: wrong_answer,
        low_confidence_count: low_confidence,
        other_rejection_count: other_rejection,
        skipped_count: skipped,
        accuracy,
        wrong_answer_rate,
        dataset_label: gold.label.clone(),
        eval_rule: "Generate from gold topic id only (stem/answer/keywords withheld); \
                     accept first variant passing confidence cutoff and eval gate"
            .into(),
        failing_topics,
    }
}

fn push_failing_topic(topics: &mut Vec<String>, topic: &str) {
    if !topics.iter().any(|existing| existing == topic) {
        topics.push(topic.to_string());
    }
}

fn classify_held_out_question(
    question: &GoldEvalQuestion,
    gold: &GoldEvalSet,
    criteria: &AiEvalAcceptanceCriteria,
) -> HeldOutOutcome {
    let Some(source) = source_section_for_topic(&question.topic) else {
        return HeldOutOutcome::Skipped;
    };

    if best_approved_variant(&question.topic, source, gold, EVAL_TIMESTAMP, criteria).is_some() {
        return HeldOutOutcome::Approved;
    }

    let mut saw_wrong_answer = false;
    let mut saw_low_confidence = false;
    let mut saw_other_rejection = false;

    for variant in 0..GENERATION_VARIANTS {
        match generate_question_for_topic_variant(&question.topic, variant, EVAL_TIMESTAMP) {
            GenerationOutcome::Accepted(draft) => {
                if draft.confidence < criteria.acceptance_cutoff {
                    saw_low_confidence = true;
                    continue;
                }
                match evaluate_draft(&draft, source, gold, &[]).status {
                    EvaluationStatus::Approved => return HeldOutOutcome::Approved,
                    EvaluationStatus::RejectedHallucination => saw_wrong_answer = true,
                    EvaluationStatus::RejectedDuplicate | EvaluationStatus::RejectedUnsupported => {
                        saw_other_rejection = true;
                    }
                    EvaluationStatus::Pending => saw_other_rejection = true,
                }
            }
            GenerationOutcome::Rejected { .. } => saw_low_confidence = true,
        }
    }

    if saw_wrong_answer {
        HeldOutOutcome::WrongAnswer
    } else if saw_low_confidence {
        HeldOutOutcome::LowConfidence
    } else if saw_other_rejection {
        HeldOutOutcome::OtherRejection
    } else {
        HeldOutOutcome::Skipped
    }
}

fn compute_verdict(
    held_out: &HeldOutQualityEval,
    pipeline: &RejectionPipelineEval,
    criteria: &AiEvalAcceptanceCriteria,
) -> AiEvalVerdict {
    let mut failure_reasons = Vec::new();

    if held_out.accuracy < criteria.min_accuracy {
        failure_reasons.push(format!(
            "held-out accuracy {:.1}% below minimum {:.1}%",
            held_out.accuracy * 100.0,
            criteria.min_accuracy * 100.0
        ));
    }
    if held_out.wrong_answer_rate > criteria.max_wrong_answer_rate {
        failure_reasons.push(format!(
            "wrong-answer rate {:.1}% exceeds maximum {:.1}%",
            held_out.wrong_answer_rate * 100.0,
            criteria.max_wrong_answer_rate * 100.0
        ));
    }
    if pipeline.rejected_hallucination < 1 {
        failure_reasons
            .push("rejection pipeline did not reject a crafted hallucination example".into());
    }
    if pipeline.rejected_duplicate < 1 {
        failure_reasons
            .push("rejection pipeline did not reject a crafted duplicate example".into());
    }
    if pipeline.rejected_unsupported < 1 {
        failure_reasons
            .push("rejection pipeline did not reject a crafted unsupported example".into());
    }

    AiEvalVerdict {
        passed: failure_reasons.is_empty(),
        accuracy: held_out.accuracy,
        wrong_answer_rate: held_out.wrong_answer_rate,
        failure_reasons,
    }
}

fn evaluate_retrieval_system(
    gold: &[GoldEvalQuestion],
    system_id: &str,
    system_label: &str,
    role: &str,
    method: RetrievalMethod,
) -> SystemEvalMetrics {
    let predictions: Vec<(String, bool, f32)> = gold
        .iter()
        .map(|question| {
            let result = retrieve(question, method);
            (result.predicted_topic, result.failed, result.keyword_recall)
        })
        .collect();
    metrics_from_predictions(gold, system_id, system_label, role, &predictions)
}

fn evaluate_ai_generation_pipeline(gold: &GoldEvalSet) -> SystemEvalMetrics {
    let mut predictions = Vec::with_capacity(gold.questions.len());

    for question in &gold.questions {
        let retrieval = retrieve(question, RetrievalMethod::AiEnhanced);
        let mut failed = retrieval.failed;
        let mut keyword_recall = retrieval.keyword_recall;

        if let Some(source) = source_section_for_topic(&retrieval.predicted_topic) {
            if let Some((draft, _report)) = best_approved_variant(
                &retrieval.predicted_topic,
                source,
                gold,
                EVAL_TIMESTAMP,
                &AiEvalAcceptanceCriteria {
                    min_accuracy: DEFAULT_MIN_ACCURACY,
                    max_wrong_answer_rate: DEFAULT_MAX_WRONG_ANSWER_RATE,
                    acceptance_cutoff: MIN_GENERATION_CONFIDENCE,
                },
            ) {
                keyword_recall =
                    keyword_recall.max(keyword_overlap(&draft.stem, &question.keywords));
            } else {
                failed = true;
            }
        } else {
            failed = true;
        }

        predictions.push((retrieval.predicted_topic, failed, keyword_recall));
    }

    metrics_from_predictions(
        &gold.questions,
        "ai_generation_pipeline",
        "AI retrieval + best-variant generation + eval gate",
        "ai",
        &predictions,
    )
}

fn metrics_from_predictions(
    gold: &[GoldEvalQuestion],
    system_id: &str,
    system_label: &str,
    role: &str,
    predictions: &[(String, bool, f32)],
) -> SystemEvalMetrics {
    let count = gold.len() as u32;
    let mut correct = 0u32;
    let mut failures = 0u32;
    let mut recall_sum = 0.0f32;

    let mut topics: HashSet<String> = HashSet::new();
    for question in gold {
        topics.insert(question.topic.clone());
    }

    let mut tp: HashMap<String, u32> = HashMap::new();
    let mut fp: HashMap<String, u32> = HashMap::new();
    let mut fn_count: HashMap<String, u32> = HashMap::new();

    for topic in &topics {
        tp.insert(topic.clone(), 0);
        fp.insert(topic.clone(), 0);
        fn_count.insert(topic.clone(), 0);
    }

    for (question, (predicted, failed, keyword_recall)) in gold.iter().zip(predictions.iter()) {
        if *failed {
            failures += 1;
        }
        recall_sum += keyword_recall;
        let matched = predicted == &question.topic;
        if matched {
            correct += 1;
            *tp.entry(question.topic.clone()).or_insert(0) += 1;
        } else {
            *fp.entry(predicted.clone()).or_insert(0) += 1;
            *fn_count.entry(question.topic.clone()).or_insert(0) += 1;
        }
    }

    let mut precision_sum = 0.0f32;
    let mut recall_macro_sum = 0.0f32;
    let topic_count = topics.len().max(1) as f32;
    for topic in &topics {
        let tp_t = *tp.get(topic).unwrap_or(&0) as f32;
        let fp_t = *fp.get(topic).unwrap_or(&0) as f32;
        let fn_t = *fn_count.get(topic).unwrap_or(&0) as f32;
        precision_sum += if tp_t + fp_t > 0.0 {
            tp_t / (tp_t + fp_t)
        } else {
            1.0
        };
        recall_macro_sum += if tp_t + fn_t > 0.0 {
            tp_t / (tp_t + fn_t)
        } else {
            1.0
        };
    }

    let precision = precision_sum / topic_count;
    let recall = recall_macro_sum / topic_count;
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    SystemEvalMetrics {
        system_id: system_id.into(),
        system_label: system_label.into(),
        role: role.into(),
        accuracy: correct as f32 / count.max(1) as f32,
        precision,
        recall,
        f1,
        failure_rate: failures as f32 / count.max(1) as f32,
        mean_keyword_recall: recall_sum / count.max(1) as f32,
        evaluated_count: count,
    }
}

fn build_comparison(systems: &[SystemEvalMetrics]) -> BenchmarkComparison {
    let baselines: Vec<&SystemEvalMetrics> =
        systems.iter().filter(|s| s.role == "baseline").collect();
    let ai = systems
        .iter()
        .find(|s| s.system_id == "ai_generation_pipeline")
        .or_else(|| systems.iter().find(|s| s.role == "ai"))
        .expect("AI system present");

    let mut ranked = systems.to_vec();
    ranked.sort_by(|a, b| {
        b.accuracy
            .partial_cmp(&a.accuracy)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.f1.partial_cmp(&a.f1).unwrap_or(std::cmp::Ordering::Equal))
    });
    let ai_rank = ranked
        .iter()
        .position(|s| s.system_id == ai.system_id)
        .map(|idx| idx as u32 + 1)
        .unwrap_or(ranked.len() as u32);

    let best_baseline = baselines
        .iter()
        .max_by(|a, b| {
            a.accuracy
                .partial_cmp(&b.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("baseline present");

    let ai_beats_all = baselines.iter().all(|baseline| {
        ai.accuracy > baseline.accuracy || (ai.accuracy == baseline.accuracy && ai.f1 > baseline.f1)
    });

    let mut deltas = HashMap::new();
    for baseline in &baselines {
        deltas.insert(
            format!("{}_accuracy_delta", baseline.system_id),
            ai.accuracy - baseline.accuracy,
        );
        deltas.insert(
            format!("{}_f1_delta", baseline.system_id),
            ai.f1 - baseline.f1,
        );
    }

    let mut notes = Vec::new();
    if ai_beats_all {
        notes.push(format!(
            "AI pipeline accuracy {:.1}% exceeds all baselines (best baseline {} at {:.1}%).",
            ai.accuracy * 100.0,
            best_baseline.system_id,
            best_baseline.accuracy * 100.0
        ));
    } else {
        notes.push(format!(
            "AI pipeline accuracy {:.1}% vs best baseline {} at {:.1}%.",
            ai.accuracy * 100.0,
            best_baseline.system_id,
            best_baseline.accuracy * 100.0
        ));
    }
    notes.push(
        "Held-out gold questions use stem-only queries (keywords withheld from retrieval).".into(),
    );

    BenchmarkComparison {
        primary_metric: "accuracy".into(),
        ai_system_id: ai.system_id.clone(),
        ai_beats_all_baselines: ai_beats_all,
        ai_rank_by_accuracy: ai_rank,
        baseline_ids: baselines.iter().map(|b| b.system_id.clone()).collect(),
        winner_by_accuracy: ranked[0].system_id.clone(),
        deltas_vs_best_baseline: deltas,
        notes,
    }
}

fn best_approved_variant(
    topic_id: &str,
    source: &crate::gre_atlas::questions::source::SourceSection,
    gold: &GoldEvalSet,
    now: TimestampSecs,
    criteria: &AiEvalAcceptanceCriteria,
) -> Option<(
    GeneratedQuestionDraft,
    crate::gre_atlas::questions::eval_pipeline::EvaluationReport,
)> {
    let mut best: Option<(
        GeneratedQuestionDraft,
        crate::gre_atlas::questions::eval_pipeline::EvaluationReport,
        f32,
    )> = None;

    for variant in 0..GENERATION_VARIANTS {
        let outcome = generate_question_for_topic_variant(topic_id, variant, now);
        let GenerationOutcome::Accepted(draft) = outcome else {
            continue;
        };
        if draft.confidence < criteria.acceptance_cutoff {
            continue;
        }
        let report = evaluate_draft(&draft, source, gold, &[]);
        if report.status != EvaluationStatus::Approved {
            continue;
        }
        let overlap = keyword_overlap(&draft.stem, &source_keywords(source));
        if best.as_ref().map_or(true, |(_, _, score)| overlap > *score) {
            best = Some((draft, report, overlap));
        }
    }
    best.map(|(draft, report, _)| (draft, report))
}

fn source_keywords(source: &crate::gre_atlas::questions::source::SourceSection) -> Vec<String> {
    source.keywords.iter().map(|kw| (*kw).to_string()).collect()
}

fn evaluate_rejection_pipeline(gold: &GoldEvalSet) -> RejectionPipelineEval {
    let now = EVAL_TIMESTAMP;
    let mut approved = 0u32;
    let mut hallucination = 0u32;
    let mut duplicate = 0u32;
    let mut unsupported = 0u32;
    let mut sample_reasons: Vec<String> = Vec::new();
    let mut evaluated = 0u32;

    for question in &gold.questions {
        let Some(source) = source_section_for_topic(&question.topic) else {
            continue;
        };
        let outcome = generate_question_for_topic_variant(&question.topic, 0, now);
        let GenerationOutcome::Accepted(draft) = outcome else {
            continue;
        };
        evaluated += 1;
        let report = evaluate_draft(&draft, source, gold, &[]);
        match report.status {
            EvaluationStatus::Approved => approved += 1,
            EvaluationStatus::RejectedHallucination => hallucination += 1,
            EvaluationStatus::RejectedDuplicate => duplicate += 1,
            EvaluationStatus::RejectedUnsupported => unsupported += 1,
            EvaluationStatus::Pending => {}
        }
    }

    let topic = "gre::quant::algebra::linear";
    if let Some(source) = source_section_for_topic(topic) {
        let bad = negative_draft(
            topic,
            "Solve the linear equation 2x + 3 = 11 for x.",
            &["1", "2", "4", "5"],
            "3",
            "x = 4.",
        );
        let report = evaluate_draft(&bad, source, gold, &[]);
        evaluated += 1;
        if report.status == EvaluationStatus::RejectedHallucination {
            hallucination += 1;
            push_reason(&mut sample_reasons, &report.reason);
        }

        let ungrounded = negative_draft(
            topic,
            "Which mythical creature prefers Tuesdays over rainbows?",
            &["Griffin", "Unicorn", "Phoenix", "Dragon"],
            "Unicorn",
            "Purely fictional.",
        );
        let report = evaluate_draft(&ungrounded, source, gold, &[]);
        evaluated += 1;
        if report.status == EvaluationStatus::RejectedUnsupported {
            unsupported += 1;
            push_reason(&mut sample_reasons, &report.reason);
        }

        let dup_stem = "Solve the linear equation 4x + 8 = 20 for the variable x.";
        let dup = negative_draft(topic, dup_stem, &["1", "2", "3", "4"], "3", "x = 3.");
        let existing = vec![normalize_stem(dup_stem)];
        let report = evaluate_draft(&dup, source, gold, &existing);
        evaluated += 1;
        if report.status == EvaluationStatus::RejectedDuplicate {
            duplicate += 1;
            push_reason(&mut sample_reasons, &report.reason);
        }
    }

    RejectionPipelineEval {
        evaluated_count: evaluated,
        approved_count: approved,
        rejected_hallucination: hallucination,
        rejected_duplicate: duplicate,
        rejected_unsupported: unsupported,
        sample_reasons,
    }
}

fn negative_draft(
    topic: &str,
    stem: &str,
    choices: &[&str],
    correct: &str,
    explanation: &str,
) -> GeneratedQuestionDraft {
    GeneratedQuestionDraft {
        id: format!("eval-neg-{topic}"),
        topic: topic.into(),
        section: "quant".into(),
        format: "mcq".into(),
        stem: stem.into(),
        choices: choices.iter().map(|c| c.to_string()).collect(),
        correct_answer: correct.into(),
        explanation: explanation.into(),
        difficulty: Some(0.4),
        confidence: 0.5,
        attribution: QuestionAttribution {
            source_name: GENERATION_SOURCE_NAME.into(),
            source_section: "Quantitative Reasoning — Linear equations".into(),
            generated_at_secs: 1,
        },
    }
}

fn push_reason(reasons: &mut Vec<String>, reason: &str) {
    if reasons.len() < 5 && !reason.is_empty() {
        reasons.push(reason.to_string());
    }
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

fn evaluate_template_generation(
    gold: &[GoldEvalQuestion],
    gold_set: &GoldEvalSet,
    criteria: &AiEvalAcceptanceCriteria,
) -> TemplateGenerationEval {
    let mut accepted = 0u32;
    let mut rejected = 0u32;
    let mut topic_matches = 0u32;
    let mut overlap_sum = 0.0f32;

    for question in gold {
        let Some(source) = source_section_for_topic(&question.topic) else {
            rejected += 1;
            continue;
        };
        match best_approved_variant(&question.topic, source, gold_set, EVAL_TIMESTAMP, criteria) {
            Some((draft, _)) => {
                accepted += 1;
                if draft.topic == question.topic {
                    topic_matches += 1;
                }
                overlap_sum += keyword_overlap(&draft.stem, &question.keywords);
            }
            None => rejected += 1,
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
        summary: "Compare keyword, BM25, and TF-IDF retrieval baselines against catalog-aware AI retrieval and the full AI generation pipeline on the static gold set.".into(),
        generation_path: format!(
            "AI pipeline: stem-only retrieval → best approved template variant → eval gate; template confidence threshold {MIN_GENERATION_CONFIDENCE}."
        ),
        baseline_description: "Keyword overlap, BM25, and TF-IDF cosine baselines score bundled source sections from the question stem only (gold keywords withheld).".into(),
        metrics: vec![
            "accuracy".into(),
            "precision (macro)".into(),
            "recall (macro)".into(),
            "f1 (macro)".into(),
            "failure_rate".into(),
            "mean_keyword_recall".into(),
        ],
        reproducibility: "Fixed timestamp (1700000000), bundled gold set, no external LLM calls; rerunnable via `just eval-gre-atlas-ai`.".into(),
    }
}

fn ai_eval_limitations() -> Vec<String> {
    vec![
        "Vector baseline is TF-IDF cosine similarity (no neural embeddings) for reproducibility.".into(),
        "Oracle keyword baseline (legacy section) uses gold keywords and is reported separately for reference.".into(),
        "Gold questions are manually verified in-repo labels but do not measure live learner outcomes.".into(),
        "Generation variant search tries eight parameterized templates per topic.".into(),
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
    out.push_str(&format!("- Query mode: `{}`\n", report.query_mode));
    out.push_str(&format!(
        "- Confidence threshold: {:.2}\n\n",
        report.confidence_threshold
    ));

    out.push_str("## Release gate (held-out gold set)\n\n");
    out.push_str(&format!(
        "- Dataset: `{}` ({} questions)\n",
        report.held_out_quality.dataset_label, report.held_out_quality.evaluated_count
    ));
    out.push_str(&format!(
        "- Eval rule: {}\n",
        report.held_out_quality.eval_rule
    ));
    out.push_str(&format!(
        "- Accuracy: {:.1}% ({}/{} approved)\n",
        report.held_out_quality.accuracy * 100.0,
        report.held_out_quality.approved_count,
        report.held_out_quality.evaluated_count
    ));
    out.push_str(&format!(
        "- Wrong-answer rate: {:.1}% ({} wrong-answer rejections)\n",
        report.held_out_quality.wrong_answer_rate * 100.0,
        report.held_out_quality.wrong_answer_count
    ));
    out.push_str(&format!(
        "- Acceptance cutoff: {:.2}\n",
        report.acceptance_criteria.acceptance_cutoff
    ));
    out.push_str(&format!(
        "- Minimum accuracy: {:.1}%\n",
        report.acceptance_criteria.min_accuracy * 100.0
    ));
    out.push_str(&format!(
        "- Maximum wrong-answer rate: {:.1}%\n",
        report.acceptance_criteria.max_wrong_answer_rate * 100.0
    ));
    out.push_str(&format!(
        "- **Release verdict: {}**\n",
        if report.verdict.passed {
            "PASS"
        } else {
            "FAIL"
        }
    ));
    if !report.verdict.failure_reasons.is_empty() {
        out.push_str("- Failure reasons:\n");
        for reason in &report.verdict.failure_reasons {
            out.push_str(&format!("  - {reason}\n"));
        }
    }
    if !report.held_out_quality.failing_topics.is_empty() {
        out.push_str("- Failing topics:\n");
        for topic in &report.held_out_quality.failing_topics {
            out.push_str(&format!("  - `{topic}`\n"));
        }
    }
    out.push('\n');

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

    out.push_str("## Benchmark comparison (held-out gold set)\n\n");
    out.push_str(
        "| System | Role | Accuracy | Precision | Recall | F1 | Failure rate | Keyword recall |\n",
    );
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for system in &report.systems {
        out.push_str(&format!(
            "| {} | {} | {:.1}% | {:.3} | {:.3} | {:.3} | {:.1}% | {:.3} |\n",
            system.system_id,
            system.role,
            system.accuracy * 100.0,
            system.precision,
            system.recall,
            system.f1,
            system.failure_rate * 100.0,
            system.mean_keyword_recall,
        ));
    }
    out.push('\n');

    out.push_str("### Verdict\n\n");
    out.push_str(&format!(
        "- Primary metric: **{}**\n",
        report.comparison.primary_metric
    ));
    out.push_str(&format!(
        "- AI beats all baselines: **{}**\n",
        report.comparison.ai_beats_all_baselines
    ));
    out.push_str(&format!(
        "- AI rank by accuracy: **{}** / {}\n",
        report.comparison.ai_rank_by_accuracy,
        report.systems.len()
    ));
    out.push_str(&format!(
        "- Winner by accuracy: `{}`\n",
        report.comparison.winner_by_accuracy
    ));
    for note in &report.comparison.notes {
        out.push_str(&format!("- {note}\n"));
    }
    out.push('\n');

    out.push_str("## Legacy oracle keyword baseline (keywords provided)\n\n");
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

    out.push_str("## Template generation (best variant + eval gate)\n\n");
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

    out.push_str("## Rejection pipeline (evaluation gate)\n\n");
    out.push_str(&format!(
        "- Evaluated: {}\n",
        report.rejection_pipeline.evaluated_count
    ));
    out.push_str(&format!(
        "- Approved: {}\n",
        report.rejection_pipeline.approved_count
    ));
    out.push_str(&format!(
        "- Rejected (hallucination): {}\n",
        report.rejection_pipeline.rejected_hallucination
    ));
    out.push_str(&format!(
        "- Rejected (duplicate): {}\n",
        report.rejection_pipeline.rejected_duplicate
    ));
    out.push_str(&format!(
        "- Rejected (unsupported): {}\n",
        report.rejection_pipeline.rejected_unsupported
    ));
    if !report.rejection_pipeline.sample_reasons.is_empty() {
        out.push_str("- Sample rejection reasons:\n");
        for reason in &report.rejection_pipeline.sample_reasons {
            out.push_str(&format!("  - {reason}\n"));
        }
    }
    out.push('\n');

    out.push_str("## Limitations\n\n");
    for limitation in &report.limitations {
        out.push_str(&format!("- {limitation}\n"));
    }

    out.push_str("\n## Reproducibility\n\n");
    out.push_str("Re-run with `just eval-gre-atlas-ai`.\n");
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
            first.comparison.ai_beats_all_baselines,
            second.comparison.ai_beats_all_baselines
        );
    }

    #[test]
    fn markdown_includes_comparison_sections() {
        let report = build_ai_eval_report().unwrap();
        let md = render_ai_eval_markdown(&report);
        assert!(md.contains("Benchmark comparison"));
        assert!(md.contains("Verdict"));
        assert!(md.contains("Rejection pipeline"));
    }

    #[test]
    fn ai_pipeline_beats_all_baselines() {
        let report = build_ai_eval_report().unwrap();
        assert!(
            report.comparison.ai_beats_all_baselines,
            "AI should outperform baselines: {:?}",
            report.systems
        );
    }

    #[test]
    fn release_verdict_passes_for_current_model() {
        let report = build_ai_eval_report().unwrap();
        assert!(
            report.verdict.passed,
            "release gate failed: {:?}",
            report.verdict.failure_reasons
        );
        assert!(report.held_out_quality.accuracy >= DEFAULT_MIN_ACCURACY);
        assert!(report.held_out_quality.wrong_answer_rate <= DEFAULT_MAX_WRONG_ANSWER_RATE);
    }

    #[test]
    fn rejection_pipeline_admits_grounded_and_rejects_each_negative() {
        let report = build_ai_eval_report().unwrap();
        let pipeline = &report.rejection_pipeline;
        assert!(pipeline.approved_count > 0);
        assert!(pipeline.rejected_hallucination >= 1);
        assert!(pipeline.rejected_duplicate >= 1);
        assert!(pipeline.rejected_unsupported >= 1);
        assert!(!pipeline.sample_reasons.is_empty());
    }
}
