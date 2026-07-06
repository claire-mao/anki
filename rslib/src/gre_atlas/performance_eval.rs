// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use serde::Serialize;

use anki_proto::brainlift::PerformanceConfusionMatrix as PerformanceConfusionMatrixProto;
use anki_proto::brainlift::PerformanceEvalMetrics as PerformanceEvalMetricsProto;
use anki_proto::brainlift::PerformanceEvalResponse;
use anki_proto::brainlift::PerformanceTopicAccuracy as PerformanceTopicAccuracyProto;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::domain::GreCatalog;
use crate::gre_atlas::eval::ConfidenceInterval;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::readiness::wilson_ci;
use crate::gre_atlas::storage::PerformanceAttemptEvalRow;
use crate::prelude::TimestampMillis;

pub(crate) const PERFORMANCE_MODEL_VERSION: &str = "performance_v1";
pub(crate) const MIN_HELD_OUT_ATTEMPTS: u32 = 5;
const PREDICTION_THRESHOLD: f32 = 0.5;
const WILSON_Z: f32 = 1.96;

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceEval {
    pub model_version: String,
    pub methodology: PerformanceMethodology,
    pub split: PerformanceSplit,
    pub train: PerformanceTrainSummary,
    pub test: PerformanceTestMetrics,
    pub sufficient_data: bool,
    pub assessment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMethodology {
    pub summary: String,
    pub train_rule: String,
    pub test_rule: String,
    pub model_description: String,
    pub prediction_rule: String,
    pub metrics_scope: String,
    pub leakage_safety: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceSplit {
    pub rule: String,
    pub minimum_test_attempts: u32,
    pub training_attempts_rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceTrainSummary {
    pub attempt_count: u32,
    pub correct_count: u32,
    pub accuracy: f32,
    pub topic_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceTopicAccuracy {
    pub topic_id: String,
    pub attempt_count: u32,
    pub correct_count: u32,
    pub accuracy: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceTestMetrics {
    pub attempt_count: u32,
    pub correct_count: u32,
    pub accuracy: f32,
    pub accuracy_ci: ConfidenceInterval,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub prediction_accuracy: f32,
    pub confusion: ConfusionMatrix,
    pub topic_accuracy: Vec<PerformanceTopicAccuracy>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfusionMatrix {
    pub true_positive: u32,
    pub false_positive: u32,
    pub true_negative: u32,
    pub false_negative: u32,
}

pub(crate) fn is_held_out_attempt(id: i64) -> bool {
    id % 5 == 0
}

pub(crate) fn compute_performance_eval(attempts: &[PerformanceAttemptEvalRow]) -> PerformanceEval {
    let methodology = performance_methodology();
    let split = performance_split_doc();

    let train: Vec<_> = attempts
        .iter()
        .filter(|row| !is_held_out_attempt(row.id))
        .collect();
    let test: Vec<_> = attempts
        .iter()
        .filter(|row| is_held_out_attempt(row.id))
        .collect();

    let train_summary = summarize_train(&train);
    let test_metrics = evaluate_test(&train, &test, &train_summary);

    let sufficient_data = test.len() as u32 >= MIN_HELD_OUT_ATTEMPTS;
    let assessment = performance_assessment(sufficient_data, test.len() as u32, &test_metrics);

    PerformanceEval {
        model_version: PERFORMANCE_MODEL_VERSION.into(),
        methodology,
        split,
        train: train_summary,
        test: test_metrics,
        sufficient_data,
        assessment,
    }
}

fn performance_split_doc() -> PerformanceSplit {
    PerformanceSplit {
        rule: "bl_performance_attempt.id % 5 == 0".into(),
        minimum_test_attempts: MIN_HELD_OUT_ATTEMPTS,
        training_attempts_rule: "bl_performance_attempt.id % 5 != 0".into(),
    }
}

fn performance_methodology() -> PerformanceMethodology {
    PerformanceMethodology {
        summary: "Practice attempts are split deterministically by row id. The performance model estimates P(correct) from training attempts only (topic-specific accuracy with a global fallback). Each held-out attempt receives a binary prediction (correct vs incorrect) using a 0.5 threshold. All reported metrics are computed on held-out attempts only.".into(),
        train_rule: "bl_performance_attempt.id % 5 != 0".into(),
        test_rule: "bl_performance_attempt.id % 5 == 0".into(),
        model_description: "Topic-stratified empirical accuracy from training attempts; unseen topics use global training accuracy.".into(),
        prediction_rule: "Predict correct when estimated P(correct) >= 0.5.".into(),
        metrics_scope: "Accuracy (observed held-out correct rate), Wilson 95% CI, precision, recall, F1, and prediction accuracy use only held-out attempts. Training attempts never appear in test metric denominators.".into(),
        leakage_safety: "Held-out membership is fixed at insert time from the auto-increment attempt id, before the answer is known to the evaluator. The eval harness reads attempts read-only and never mixes training rows into test metrics.".into(),
    }
}

fn summarize_train(train: &[&PerformanceAttemptEvalRow]) -> PerformanceTrainSummary {
    let attempt_count = train.len() as u32;
    let correct_count = train.iter().filter(|row| row.correct).count() as u32;
    let accuracy = if attempt_count > 0 {
        correct_count as f32 / attempt_count as f32
    } else {
        0.0
    };
    let topic_count = train
        .iter()
        .map(|row| row.topic.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len() as u32;
    PerformanceTrainSummary {
        attempt_count,
        correct_count,
        accuracy,
        topic_count,
    }
}

fn topic_accuracies(train: &[&PerformanceAttemptEvalRow]) -> HashMap<String, f32> {
    let mut totals: HashMap<String, (u32, u32)> = HashMap::new();
    for row in train {
        let entry = totals.entry(row.topic.clone()).or_insert((0, 0));
        entry.1 += 1;
        if row.correct {
            entry.0 += 1;
        }
    }
    totals
        .into_iter()
        .map(|(topic, (correct, total))| (topic, correct as f32 / total as f32))
        .collect()
}

fn predicted_correct(
    row: &PerformanceAttemptEvalRow,
    topic_rates: &HashMap<String, f32>,
    global_accuracy: f32,
) -> bool {
    let rate = topic_rates
        .get(&row.topic)
        .copied()
        .unwrap_or(global_accuracy);
    rate >= PREDICTION_THRESHOLD
}

fn evaluate_test(
    train: &[&PerformanceAttemptEvalRow],
    test: &[&PerformanceAttemptEvalRow],
    train_summary: &PerformanceTrainSummary,
) -> PerformanceTestMetrics {
    let topic_rates = topic_accuracies(train);
    let global_accuracy = train_summary.accuracy;

    let mut true_positive = 0u32;
    let mut false_positive = 0u32;
    let mut true_negative = 0u32;
    let mut false_negative = 0u32;
    let mut prediction_matches = 0u32;

    for row in test {
        let predicted = predicted_correct(row, &topic_rates, global_accuracy);
        let actual = row.correct;
        if predicted == actual {
            prediction_matches += 1;
        }
        match (predicted, actual) {
            (true, true) => true_positive += 1,
            (true, false) => false_positive += 1,
            (false, true) => false_negative += 1,
            (false, false) => true_negative += 1,
        }
    }

    let attempt_count = test.len() as u32;
    let correct_count = test.iter().filter(|row| row.correct).count() as u32;
    let accuracy = if attempt_count > 0 {
        correct_count as f32 / attempt_count as f32
    } else {
        0.0
    };
    let (ci_low, ci_high) = wilson_ci(correct_count, attempt_count, WILSON_Z);
    let accuracy_ci = ConfidenceInterval {
        level: 0.95,
        low: ci_low * 100.0,
        high: ci_high * 100.0,
        method: "wilson_score".into(),
    };

    let precision = divide(true_positive, true_positive + false_positive);
    let recall = divide(true_positive, true_positive + false_negative);
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let prediction_accuracy = if attempt_count > 0 {
        prediction_matches as f32 / attempt_count as f32
    } else {
        0.0
    };

    PerformanceTestMetrics {
        attempt_count,
        correct_count,
        accuracy: accuracy * 100.0,
        accuracy_ci,
        precision,
        recall,
        f1,
        prediction_accuracy,
        confusion: ConfusionMatrix {
            true_positive,
            false_positive,
            true_negative,
            false_negative,
        },
        topic_accuracy: topic_accuracy_from_test(test),
    }
}

fn topic_accuracy_from_test(test: &[&PerformanceAttemptEvalRow]) -> Vec<PerformanceTopicAccuracy> {
    let mut totals: HashMap<String, (u32, u32)> = HashMap::new();
    for row in test {
        let entry = totals.entry(row.topic.clone()).or_insert((0, 0));
        entry.1 += 1;
        if row.correct {
            entry.0 += 1;
        }
    }
    let mut rows: Vec<_> = totals
        .into_iter()
        .map(|(topic_id, (correct_count, attempt_count))| PerformanceTopicAccuracy {
            topic_id: topic_id.clone(),
            attempt_count,
            correct_count,
            accuracy: if attempt_count > 0 {
                correct_count as f32 / attempt_count as f32 * 100.0
            } else {
                0.0
            },
        })
        .collect();
    rows.sort_by(|left, right| {
        right
            .attempt_count
            .cmp(&left.attempt_count)
            .then_with(|| left.topic_id.cmp(&right.topic_id))
    });
    rows
}

impl Collection {
    pub fn gre_atlas_get_performance_eval(&mut self) -> Result<PerformanceEvalResponse> {
        let storage = gre_atlas_storage(self)?;
        let attempts = storage.list_performance_attempts_for_eval()?;
        let performance = compute_performance_eval(&attempts);
        Ok(performance_eval_to_proto(
            &performance,
            TimestampMillis::now().0,
        ))
    }
}

fn performance_eval_to_proto(
    performance: &PerformanceEval,
    computed_at_millis: i64,
) -> PerformanceEvalResponse {
    PerformanceEvalResponse {
        model_version: performance.model_version.clone(),
        sufficient_data: performance.sufficient_data,
        assessment: performance.assessment.clone(),
        test: Some(performance_test_metrics_to_proto(&performance.test)),
        computed_at_millis,
    }
}

fn performance_test_metrics_to_proto(metrics: &PerformanceTestMetrics) -> PerformanceEvalMetricsProto {
    PerformanceEvalMetricsProto {
        attempt_count: metrics.attempt_count,
        correct_count: metrics.correct_count,
        accuracy: metrics.accuracy,
        accuracy_ci: Some(confidence_interval_to_proto(&metrics.accuracy_ci)),
        confusion: Some(confusion_matrix_to_proto(&metrics.confusion)),
        topic_accuracy: metrics
            .topic_accuracy
            .iter()
            .map(topic_accuracy_to_proto)
            .collect(),
    }
}

fn confidence_interval_to_proto(
    interval: &ConfidenceInterval,
) -> anki_proto::brainlift::PerformanceConfidenceInterval {
    anki_proto::brainlift::PerformanceConfidenceInterval {
        level: interval.level,
        low: interval.low,
        high: interval.high,
        method: interval.method.clone(),
    }
}

fn confusion_matrix_to_proto(matrix: &ConfusionMatrix) -> PerformanceConfusionMatrixProto {
    PerformanceConfusionMatrixProto {
        true_positive: matrix.true_positive,
        false_positive: matrix.false_positive,
        true_negative: matrix.true_negative,
        false_negative: matrix.false_negative,
    }
}

fn topic_accuracy_to_proto(row: &PerformanceTopicAccuracy) -> PerformanceTopicAccuracyProto {
    PerformanceTopicAccuracyProto {
        topic_id: row.topic_id.clone(),
        display_name: GreCatalog::display_name_for_tag(&row.topic_id),
        attempt_count: row.attempt_count,
        correct_count: row.correct_count,
        accuracy: row.accuracy,
    }
}

fn divide(numerator: u32, denominator: u32) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn performance_assessment(
    sufficient_data: bool,
    test_count: u32,
    metrics: &PerformanceTestMetrics,
) -> String {
    if !sufficient_data {
        return format!(
            "Performance evaluation requires at least {MIN_HELD_OUT_ATTEMPTS} held-out practice attempts (current: {test_count})."
        );
    }
    format!(
        "Held-out performance on {test_count} attempts: {:.1}% correct (Wilson 95% CI [{:.1}%, {:.1}%]). \
         Model precision {:.3}, recall {:.3}, F1 {:.3} (positive class: correct answer).",
        metrics.accuracy,
        metrics.accuracy_ci.low,
        metrics.accuracy_ci.high,
        metrics.precision,
        metrics.recall,
        metrics.f1,
    )
}

pub(crate) fn render_performance_eval_document(
    performance: &PerformanceEval,
    generated_at_millis: i64,
) -> String {
    let mut out = String::new();
    out.push_str("# GRE Atlas performance model — held-out evaluation\n\n");
    out.push_str(&format!(
        "- Generated at (UTC millis): {generated_at_millis}\n"
    ));
    out.push_str(&format!(
        "- Model version: `{}`\n\n",
        performance.model_version
    ));

    out.push_str("## Train/test split\n\n");
    out.push_str(&format!("- Test rule: `{}`\n", performance.split.rule));
    out.push_str(&format!(
        "- Training rule: `{}`\n",
        performance.split.training_attempts_rule
    ));
    out.push_str(&format!(
        "- Minimum test attempts: {}\n\n",
        performance.split.minimum_test_attempts
    ));

    append_performance_eval_body(&mut out, performance);

    out.push_str("## Reproducibility\n\n");
    out.push_str(
        "Re-run with `just eval-gre_atlas /path/to/collection.anki2`. \
         This report is read-only and does not record practice attempts or mutate `greatlas.db`.\n",
    );
    out
}

pub(crate) fn render_performance_markdown(performance: &PerformanceEval) -> String {
    let mut out = String::new();
    out.push_str("## Performance model (held-out)\n\n");
    append_performance_eval_body(&mut out, performance);
    out
}

fn append_performance_eval_body(out: &mut String, performance: &PerformanceEval) {
    out.push_str(&format!(
        "- Sufficient data: {}\n",
        performance.sufficient_data
    ));
    out.push_str(&format!("- Assessment: {}\n\n", performance.assessment));

    out.push_str("### Methodology\n\n");
    out.push_str(&format!("{}\n\n", performance.methodology.summary));
    out.push_str(&format!(
        "- Train rule: `{}`\n",
        performance.methodology.train_rule
    ));
    out.push_str(&format!(
        "- Test rule: `{}`\n",
        performance.methodology.test_rule
    ));
    out.push_str(&format!(
        "- Model: {}\n",
        performance.methodology.model_description
    ));
    out.push_str(&format!(
        "- Prediction: {}\n",
        performance.methodology.prediction_rule
    ));
    out.push_str(&format!(
        "- Metrics scope: {}\n",
        performance.methodology.metrics_scope
    ));
    out.push_str(&format!(
        "- Leakage safety: {}\n\n",
        performance.methodology.leakage_safety
    ));

    out.push_str("### Train split\n\n");
    out.push_str(&format!(
        "- Attempts: {} ({} correct, {:.1}% accuracy)\n",
        performance.train.attempt_count,
        performance.train.correct_count,
        performance.train.accuracy * 100.0
    ));
    out.push_str(&format!(
        "- Topics with training data: {}\n\n",
        performance.train.topic_count
    ));

    out.push_str("### Test split (metrics computed here only)\n\n");
    out.push_str(&format!(
        "- Attempts: {} ({} correct)\n",
        performance.test.attempt_count, performance.test.correct_count
    ));
    out.push_str(&format!(
        "- Held-out accuracy: {:.1}% (Wilson 95% CI [{:.1}%, {:.1}%])\n",
        performance.test.accuracy,
        performance.test.accuracy_ci.low,
        performance.test.accuracy_ci.high
    ));
    out.push_str(&format!("- Precision: {:.3}\n", performance.test.precision));
    out.push_str(&format!("- Recall: {:.3}\n", performance.test.recall));
    out.push_str(&format!("- F1: {:.3}\n", performance.test.f1));
    out.push_str(&format!(
        "- Prediction accuracy: {:.1}%\n\n",
        performance.test.prediction_accuracy * 100.0
    ));

    out.push_str("### Confusion matrix (positive class = correct answer)\n\n");
    out.push_str("|  | Predicted correct | Predicted incorrect |\n");
    out.push_str("| --- | ---: | ---: |\n");
    out.push_str(&format!(
        "| Actual correct | {} (TP) | {} (FN) |\n",
        performance.test.confusion.true_positive, performance.test.confusion.false_negative
    ));
    out.push_str(&format!(
        "| Actual incorrect | {} (FP) | {} (TN) |\n\n",
        performance.test.confusion.false_positive, performance.test.confusion.true_negative
    ));

    if !performance.test.topic_accuracy.is_empty() {
        out.push_str("### Accuracy by topic (held-out only)\n\n");
        out.push_str("| Topic | Attempts | Correct | Accuracy |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for row in &performance.test.topic_accuracy {
            out.push_str(&format!(
                "| {} | {} | {} | {:.1}% |\n",
                GreCatalog::display_name_for_tag(&row.topic_id),
                row.attempt_count,
                row.correct_count,
                row.accuracy,
            ));
        }
        out.push('\n');
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn attempt(id: i64, topic: &str, correct: bool) -> PerformanceAttemptEvalRow {
        PerformanceAttemptEvalRow {
            id,
            topic: topic.into(),
            correct,
        }
    }

    #[test]
    fn held_out_filter_matches_readiness_pattern() {
        assert!(is_held_out_attempt(5));
        assert!(!is_held_out_attempt(1));
    }

    #[test]
    fn test_metrics_never_include_training_rows() {
        let attempts = vec![
            attempt(1, "gre::quant::algebra", true),
            attempt(2, "gre::quant::algebra", true),
            attempt(3, "gre::quant::algebra", false),
            attempt(4, "gre::quant::algebra", true),
            attempt(5, "gre::quant::algebra", false),
            attempt(10, "gre::quant::algebra", true),
            attempt(15, "gre::quant::algebra", true),
            attempt(20, "gre::quant::algebra", false),
            attempt(25, "gre::quant::algebra", true),
            attempt(30, "gre::quant::algebra", true),
        ];
        let eval = compute_performance_eval(&attempts);
        assert!(eval.sufficient_data);
        assert_eq!(eval.test.attempt_count, 6);
        assert_eq!(eval.train.attempt_count, 4);
        assert!((eval.test.accuracy - (4.0 / 6.0 * 100.0)).abs() < 0.01);
    }

    #[test]
    fn evaluation_is_deterministic() {
        let attempts = (1..=25)
            .map(|id| attempt(id, "gre::quant", id % 3 != 0))
            .collect::<Vec<_>>();
        let first = compute_performance_eval(&attempts);
        let second = compute_performance_eval(&attempts);
        assert_eq!(first.test.f1, second.test.f1);
        assert_eq!(first.test.accuracy, second.test.accuracy);
    }

    #[test]
    fn insufficient_test_attempts_is_reported() {
        let attempts = vec![attempt(5, "gre::quant", true)];
        let eval = compute_performance_eval(&attempts);
        assert!(!eval.sufficient_data);
        assert!(eval.assessment.contains("requires at least"));
    }

    #[test]
    fn precision_recall_f1_are_well_defined_on_mixed_test_set() {
        let attempts = (1..=30)
            .map(|id| attempt(id, "gre::verbal", id % 2 == 0))
            .collect::<Vec<_>>();
        let eval = compute_performance_eval(&attempts);
        assert!(eval.sufficient_data);
        assert!(eval.test.precision >= 0.0 && eval.test.precision <= 1.0);
        assert!(eval.test.recall >= 0.0 && eval.test.recall <= 1.0);
        assert!(eval.test.f1 >= 0.0 && eval.test.f1 <= 1.0);
    }

    #[test]
    fn wilson_ci_is_computed_on_held_out_accuracy() {
        let attempts = (1..=25)
            .map(|id| attempt(id, "gre::quant", id % 2 == 0))
            .collect::<Vec<_>>();
        let eval = compute_performance_eval(&attempts);
        assert!(eval.sufficient_data);
        assert!(eval.test.accuracy_ci.low <= eval.test.accuracy);
        assert!(eval.test.accuracy_ci.high >= eval.test.accuracy);
        assert_eq!(eval.test.accuracy_ci.method, "wilson_score");
    }

    #[test]
    fn confusion_matrix_counts_sum_to_test_attempts() {
        let attempts = (1..=20)
            .map(|id| attempt(id, "gre::quant::algebra", id % 3 != 0))
            .collect::<Vec<_>>();
        let eval = compute_performance_eval(&attempts);
        let cm = &eval.test.confusion;
        let total = cm.true_positive + cm.false_positive + cm.true_negative + cm.false_negative;
        assert_eq!(total, eval.test.attempt_count);
    }

    #[test]
    fn topic_accuracy_is_computed_from_held_out_attempts_only() {
        let attempts = vec![
            attempt(1, "gre::quant::algebra", true),
            attempt(2, "gre::quant::algebra", true),
            attempt(3, "gre::verbal::text_completion", false),
            attempt(4, "gre::verbal::text_completion", true),
            attempt(5, "gre::quant::algebra", false),
            attempt(10, "gre::quant::algebra", true),
            attempt(15, "gre::verbal::text_completion", true),
            attempt(20, "gre::quant::algebra", false),
            attempt(25, "gre::verbal::text_completion", true),
            attempt(30, "gre::quant::algebra", true),
        ];
        let eval = compute_performance_eval(&attempts);
        assert!(eval.sufficient_data);
        assert_eq!(eval.test.topic_accuracy.len(), 2);
        let algebra = eval
            .test
            .topic_accuracy
            .iter()
            .find(|row| row.topic_id == "gre::quant::algebra")
            .unwrap();
        assert_eq!(algebra.attempt_count, 4);
        assert_eq!(algebra.correct_count, 2);
    }

    #[test]
    fn standalone_markdown_documents_split_and_metrics() {
        let attempts = (1..=25)
            .map(|id| attempt(id, "gre::quant", id % 2 == 0))
            .collect::<Vec<_>>();
        let eval = compute_performance_eval(&attempts);
        let md = render_performance_eval_document(&eval, 1_700_000_000);
        assert!(md.contains("# GRE Atlas performance model"));
        assert!(md.contains("bl_performance_attempt.id % 5 == 0"));
        assert!(md.contains("Wilson 95% CI"));
        assert!(md.contains("Confusion matrix"));
        assert!(md.contains("Precision:"));
        assert!(md.contains("Recall:"));
        assert!(md.contains("F1:"));
    }
}
