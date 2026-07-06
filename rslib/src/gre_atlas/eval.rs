// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::AbstentionRequirement;
use anki_proto::stats::TopicMasteryRequest;
use serde::Serialize;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::ablation_eval::compute_ablation_eval;
use crate::gre_atlas::ablation_eval::render_ablation_markdown;
use crate::gre_atlas::ablation_eval::AblationEval;
use crate::gre_atlas::abstention::readiness_requirements;
use crate::gre_atlas::calibration::compute_calibration_stats;
use crate::gre_atlas::calibration::is_held_out_prediction;
use crate::gre_atlas::calibration::CALIBRATION_BIN_COUNT;
use crate::gre_atlas::calibration::MIN_CALIBRATION_HELD_OUT;
use crate::gre_atlas::calibration::MIN_OUTCOME_AGE_SECS;
use crate::gre_atlas::calibration::MIN_OUTCOME_PRACTICE_ATTEMPTS;
use crate::gre_atlas::calibration::READINESS_MODEL_VERSION;
use crate::gre_atlas::compute_coverage;
use crate::gre_atlas::coverage_report::section_ui_label;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::gre_deck_search;
use crate::gre_atlas::memory_eval::compute_memory_eval;
use crate::gre_atlas::memory_eval::render_memory_markdown;
use crate::gre_atlas::memory_eval::MemoryEval;
use crate::gre_atlas::memory_eval::MEMORY_MODEL_VERSION;
use crate::gre_atlas::performance_eval::compute_performance_eval;
use crate::gre_atlas::performance_eval::render_performance_eval_document;
use crate::gre_atlas::performance_eval::render_performance_markdown;
use crate::gre_atlas::performance_eval::PerformanceEval;
use crate::gre_atlas::performance_eval::PERFORMANCE_MODEL_VERSION;
use crate::gre_atlas::readiness::compute_memory_score;
use crate::gre_atlas::readiness::compute_performance_score;
use crate::gre_atlas::readiness::compute_readiness_score;
use crate::gre_atlas::readiness::MemoryInputs;
use crate::gre_atlas::readiness::PerformanceInputs;
use crate::gre_atlas::storage::ReadinessPredictionRow;
use crate::gre_atlas::TOPIC_TAG_PREFIX;

const CONFIDENCE_Z: f32 = 1.96;
const MILLIS_PER_SEC: i64 = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct GreAtlasEvalReport {
    pub generated_at_millis: i64,
    pub model_version: String,
    pub held_out_split: HeldOutSplitDoc,
    pub calibration: CalibrationEval,
    pub memory: MemoryEval,
    pub performance: PerformanceEval,
    pub abstention: AbstentionEval,
    pub coverage: CoverageEval,
    pub prediction_distribution: PredictionDistributionEval,
    pub ablation: AblationEval,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PredictionDistributionEval {
    pub total_predictions: u32,
    pub resolved_outcomes: u32,
    pub pending_outcomes: u32,
    pub mean_projected_score: Option<f32>,
    pub min_projected_score: Option<f32>,
    pub max_projected_score: Option<f32>,
    pub score_bins: Vec<PredictionScoreBin>,
    pub confidence_levels: Vec<ConfidenceLevelCount>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PredictionScoreBin {
    pub bin_low: f32,
    pub bin_high: f32,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfidenceLevelCount {
    pub confidence_level: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfidenceInterval {
    pub level: f32,
    pub low: f32,
    pub high: f32,
    pub method: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeldOutSplitDoc {
    pub rule: String,
    pub minimum_held_out_predictions: u32,
    pub outcome_resolution: OutcomeResolutionDoc,
    pub training_rows_rule: String,
    pub leakage_safety: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutcomeResolutionDoc {
    pub min_age_secs: i64,
    pub min_practice_attempts: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationEval {
    pub total_predictions: u32,
    pub resolved_outcomes: u32,
    pub held_out_count: u32,
    pub sufficient_data: bool,
    pub well_calibrated: bool,
    pub assessment: String,
    pub brier_score: Option<f32>,
    pub brier_score_ci: Option<ConfidenceInterval>,
    pub mean_absolute_error: Option<f32>,
    pub mean_absolute_error_ci: Option<ConfidenceInterval>,
    pub bins: Vec<CalibrationBinEval>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationBinEval {
    pub bin_low: f32,
    pub bin_high: f32,
    pub predicted_mean: f32,
    pub outcome_mean: f32,
    pub outcome_ci: ConfidenceInterval,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AbstentionEval {
    pub abstention_rate: f32,
    pub memory_abstaining: bool,
    pub performance_abstaining: bool,
    pub readiness_abstaining: bool,
    pub unmet_requirements: Vec<AbstentionRequirementDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AbstentionRequirementDoc {
    pub id: String,
    pub label: String,
    pub status: String,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageEval {
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    pub unweighted_ratio: f32,
    pub weighted_ratio: f32,
    pub sections: Vec<SectionCoverageEval>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionCoverageEval {
    pub section: String,
    pub section_weight: f32,
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    pub leaf_coverage_ratio: f32,
    pub weighted_contribution: f32,
    pub covered_exam_weight: f32,
}

impl Collection {
    pub fn gre_atlas_generate_eval_report(&mut self) -> Result<(String, String, String)> {
        let report = build_eval_report(self)?;
        let json = serde_json::to_string_pretty(&report)?;
        let markdown = render_markdown(&report);
        let performance_markdown =
            render_performance_eval_document(&report.performance, report.generated_at_millis);
        Ok((json, markdown, performance_markdown))
    }
}

fn build_eval_report(col: &mut Collection) -> Result<GreAtlasEvalReport> {
    let mastery = col.compute_topic_mastery(TopicMasteryRequest {
        search: gre_deck_search(),
        topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
        mastery_threshold: None,
        min_reviews: 1,
    })?;
    let summary = mastery.summary.clone().unwrap_or_default();

    let studied_tags: Vec<String> = mastery
        .topics
        .iter()
        .filter(|topic| topic.studied_cards > 0)
        .map(|topic| topic.topic_id.clone())
        .collect();
    let studied_tag_refs: Vec<&str> = studied_tags.iter().map(String::as_str).collect();
    let coverage = compute_coverage(&studied_tag_refs);

    let memory_score = compute_memory_score(&MemoryInputs {
        fsrs_enabled: mastery.fsrs_enabled,
        overall_retrievability: summary.overall_avg_retrievability,
        coverage_ratio: summary.coverage_ratio,
        studied_cards: summary.studied_cards,
        topics: &mastery.topics,
    });
    let memory = compute_memory_eval(col)?;

    let storage = gre_atlas_storage(col)?;
    let (correct, total) = storage.performance_summary()?;
    let performance_score = compute_performance_score(&PerformanceInputs { correct, total });

    let rows = storage.list_readiness_predictions()?;
    let report_timestamp = deterministic_report_timestamp(&rows);
    let readiness = compute_readiness_score(&memory_score, &performance_score, report_timestamp);
    let calibration = calibration_eval(&rows);
    let attempts = storage.list_performance_attempts_for_eval()?;
    let performance = compute_performance_eval(&attempts);
    let abstention = abstention_eval(&memory_score, &performance_score, &readiness);
    let coverage = coverage_eval(&coverage);
    let prediction_distribution = prediction_distribution_eval(&rows);
    let ablation = compute_ablation_eval(col)?;

    Ok(GreAtlasEvalReport {
        generated_at_millis: report_timestamp,
        model_version: READINESS_MODEL_VERSION.into(),
        held_out_split: held_out_split_doc(),
        calibration,
        memory,
        performance,
        abstention,
        coverage,
        prediction_distribution,
        ablation,
    })
}

fn deterministic_report_timestamp(rows: &[ReadinessPredictionRow]) -> i64 {
    rows.iter()
        .map(|row| row.predicted_at_secs.0)
        .max()
        .unwrap_or(0)
        * MILLIS_PER_SEC
}

fn held_out_split_doc() -> HeldOutSplitDoc {
    HeldOutSplitDoc {
        rule: "bl_readiness_prediction.id % 5 == 0".into(),
        minimum_held_out_predictions: MIN_CALIBRATION_HELD_OUT,
        outcome_resolution: OutcomeResolutionDoc {
            min_age_secs: MIN_OUTCOME_AGE_SECS,
            min_practice_attempts: MIN_OUTCOME_PRACTICE_ATTEMPTS,
            description: "An outcome is recorded when the prediction is at least three days old or the learner has at least three practice attempts after the prediction timestamp.".into(),
        },
        training_rows_rule: "bl_readiness_prediction.id % 5 != 0".into(),
        leakage_safety: "Held-out membership is fixed when a prediction row is inserted, using only the auto-increment row id. Outcomes are observed later and never influence held-out selection. This evaluation reads prediction rows read-only and does not record new predictions or resolve pending outcomes, so metrics are reproducible for a fixed greatlas.db snapshot.".into(),
    }
}

fn calibration_eval(rows: &[ReadinessPredictionRow]) -> CalibrationEval {
    let stats = compute_calibration_stats(rows);
    let held_out: Vec<_> = rows
        .iter()
        .filter(|row| row.outcome_score.is_some() && is_held_out_prediction(row.id))
        .collect();

    let brier_score_ci = stats.brier_score.and_then(|brier| {
        let squared_errors: Vec<f32> = held_out
            .iter()
            .map(|row| {
                let predicted = row.projected_score / 100.0;
                let outcome = row.outcome_score.unwrap() / 100.0;
                (predicted - outcome).powi(2)
            })
            .collect();
        normal_mean_ci(brier, &squared_errors)
    });

    let mean_absolute_error_ci = stats.mean_absolute_error.and_then(|mae| {
        let abs_errors: Vec<f32> = held_out
            .iter()
            .map(|row| (row.projected_score - row.outcome_score.unwrap()).abs())
            .collect();
        normal_mean_ci(mae, &abs_errors)
    });

    let bins = stats
        .calibration_curve
        .iter()
        .map(|bin| {
            let outcomes: Vec<f32> = held_out
                .iter()
                .filter(|row| {
                    row.projected_score >= bin.bin_low && row.projected_score < bin.bin_high
                })
                .map(|row| row.outcome_score.unwrap())
                .collect();
            CalibrationBinEval {
                bin_low: bin.bin_low,
                bin_high: bin.bin_high,
                predicted_mean: bin.predicted_mean,
                outcome_mean: bin.outcome_mean,
                outcome_ci: normal_mean_ci(bin.outcome_mean, &outcomes)
                    .unwrap_or_else(|| confidence_interval(0.0, 0.0, "normal_approximation_empty")),
                count: bin.count,
            }
        })
        .collect();

    CalibrationEval {
        total_predictions: stats.total_predictions,
        resolved_outcomes: stats.resolved_outcomes,
        held_out_count: stats.held_out_count,
        sufficient_data: stats.sufficient_data,
        well_calibrated: stats.well_calibrated,
        assessment: stats.assessment,
        brier_score: stats.brier_score,
        brier_score_ci,
        mean_absolute_error: stats.mean_absolute_error,
        mean_absolute_error_ci,
        bins,
    }
}

fn abstention_eval(
    memory: &anki_proto::brainlift::MemoryScore,
    performance: &anki_proto::brainlift::PerformanceScore,
    readiness: &anki_proto::brainlift::ReadinessScore,
) -> AbstentionEval {
    let memory_abstaining = !memory.sufficient_data;
    let performance_abstaining = !performance.sufficient_data;
    let readiness_abstaining = !readiness.sufficient_data;
    let abstaining_components = [
        memory_abstaining,
        performance_abstaining,
        readiness_abstaining,
    ];
    let abstention_rate = abstaining_components
        .iter()
        .filter(|&&abstain| abstain)
        .count() as f32
        / 3.0;

    let requirements = readiness_requirements(
        &memory.abstention_requirements,
        &performance.abstention_requirements,
    );
    let unmet_requirements = requirements
        .into_iter()
        .filter(|req| !req.met)
        .map(abstention_requirement_doc)
        .collect();

    AbstentionEval {
        abstention_rate,
        memory_abstaining,
        performance_abstaining,
        readiness_abstaining,
        unmet_requirements,
    }
}

fn abstention_requirement_doc(req: AbstentionRequirement) -> AbstentionRequirementDoc {
    AbstentionRequirementDoc {
        id: req.id,
        label: req.label,
        status: req.status,
        met: req.met,
    }
}

fn prediction_distribution_eval(rows: &[ReadinessPredictionRow]) -> PredictionDistributionEval {
    let total_predictions = rows.len() as u32;
    let resolved_outcomes = rows
        .iter()
        .filter(|row| row.outcome_score.is_some())
        .count() as u32;
    let pending_outcomes = total_predictions.saturating_sub(resolved_outcomes);

    let projected_scores: Vec<f32> = rows.iter().map(|row| row.projected_score).collect();
    let (mean_projected_score, min_projected_score, max_projected_score) =
        if projected_scores.is_empty() {
            (None, None, None)
        } else {
            let sum = projected_scores.iter().sum::<f32>();
            let mean = sum / projected_scores.len() as f32;
            let min = projected_scores
                .iter()
                .copied()
                .fold(f32::INFINITY, f32::min);
            let max = projected_scores
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            (Some(mean), Some(min), Some(max))
        };

    let mut score_bins = Vec::new();
    for bin in 0..CALIBRATION_BIN_COUNT {
        let bin_low = bin as f32 * 10.0;
        let bin_high = bin_low + 10.0;
        let count = rows
            .iter()
            .filter(|row| row.projected_score >= bin_low && row.projected_score < bin_high)
            .count() as u32;
        if count > 0 {
            score_bins.push(PredictionScoreBin {
                bin_low,
                bin_high,
                count,
            });
        }
    }

    let mut confidence_counts: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    for row in rows {
        *confidence_counts
            .entry(row.confidence_level.clone())
            .or_default() += 1;
    }
    let confidence_levels = confidence_counts
        .into_iter()
        .map(|(confidence_level, count)| ConfidenceLevelCount {
            confidence_level,
            count,
        })
        .collect();

    PredictionDistributionEval {
        total_predictions,
        resolved_outcomes,
        pending_outcomes,
        mean_projected_score,
        min_projected_score,
        max_projected_score,
        score_bins,
        confidence_levels,
    }
}

fn coverage_eval(coverage: &crate::gre_atlas::GreCoverage) -> CoverageEval {
    CoverageEval {
        catalog_leaf_count: coverage.catalog_leaf_count,
        covered_leaf_count: coverage.covered_leaf_count,
        unweighted_ratio: coverage.unweighted_ratio,
        weighted_ratio: coverage.weighted_ratio,
        sections: coverage
            .sections
            .iter()
            .map(|section| SectionCoverageEval {
                section: section_ui_label(section.section).into(),
                section_weight: section.section_weight,
                catalog_leaf_count: section.catalog_leaf_count,
                covered_leaf_count: section.covered_leaf_count,
                leaf_coverage_ratio: section.leaf_coverage_ratio,
                weighted_contribution: section.weighted_contribution,
                covered_exam_weight: section.covered_exam_weight,
            })
            .collect(),
    }
}

fn normal_mean_ci(mean: f32, values: &[f32]) -> Option<ConfidenceInterval> {
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return Some(confidence_interval(
            mean,
            mean,
            "normal_approximation_degenerate",
        ));
    }
    let n = values.len() as f32;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / (n - 1.0);
    let margin = CONFIDENCE_Z * (variance / n).sqrt();
    Some(confidence_interval(
        (mean - margin).max(0.0),
        mean + margin,
        "normal_approximation",
    ))
}

fn confidence_interval(low: f32, high: f32, method: &str) -> ConfidenceInterval {
    ConfidenceInterval {
        level: 0.95,
        low,
        high,
        method: method.into(),
    }
}

fn render_markdown(report: &GreAtlasEvalReport) -> String {
    let mut out = String::new();
    out.push_str("# GRE Atlas evaluation report\n\n");
    out.push_str(&format!(
        "- Report timestamp (UTC millis, latest prediction): {}\n",
        report.generated_at_millis
    ));
    out.push_str(&format!("- Model version: `{}`\n\n", report.model_version));

    out.push_str("## Held-out split\n\n");
    out.push_str(&format!("- Rule: `{}`\n", report.held_out_split.rule));
    out.push_str(&format!(
        "- Minimum held-out predictions: {}\n",
        report.held_out_split.minimum_held_out_predictions
    ));
    out.push_str(&format!(
        "- Training rows: `{}`\n",
        report.held_out_split.training_rows_rule
    ));
    out.push_str(&format!(
        "- Outcome resolution: {}\n",
        report.held_out_split.outcome_resolution.description
    ));
    out.push_str(&format!(
        "- Leakage safety: {}\n\n",
        report.held_out_split.leakage_safety
    ));

    out.push_str("## Readiness calibration (held-out)\n\n");
    out.push_str(&format!(
        "- Total predictions: {}\n",
        report.calibration.total_predictions
    ));
    out.push_str(&format!(
        "- Resolved outcomes: {}\n",
        report.calibration.resolved_outcomes
    ));
    out.push_str(&format!(
        "- Held-out count: {}\n",
        report.calibration.held_out_count
    ));
    out.push_str(&format!(
        "- Sufficient data: {}\n",
        report.calibration.sufficient_data
    ));
    if let Some(brier) = report.calibration.brier_score {
        out.push_str(&format!("- Brier score: {brier:.4}\n"));
        if let Some(ci) = &report.calibration.brier_score_ci {
            out.push_str(&format!(
                "- Brier 95% CI: [{:.4}, {:.4}] ({}) \n",
                ci.low, ci.high, ci.method
            ));
        }
    } else {
        out.push_str("- Brier score: n/a\n");
    }
    if let Some(mae) = report.calibration.mean_absolute_error {
        out.push_str(&format!("- Mean absolute error: {mae:.2}\n"));
        if let Some(ci) = &report.calibration.mean_absolute_error_ci {
            out.push_str(&format!(
                "- MAE 95% CI: [{:.2}, {:.2}] ({}) \n",
                ci.low, ci.high, ci.method
            ));
        }
    }
    out.push_str(&format!(
        "- Well calibrated: {}\n",
        report.calibration.well_calibrated
    ));
    out.push_str(&format!(
        "- Assessment: {}\n\n",
        report.calibration.assessment
    ));

    out.push_str("### Calibration bins\n\n");
    if report.calibration.bins.is_empty() {
        out.push_str("_No calibration bins (insufficient held-out data)._\n\n");
    } else {
        out.push_str("| Bin | Count | Predicted mean | Outcome mean | Outcome 95% CI |\n");
        out.push_str("| --- | ---: | ---: | ---: | --- |\n");
        for bin in &report.calibration.bins {
            out.push_str(&format!(
                "| {:.0}–{:.0} | {} | {:.1} | {:.1} | [{:.1}, {:.1}] |\n",
                bin.bin_low,
                bin.bin_high,
                bin.count,
                bin.predicted_mean,
                bin.outcome_mean,
                bin.outcome_ci.low,
                bin.outcome_ci.high
            ));
        }
        out.push('\n');
    }

    out.push_str(&render_memory_markdown(&report.memory));
    out.push_str(&render_performance_markdown(&report.performance));

    out.push_str("## Abstention\n\n");
    out.push_str(&format!(
        "- Abstention rate (memory/performance/readiness): {:.1}%\n",
        report.abstention.abstention_rate * 100.0
    ));
    out.push_str(&format!(
        "- Memory abstaining: {}\n",
        report.abstention.memory_abstaining
    ));
    out.push_str(&format!(
        "- Performance abstaining: {}\n",
        report.abstention.performance_abstaining
    ));
    out.push_str(&format!(
        "- Readiness abstaining: {}\n",
        report.abstention.readiness_abstaining
    ));
    if report.abstention.unmet_requirements.is_empty() {
        out.push_str("- Unmet requirements: none\n\n");
    } else {
        out.push_str("- Unmet requirements:\n");
        for req in &report.abstention.unmet_requirements {
            out.push_str(&format!(
                "  - **{}**: {} ({})\n",
                req.label, req.status, req.id
            ));
        }
        out.push('\n');
    }

    out.push_str("## Coverage\n\n");
    out.push_str(&format!(
        "- Catalog leaves: {} covered / {} total\n",
        report.coverage.covered_leaf_count, report.coverage.catalog_leaf_count
    ));
    out.push_str(&format!(
        "- Unweighted ratio: {:.1}%\n",
        report.coverage.unweighted_ratio * 100.0
    ));
    out.push_str(&format!(
        "- Weighted ratio: {:.1}%\n\n",
        report.coverage.weighted_ratio * 100.0
    ));
    out.push_str(
        "| Section | Leaves covered | Leaf ratio | Section weight | Weighted contribution |\n",
    );
    out.push_str("| --- | ---: | ---: | ---: | ---: |\n");
    for section in &report.coverage.sections {
        out.push_str(&format!(
            "| {} | {}/{} | {:.1}% | {:.2} | {:.3} |\n",
            section.section,
            section.covered_leaf_count,
            section.catalog_leaf_count,
            section.leaf_coverage_ratio * 100.0,
            section.section_weight,
            section.weighted_contribution
        ));
    }

    out.push_str("\n## Prediction distribution\n\n");
    out.push_str(&format!(
        "- Total predictions: {}\n",
        report.prediction_distribution.total_predictions
    ));
    out.push_str(&format!(
        "- Resolved outcomes: {}\n",
        report.prediction_distribution.resolved_outcomes
    ));
    out.push_str(&format!(
        "- Pending outcomes: {}\n",
        report.prediction_distribution.pending_outcomes
    ));
    if let Some(mean) = report.prediction_distribution.mean_projected_score {
        out.push_str(&format!("- Mean projected score: {mean:.1}\n"));
        out.push_str(&format!(
            "- Min projected score: {:.1}\n",
            report
                .prediction_distribution
                .min_projected_score
                .unwrap_or(mean)
        ));
        out.push_str(&format!(
            "- Max projected score: {:.1}\n",
            report
                .prediction_distribution
                .max_projected_score
                .unwrap_or(mean)
        ));
    } else {
        out.push_str("- Mean projected score: n/a\n");
    }
    out.push_str("\n### Score bins (all predictions)\n\n");
    if report.prediction_distribution.score_bins.is_empty() {
        out.push_str("_No prediction rows in greatlas.db._\n\n");
    } else {
        out.push_str("| Bin | Count |\n");
        out.push_str("| --- | ---: |\n");
        for bin in &report.prediction_distribution.score_bins {
            out.push_str(&format!(
                "| {:.0}–{:.0} | {} |\n",
                bin.bin_low, bin.bin_high, bin.count
            ));
        }
        out.push('\n');
    }
    if !report.prediction_distribution.confidence_levels.is_empty() {
        out.push_str("### Confidence levels\n\n");
        out.push_str("| Level | Count |\n");
        out.push_str("| --- | ---: |\n");
        for level in &report.prediction_distribution.confidence_levels {
            out.push_str(&format!(
                "| {} | {} |\n",
                level.confidence_level, level.count
            ));
        }
        out.push('\n');
    }

    out.push_str(&render_ablation_markdown(&report.ablation));

    out.push_str("\n## Reproducibility\n\n");
    out.push_str(&format!(
        "Re-run with `just eval-gre_atlas /path/to/collection.anki2`. \
         Readiness, memory, and performance evaluation are read-only. \
         Readiness model: `{}`; memory model: `{}`; performance model: `{}`; ablation model: `{}`.\n",
        READINESS_MODEL_VERSION, MEMORY_MODEL_VERSION, PERFORMANCE_MODEL_VERSION,
        crate::gre_atlas::ablation_eval::ABLATION_MODEL_VERSION
    ));
    out
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::Collection;
    use crate::gre_atlas::ablation_eval::ablation_methodology;
    use crate::gre_atlas::ablation_eval::insufficient_scenario;
    use crate::gre_atlas::ablation_eval::synthetic_reference_scenario;
    use crate::gre_atlas::ablation_eval::AblationEval;
    use crate::gre_atlas::ablation_eval::ABLATION_MODEL_VERSION;
    use crate::gre_atlas::calibration::READINESS_MODEL_VERSION;
    use crate::gre_atlas::storage::ReadinessPredictionRow;
    use crate::gre_atlas::study_plan::DAILY_FOCUS_TOPIC_COUNT;
    use crate::timestamp::TimestampSecs;

    fn prediction(id: i64, projected: f32, outcome: Option<f32>) -> ReadinessPredictionRow {
        ReadinessPredictionRow {
            id,
            predicted_at_secs: TimestampSecs(1_700_000_000),
            projected_score: projected,
            projected_score_low: Some(projected - 5.0),
            projected_score_high: Some(projected + 5.0),
            memory_score: 70.0,
            performance_score: 70.0,
            coverage_ratio: 0.5,
            confidence_level: "medium".into(),
            model_version: READINESS_MODEL_VERSION.into(),
            outcome_score: outcome,
            outcome_observed_at_secs: outcome.map(|_| TimestampSecs(1_700_100_000)),
            outcome_memory_score: outcome.map(|_| 68.0),
            outcome_performance_score: outcome.map(|_| 65.0),
            practice_correct: outcome.map(|_| 3),
            practice_total: outcome.map(|_| 5),
        }
    }

    #[test]
    fn eval_report_is_deterministic_for_fixed_rows() {
        let rows: Vec<_> = (0..10)
            .map(|idx| {
                let id = (idx + 1) * 5;
                prediction(id, 70.0 + idx as f32, Some(68.0 + idx as f32))
            })
            .collect();
        let first = calibration_eval(&rows);
        let second = calibration_eval(&rows);
        assert_eq!(first.brier_score, second.brier_score);
        assert_eq!(first.bins.len(), second.bins.len());
        assert!(first.sufficient_data);
        assert!(first.brier_score.unwrap() >= 0.0);
    }

    #[test]
    fn prediction_distribution_bins_all_predictions() {
        let rows = vec![
            prediction(1, 55.0, None),
            prediction(2, 65.0, Some(60.0)),
            prediction(5, 75.0, Some(70.0)),
            prediction(6, 85.0, None),
        ];
        let first = prediction_distribution_eval(&rows);
        let second = prediction_distribution_eval(&rows);
        assert_eq!(first.total_predictions, 4);
        assert_eq!(first.resolved_outcomes, 2);
        assert_eq!(first.pending_outcomes, 2);
        assert_eq!(first, second);
        assert_eq!(first.score_bins.len(), 4);
        assert_eq!(first.confidence_levels.len(), 1);
        assert!(first.mean_projected_score.unwrap() > 0.0);
    }

    #[test]
    fn deterministic_report_timestamp_uses_latest_prediction() {
        let rows = vec![prediction(1, 70.0, None), prediction(2, 72.0, None)];
        assert_eq!(
            deterministic_report_timestamp(&rows),
            rows[1].predicted_at_secs.0 * MILLIS_PER_SEC
        );
        assert_eq!(deterministic_report_timestamp(&[]), 0);
    }

    #[test]
    fn eval_report_from_collection_is_deterministic() -> Result<()> {
        let mut col = Collection::new();
        let (json1, md1, perf1) = col.gre_atlas_generate_eval_report()?;
        let (json2, md2, perf2) = col.gre_atlas_generate_eval_report()?;
        assert_eq!(json1, json2);
        assert_eq!(md1, md2);
        assert_eq!(perf1, perf2);
        Ok(())
    }

    #[test]
    fn normal_mean_ci_is_deterministic() {
        let values = vec![10.0, 12.0, 14.0, 16.0];
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let ci = normal_mean_ci(mean, &values).unwrap();
        assert!(ci.low <= mean);
        assert!(ci.high >= mean);
        assert_eq!(ci.method, "normal_approximation");
    }

    #[test]
    fn held_out_split_doc_documents_leakage_safety() {
        let doc = held_out_split_doc();
        assert!(doc.rule.contains("% 5 == 0"));
        assert!(doc.leakage_safety.contains("read-only"));
    }

    #[test]
    fn markdown_includes_core_sections() {
        let report = GreAtlasEvalReport {
            generated_at_millis: 1,
            model_version: READINESS_MODEL_VERSION.into(),
            held_out_split: held_out_split_doc(),
            calibration: CalibrationEval {
                total_predictions: 0,
                resolved_outcomes: 0,
                held_out_count: 0,
                sufficient_data: false,
                well_calibrated: false,
                assessment: "insufficient".into(),
                brier_score: None,
                brier_score_ci: None,
                mean_absolute_error: None,
                mean_absolute_error_ci: None,
                bins: Vec::new(),
            },
            performance: compute_performance_eval(&[]),
            memory: MemoryEval {
                model_version: MEMORY_MODEL_VERSION.into(),
                methodology: crate::gre_atlas::memory_eval::memory_methodology(),
                split: crate::gre_atlas::memory_eval::memory_split_doc(),
                fsrs_enabled: false,
                total_reviews_scored: 0,
                held_out_review_count: 0,
                sufficient_data: false,
                brier_score: None,
                log_loss: None,
                brier_score_ci: None,
                reliability_bins: Vec::new(),
                calibration_curve: Vec::new(),
                assessment: "insufficient".into(),
            },
            abstention: AbstentionEval {
                abstention_rate: 1.0,
                memory_abstaining: true,
                performance_abstaining: true,
                readiness_abstaining: true,
                unmet_requirements: Vec::new(),
            },
            coverage: CoverageEval {
                catalog_leaf_count: 1,
                covered_leaf_count: 0,
                unweighted_ratio: 0.0,
                weighted_ratio: 0.0,
                sections: Vec::new(),
            },
            prediction_distribution: PredictionDistributionEval {
                total_predictions: 0,
                resolved_outcomes: 0,
                pending_outcomes: 0,
                mean_projected_score: None,
                min_projected_score: None,
                max_projected_score: None,
                score_bins: Vec::new(),
                confidence_levels: Vec::new(),
            },
            ablation: AblationEval {
                model_version: ABLATION_MODEL_VERSION.into(),
                methodology: ablation_methodology(),
                focus_topic_count: DAILY_FOCUS_TOPIC_COUNT,
                collection: insufficient_scenario("Collection", "collection", "test"),
                synthetic_reference: synthetic_reference_scenario(),
            },
        };
        let md = render_markdown(&report);
        assert!(md.contains("Held-out split"));
        assert!(md.contains("FSRS memory calibration"));
        assert!(md.contains("Performance model"));
        assert!(md.contains("Abstention"));
        assert!(md.contains("Coverage"));
        assert!(md.contains("Prediction distribution"));
        assert!(md.contains("Topic-priority ablation"));
    }
}
