// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::ReadinessCalibrationBin;
use anki_proto::brainlift::ReadinessCalibrationResponse;
use anki_proto::brainlift::ReadinessCalibrationStats;
use anki_proto::brainlift::ReadinessScore;

use crate::brainlift::brainlift_storage;
use crate::brainlift::readiness::MEMORY_WEIGHT;
use crate::brainlift::readiness::PERFORMANCE_WEIGHT;
use crate::brainlift::readiness::COVERAGE_WEIGHT;
use crate::brainlift::storage::BrainliftStorage;
use crate::brainlift::storage::ReadinessPredictionRow;
use crate::error::Result;
use crate::timestamp::TimestampSecs;

pub(crate) const READINESS_MODEL_VERSION: &str = "readiness_v1";
pub(crate) const MIN_CALIBRATION_HELD_OUT: u32 = 5;
pub(crate) const MIN_OUTCOME_AGE_SECS: i64 = 3 * 86_400;
pub(crate) const MIN_OUTCOME_PRACTICE_ATTEMPTS: u32 = 3;
pub(crate) const MIN_PREDICTION_INTERVAL_SECS: i64 = 6 * 3_600;
pub(crate) const PREDICTION_SCORE_DELTA: f32 = 2.0;
pub(crate) const BRIER_WELL_CALIBRATED_MAX: f32 = 0.08;
pub(crate) const BRIER_MODERATE_MAX: f32 = 0.15;
pub(crate) const CALIBRATION_BIN_COUNT: u32 = 10;

#[derive(Debug, Clone)]
pub(crate) struct ReadinessPredictionSnapshot {
    pub projected_score: f32,
    pub projected_score_low: Option<f32>,
    pub projected_score_high: Option<f32>,
    pub memory_score: f32,
    pub performance_score: f32,
    pub coverage_ratio: f32,
    pub confidence_level: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OutcomeInputs {
    pub memory_score: f32,
    pub performance_score: f32,
    pub coverage_ratio: f32,
    pub practice_correct: u32,
    pub practice_total: u32,
}

pub(crate) fn composite_readiness_score(
    memory_score: f32,
    performance_score: f32,
    coverage_ratio: f32,
) -> f32 {
    let memory_norm = memory_score / 100.0;
    let perf_norm = performance_score / 100.0;
    (MEMORY_WEIGHT * memory_norm + PERFORMANCE_WEIGHT * perf_norm + COVERAGE_WEIGHT * coverage_ratio)
        * 100.0
}

pub(crate) fn resolve_outcome_score(
    prediction: &ReadinessPredictionRow,
    inputs: &OutcomeInputs,
    now: TimestampSecs,
) -> Option<f32> {
    let age = now.0 - prediction.predicted_at_secs.0;
    if age < MIN_OUTCOME_AGE_SECS && inputs.practice_total < MIN_OUTCOME_PRACTICE_ATTEMPTS {
        return None;
    }

    let perf_norm = if inputs.practice_total > 0 {
        inputs.practice_correct as f32 / inputs.practice_total as f32
    } else {
        prediction.performance_score / 100.0
    };

    Some(
        composite_readiness_score(
            inputs.memory_score,
            perf_norm * 100.0,
            inputs.coverage_ratio,
        )
        .clamp(0.0, 100.0),
    )
}

pub(crate) fn is_held_out_prediction(id: i64) -> bool {
    id % 5 == 0
}

pub(crate) fn compute_calibration_stats(rows: &[ReadinessPredictionRow]) -> ReadinessCalibrationStats {
    let total_predictions = rows.len() as u32;
    let resolved: Vec<_> = rows.iter().filter(|row| row.outcome_score.is_some()).collect();
    let resolved_outcomes = resolved.len() as u32;

    let held_out: Vec<_> = resolved
        .iter()
        .copied()
        .filter(|row| is_held_out_prediction(row.id))
        .collect();
    let held_out_count = held_out.len() as u32;

    if held_out_count < MIN_CALIBRATION_HELD_OUT {
        return ReadinessCalibrationStats {
            total_predictions,
            resolved_outcomes,
            held_out_count,
            brier_score: None,
            mean_absolute_error: None,
            calibration_curve: Vec::new(),
            sufficient_data: false,
            well_calibrated: false,
            assessment: format!(
                "Calibration requires at least {MIN_CALIBRATION_HELD_OUT} held-out predictions with observed outcomes (current: {held_out_count}). Model confidence is unverified."
            ),
        };
    }

    let pairs: Vec<(f32, f32)> = held_out
        .iter()
        .map(|row| {
            (
                row.projected_score / 100.0,
                row.outcome_score.unwrap() / 100.0,
            )
        })
        .collect();

    let brier = brier_score(&pairs);
    let brier_score = Some(brier);
    let mean_absolute_error = Some(mean_absolute_error(&pairs));
    let calibration_curve = calibration_curve(&held_out);
    let (well_calibrated, assessment) = calibration_assessment(brier, held_out_count);

    ReadinessCalibrationStats {
        total_predictions,
        resolved_outcomes,
        held_out_count,
        brier_score,
        mean_absolute_error,
        calibration_curve,
        sufficient_data: true,
        well_calibrated,
        assessment,
    }
}

pub(crate) fn apply_calibration_honesty(
    mut readiness: ReadinessScore,
    stats: &ReadinessCalibrationStats,
) -> ReadinessScore {
    readiness.calibration_sufficient_data = stats.sufficient_data;
    readiness.calibration_well_calibrated = stats.well_calibrated;
    readiness.calibration_brier_score = stats.brier_score;
    readiness.calibration_note = stats.assessment.clone();

    if !readiness.sufficient_data {
        return readiness;
    }

    if stats.sufficient_data && !stats.well_calibrated {
        readiness.confidence_level = "low".into();
    }

    readiness
}

pub(crate) fn maintain_readiness_calibration(
    storage: &BrainliftStorage,
    snapshot: Option<&ReadinessPredictionSnapshot>,
    outcome_inputs: &OutcomeInputs,
) -> Result<ReadinessCalibrationStats> {
    storage.resolve_pending_outcomes(outcome_inputs)?;
    if let Some(snapshot) = snapshot {
        storage.maybe_record_readiness_prediction(snapshot)?;
    }
    let rows = storage.list_readiness_predictions()?;
    Ok(compute_calibration_stats(&rows))
}

impl crate::collection::Collection {
    pub fn brainlift_get_readiness_calibration(&mut self) -> Result<ReadinessCalibrationResponse> {
        let signals = self.load_brainlift_signals(1)?;
        let storage = brainlift_storage(self)?;
        let calibration = compute_calibration_stats(&storage.list_readiness_predictions()?);
        Ok(ReadinessCalibrationResponse {
            readiness: Some(signals.readiness),
            calibration: Some(calibration),
            computed_at_millis: signals.computed_at_millis,
        })
    }
}

fn brier_score(pairs: &[(f32, f32)]) -> f32 {
    if pairs.is_empty() {
        return 0.0;
    }
    pairs
        .iter()
        .map(|(predicted, outcome)| (predicted - outcome).powi(2))
        .sum::<f32>()
        / pairs.len() as f32
}

fn mean_absolute_error(pairs: &[(f32, f32)]) -> f32 {
    if pairs.is_empty() {
        return 0.0;
    }
    pairs
        .iter()
        .map(|(predicted, outcome)| (predicted * 100.0 - outcome * 100.0).abs())
        .sum::<f32>()
        / pairs.len() as f32
}

fn calibration_curve(rows: &[&ReadinessPredictionRow]) -> Vec<ReadinessCalibrationBin> {
    let mut bins = Vec::with_capacity(CALIBRATION_BIN_COUNT as usize);
    for bin in 0..CALIBRATION_BIN_COUNT {
        let bin_low = bin as f32 * 10.0;
        let bin_high = bin_low + 10.0;
        let in_bin: Vec<_> = rows
            .iter()
            .filter(|row| row.projected_score >= bin_low && row.projected_score < bin_high)
            .collect();
        if in_bin.is_empty() {
            continue;
        }
        let predicted_mean = in_bin.iter().map(|row| row.projected_score).sum::<f32>()
            / in_bin.len() as f32;
        let outcome_mean = in_bin
            .iter()
            .map(|row| row.outcome_score.unwrap())
            .sum::<f32>()
            / in_bin.len() as f32;
        bins.push(ReadinessCalibrationBin {
            bin_low,
            bin_high,
            predicted_mean,
            outcome_mean,
            count: in_bin.len() as u32,
        });
    }
    bins
}

fn calibration_assessment(brier: f32, held_out_count: u32) -> (bool, String) {
    if brier <= BRIER_WELL_CALIBRATED_MAX {
        (
            true,
            format!(
                "Held-out calibration on {held_out_count} predictions: Brier score {brier:.3}. Predictions track later outcomes reasonably well."
            ),
        )
    } else if brier <= BRIER_MODERATE_MAX {
        (
            false,
            format!(
                "Held-out calibration on {held_out_count} predictions: Brier score {brier:.3}. Calibration is moderate; readiness scores are approximate."
            ),
        )
    } else {
        (
            false,
            format!(
                "Held-out calibration on {held_out_count} predictions: Brier score {brier:.3}. Predictions poorly match later outcomes; treat confidence ranges skeptically."
            ),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
    fn brier_score_perfect_when_predictions_match() {
        let pairs = vec![(0.7, 0.7), (0.8, 0.8)];
        assert!((brier_score(&pairs) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn held_out_filter_is_deterministic() {
        assert!(is_held_out_prediction(5));
        assert!(!is_held_out_prediction(1));
    }

    #[test]
    fn insufficient_calibration_data_is_reported_honestly() {
        let rows = vec![prediction(5, 75.0, Some(70.0))];
        let stats = compute_calibration_stats(&rows);
        assert!(!stats.sufficient_data);
        assert!(!stats.well_calibrated);
        assert!(stats.assessment.contains("unverified"));
    }

    #[test]
    fn poor_calibration_is_not_hidden() {
        let rows = (0..10)
            .map(|idx| {
                let id = (idx + 1) * 5;
                prediction(id, 90.0, Some(50.0))
            })
            .collect::<Vec<_>>();
        let stats = compute_calibration_stats(&rows);
        assert!(stats.sufficient_data);
        assert!(!stats.well_calibrated);
        assert!(stats.brier_score.unwrap() > BRIER_WELL_CALIBRATED_MAX);
        assert!(stats.assessment.contains("poorly match"));
    }

    #[test]
    fn apply_calibration_downgrades_confidence_when_poor() {
        let readiness = ReadinessScore {
            projected_score: Some(80.0),
            projected_score_low: Some(70.0),
            projected_score_high: Some(90.0),
            confidence_level: "high".into(),
            coverage_ratio: 0.6,
            last_updated_millis: 1,
            evidence_summary: "test".into(),
            sufficient_data: true,
            abstain_reason: String::new(),
            calibration_note: String::new(),
            calibration_brier_score: None,
            calibration_sufficient_data: false,
            calibration_well_calibrated: false,
            abstention_requirements: Vec::new(),
        };
        let stats = ReadinessCalibrationStats {
            total_predictions: 10,
            resolved_outcomes: 10,
            held_out_count: 10,
            brier_score: Some(0.2),
            mean_absolute_error: Some(20.0),
            calibration_curve: Vec::new(),
            sufficient_data: true,
            well_calibrated: false,
            assessment: "poor".into(),
        };
        let adjusted = apply_calibration_honesty(readiness, &stats);
        assert_eq!(adjusted.confidence_level, "low");
        assert_eq!(adjusted.calibration_note, "poor");
    }

    #[test]
    fn resolve_outcome_requires_age_or_practice() {
        let row = prediction(1, 75.0, None);
        let inputs = OutcomeInputs {
            memory_score: 70.0,
            performance_score: 65.0,
            coverage_ratio: 0.5,
            practice_correct: 0,
            practice_total: 0,
        };
        assert!(resolve_outcome_score(
            &row,
            &inputs,
            TimestampSecs(row.predicted_at_secs.0 + 3600)
        )
        .is_none());
        assert!(resolve_outcome_score(
            &row,
            &inputs,
            TimestampSecs(row.predicted_at_secs.0 + MIN_OUTCOME_AGE_SECS + 1)
        )
        .is_some());
    }
}
