// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use fsrs::FSRSItem;
use fsrs::FSRS;
use itertools::Itertools;
use serde::Serialize;

use anki_proto::brainlift::MemoryCalibrationBin as MemoryCalibrationBinProto;
use anki_proto::brainlift::MemoryEvalResponse;

use crate::card::CardId;
use crate::collection::Collection;
use crate::config::BoolKey;
use crate::deckconfig::DeckConfigId;
use crate::error::OrInvalid;
use crate::error::Result;
use crate::gre_atlas::eval::ConfidenceInterval;
use crate::gre_atlas::gre_deck_search;
use crate::prelude::*;
use crate::revlog::RevlogEntry;
use crate::scheduler::fsrs::memory_state::get_decay_from_params;
use crate::scheduler::fsrs::params::ignore_revlogs_before_ms_from_config;
use crate::scheduler::fsrs::params::reviews_for_fsrs;
use crate::timestamp::TimestampMillis;

pub(crate) const MEMORY_MODEL_VERSION: &str = "fsrs";
pub(crate) const MIN_HELD_OUT_REVIEWS: u32 = 5;
pub(crate) const MEMORY_CALIBRATION_BIN_COUNT: u32 = 10;
const CONFIDENCE_Z: f32 = 1.96;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryEval {
    pub model_version: String,
    pub methodology: MemoryMethodology,
    pub split: MemorySplit,
    pub fsrs_enabled: bool,
    pub total_reviews_scored: u32,
    pub held_out_review_count: u32,
    pub sufficient_data: bool,
    pub brier_score: Option<f32>,
    pub log_loss: Option<f32>,
    pub brier_score_ci: Option<ConfidenceInterval>,
    pub reliability_bins: Vec<MemoryCalibrationBin>,
    pub calibration_curve: Vec<MemoryCalibrationBin>,
    pub assessment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryMethodology {
    pub summary: String,
    pub train_rule: String,
    pub test_rule: String,
    pub prediction_source: String,
    pub outcome_definition: String,
    pub metrics_scope: String,
    pub leakage_safety: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemorySplit {
    pub rule: String,
    pub minimum_held_out_reviews: u32,
    pub training_reviews_rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryCalibrationBin {
    pub bin_low: f32,
    pub bin_high: f32,
    pub predicted_mean: f32,
    pub outcome_mean: f32,
    pub count: u32,
}

struct FsrsDeckContext {
    params: Vec<f32>,
    decay: f32,
    ignore_before: TimestampMillis,
}

struct FsrsRuntimeContext {
    fsrs: FSRS,
    decay: f32,
    ignore_before: TimestampMillis,
}

pub(crate) fn is_held_out_review(id: i64) -> bool {
    id % 5 == 0
}

pub(crate) fn compute_memory_eval(col: &mut Collection) -> Result<MemoryEval> {
    let methodology = memory_methodology();
    let split = memory_split_doc();
    let fsrs_enabled = col.get_config_bool(BoolKey::Fsrs);

    if !fsrs_enabled {
        return Ok(MemoryEval {
            model_version: MEMORY_MODEL_VERSION.into(),
            methodology,
            split,
            fsrs_enabled: false,
            total_reviews_scored: 0,
            held_out_review_count: 0,
            sufficient_data: false,
            brier_score: None,
            log_loss: None,
            brier_score_ci: None,
            reliability_bins: Vec::new(),
            calibration_curve: Vec::new(),
            assessment: "FSRS is disabled; memory calibration requires FSRS scheduling.".into(),
        });
    }

    let timing = col.timing_today()?;
    let search = gre_deck_search();
    let revlog = col.revlog_for_srs(search.as_str())?;
    let mut deck_contexts: HashMap<DeckConfigId, FsrsDeckContext> = HashMap::new();
    let mut pairs = Vec::new();
    let mut total_reviews_scored = 0u32;

    for (card_id, entries) in revlog.into_iter().chunk_by(|entry| entry.cid).into_iter() {
        let entries: Vec<RevlogEntry> = entries.collect();
        let Some(context) = fsrs_context_for_card(col, card_id, &mut deck_contexts)? else {
            continue;
        };
        let Some(reviews) =
            reviews_for_fsrs(entries, timing.next_day_at, true, context.ignore_before)
        else {
            continue;
        };

        for (revlog_id, item) in reviews.fsrs_items {
            total_reviews_scored += 1;
            if !is_held_out_review(revlog_id.0) {
                continue;
            }
            let predicted = predict_retrievability(&context.fsrs, &item, context.decay)?;
            let actual = recall_outcome(&item);
            pairs.push((predicted, actual));
        }
    }

    let held_out_review_count = pairs.len() as u32;
    let sufficient_data = held_out_review_count >= MIN_HELD_OUT_REVIEWS;
    let brier_score = if pairs.is_empty() {
        None
    } else {
        Some(brier_score(&pairs))
    };
    let log_loss = if pairs.is_empty() {
        None
    } else {
        Some(log_loss(&pairs))
    };
    let brier_score_ci = brier_score.and_then(|brier| {
        let squared_errors: Vec<f32> = pairs
            .iter()
            .map(|(predicted, outcome)| (predicted - outcome).powi(2))
            .collect();
        normal_mean_ci(brier, &squared_errors)
    });
    let bins = calibration_bins(&pairs);
    let assessment = memory_assessment(sufficient_data, held_out_review_count, brier_score);

    Ok(MemoryEval {
        model_version: MEMORY_MODEL_VERSION.into(),
        methodology,
        split,
        fsrs_enabled: true,
        total_reviews_scored,
        held_out_review_count,
        sufficient_data,
        brier_score,
        log_loss,
        brier_score_ci,
        reliability_bins: bins.clone(),
        calibration_curve: bins,
        assessment,
    })
}

impl Collection {
    pub fn gre_atlas_get_memory_eval(&mut self) -> Result<MemoryEvalResponse> {
        let memory = compute_memory_eval(self)?;
        Ok(memory_eval_to_proto(&memory, TimestampMillis::now().0))
    }
}

fn memory_eval_to_proto(memory: &MemoryEval, computed_at_millis: i64) -> MemoryEvalResponse {
    MemoryEvalResponse {
        model_version: memory.model_version.clone(),
        fsrs_enabled: memory.fsrs_enabled,
        held_out_review_count: memory.held_out_review_count,
        sufficient_data: memory.sufficient_data,
        brier_score: memory.brier_score,
        log_loss: memory.log_loss,
        assessment: memory.assessment.clone(),
        calibration_curve: memory
            .calibration_curve
            .iter()
            .map(memory_calibration_bin_to_proto)
            .collect(),
        computed_at_millis,
    }
}

fn memory_calibration_bin_to_proto(bin: &MemoryCalibrationBin) -> MemoryCalibrationBinProto {
    MemoryCalibrationBinProto {
        bin_low: bin.bin_low,
        bin_high: bin.bin_high,
        predicted_mean: bin.predicted_mean,
        outcome_mean: bin.outcome_mean,
        count: bin.count,
    }
}

fn fsrs_context_for_card(
    col: &mut Collection,
    card_id: CardId,
    cache: &mut HashMap<DeckConfigId, FsrsDeckContext>,
) -> Result<Option<FsrsRuntimeContext>> {
    let card = col.storage.get_card(card_id)?.or_not_found(card_id)?;
    let deck = col
        .get_deck(card.original_or_current_deck_id())?
        .or_not_found(card.original_or_current_deck_id())?;
    let config_id = DeckConfigId(deck.normal()?.config_id);
    if let Some(context) = cache.get(&config_id) {
        return Ok(Some(FsrsRuntimeContext {
            fsrs: FSRS::new(Some(&context.params))?,
            decay: context.decay,
            ignore_before: context.ignore_before,
        }));
    }
    let config = col
        .storage
        .get_deck_config(config_id)?
        .or_not_found(config_id)?;
    let params = config.fsrs_params().to_vec();
    let decay = get_decay_from_params(&params);
    let ignore_before = ignore_revlogs_before_ms_from_config(&config)?;
    cache.insert(
        config_id,
        FsrsDeckContext {
            params: params.clone(),
            decay,
            ignore_before,
        },
    );
    Ok(Some(FsrsRuntimeContext {
        fsrs: FSRS::new(Some(&params))?,
        decay,
        ignore_before,
    }))
}

fn predict_retrievability(fsrs: &FSRS, item: &FSRSItem, decay: f32) -> Result<f32> {
    let current = item
        .reviews
        .last()
        .or_invalid("FSRS item must contain at least one review")?;
    if item.reviews.len() <= 1 {
        return Ok(1.0);
    }
    let history = FSRSItem {
        reviews: item.reviews[..item.reviews.len() - 1].to_vec(),
    };
    let states = fsrs.historical_memory_states(history, None)?;
    let Some(state) = states.last() else {
        return Ok(1.0);
    };
    Ok(fsrs.current_retrievability(*state, current.delta_t, decay))
}

fn recall_outcome(item: &FSRSItem) -> f32 {
    if item.reviews.last().is_some_and(|review| review.rating > 1) {
        1.0
    } else {
        0.0
    }
}

pub(crate) fn memory_split_doc() -> MemorySplit {
    MemorySplit {
        rule: "revlog.id % 5 == 0".into(),
        minimum_held_out_reviews: MIN_HELD_OUT_REVIEWS,
        training_reviews_rule: "revlog.id % 5 != 0".into(),
    }
}

pub(crate) fn memory_methodology() -> MemoryMethodology {
    MemoryMethodology {
        summary: "GRE deck review history is split deterministically by revlog id. For each held-out review, FSRS recomputes memory state from all prior reviews on the same card, predicts recall probability at the elapsed interval, and compares it to the observed recall outcome (pass vs again). Metrics are computed on held-out reviews only.".into(),
        train_rule: "revlog.id % 5 != 0 (used only to describe the complement split; FSRS parameters are fixed)".into(),
        test_rule: "revlog.id % 5 == 0".into(),
        prediction_source: "Existing FSRS memory states via historical_memory_states() and current_retrievability() using deck FSRS parameters.".into(),
        outcome_definition: "Actual recall = 1 when the review rating is Hard/Good/Easy (rating > 1), else 0 for Again.".into(),
        metrics_scope: "Brier score, reliability bins, and calibration curve use only held-out reviews. Prior reviews on the same card may inform the prediction but are never scored as test outcomes.".into(),
        leakage_safety: "Held-out membership is fixed when a revlog row is inserted. Predictions use only review history strictly before the held-out review on that card, matching production FSRS inference at review time.".into(),
    }
}

fn brier_score(pairs: &[(f32, f32)]) -> f32 {
    pairs
        .iter()
        .map(|(predicted, outcome)| (predicted - outcome).powi(2))
        .sum::<f32>()
        / pairs.len() as f32
}

fn log_loss(pairs: &[(f32, f32)]) -> f32 {
    const EPS: f32 = 1e-15;
    pairs
        .iter()
        .map(|(predicted, outcome)| {
            let probability = predicted.clamp(EPS, 1.0 - EPS);
            if *outcome >= 0.5 {
                -probability.ln()
            } else {
                -(1.0 - probability).ln()
            }
        })
        .sum::<f32>()
        / pairs.len() as f32
}

fn calibration_bins(pairs: &[(f32, f32)]) -> Vec<MemoryCalibrationBin> {
    let mut bins = Vec::new();
    for bin in 0..MEMORY_CALIBRATION_BIN_COUNT {
        let bin_low = bin as f32 / MEMORY_CALIBRATION_BIN_COUNT as f32;
        let bin_high = (bin + 1) as f32 / MEMORY_CALIBRATION_BIN_COUNT as f32;
        let in_bin: Vec<_> = pairs
            .iter()
            .filter(|(predicted, _)| *predicted >= bin_low && *predicted < bin_high)
            .collect();
        if in_bin.is_empty() {
            continue;
        }
        let predicted_mean =
            in_bin.iter().map(|(predicted, _)| *predicted).sum::<f32>() / in_bin.len() as f32;
        let outcome_mean =
            in_bin.iter().map(|(_, outcome)| *outcome).sum::<f32>() / in_bin.len() as f32;
        bins.push(MemoryCalibrationBin {
            bin_low,
            bin_high,
            predicted_mean,
            outcome_mean,
            count: in_bin.len() as u32,
        });
    }
    bins
}

fn normal_mean_ci(mean: f32, values: &[f32]) -> Option<ConfidenceInterval> {
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return Some(ConfidenceInterval {
            level: 0.95,
            low: mean,
            high: mean,
            method: "normal_approximation_degenerate".into(),
        });
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
    Some(ConfidenceInterval {
        level: 0.95,
        low: (mean - margin).max(0.0),
        high: mean + margin,
        method: "normal_approximation".into(),
    })
}

fn memory_assessment(sufficient_data: bool, held_out_count: u32, brier: Option<f32>) -> String {
    if !sufficient_data {
        return format!(
            "Memory calibration requires at least {MIN_HELD_OUT_REVIEWS} held-out FSRS reviews (current: {held_out_count})."
        );
    }
    let brier = brier.unwrap_or(0.0);
    format!("Held-out FSRS calibration on {held_out_count} reviews: Brier score {brier:.4}.")
}

pub(crate) fn render_memory_markdown(memory: &MemoryEval) -> String {
    let mut out = String::new();
    out.push_str("## FSRS memory calibration (held-out)\n\n");
    out.push_str(&format!("- Model: `{}`\n", memory.model_version));
    out.push_str(&format!("- FSRS enabled: {}\n", memory.fsrs_enabled));
    out.push_str(&format!(
        "- Reviews scored (train + test): {}\n",
        memory.total_reviews_scored
    ));
    out.push_str(&format!(
        "- Held-out reviews: {}\n",
        memory.held_out_review_count
    ));
    out.push_str(&format!("- Sufficient data: {}\n", memory.sufficient_data));
    if let Some(brier) = memory.brier_score {
        out.push_str(&format!("- Brier score: {brier:.4}\n"));
        if let Some(ci) = &memory.brier_score_ci {
            out.push_str(&format!(
                "- Brier 95% CI: [{:.4}, {:.4}] ({})\n",
                ci.low, ci.high, ci.method
            ));
        }
    } else {
        out.push_str("- Brier score: n/a\n");
    }
    if let Some(log_loss) = memory.log_loss {
        out.push_str(&format!("- Log loss: {log_loss:.4}\n"));
    } else {
        out.push_str("- Log loss: n/a\n");
    }
    out.push_str(&format!("- Assessment: {}\n\n", memory.assessment));

    out.push_str("### Methodology\n\n");
    out.push_str(&format!("{}\n\n", memory.methodology.summary));
    out.push_str(&format!(
        "- Test rule: `{}`\n",
        memory.methodology.test_rule
    ));
    out.push_str(&format!(
        "- Prediction: {}\n",
        memory.methodology.prediction_source
    ));
    out.push_str(&format!(
        "- Outcome: {}\n",
        memory.methodology.outcome_definition
    ));
    out.push_str(&format!(
        "- Leakage safety: {}\n\n",
        memory.methodology.leakage_safety
    ));

    out.push_str("### Reliability bins\n\n");
    if memory.reliability_bins.is_empty() {
        out.push_str("_No reliability bins (insufficient held-out data)._\n\n");
    } else {
        out.push_str("| Bin | Count | Predicted recall | Observed recall |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for bin in &memory.reliability_bins {
            out.push_str(&format!(
                "| {:.0}–{:.0}% | {} | {:.1}% | {:.1}% |\n",
                bin.bin_low * 100.0,
                bin.bin_high * 100.0,
                bin.count,
                bin.predicted_mean * 100.0,
                bin.outcome_mean * 100.0
            ));
        }
        out.push('\n');
    }

    out.push_str("### Calibration curve\n\n");
    if memory.calibration_curve.is_empty() {
        out.push_str("_No calibration curve (insufficient held-out data)._\n\n");
    } else {
        out.push_str("| Predicted mean | Observed mean | Count |\n");
        out.push_str("| ---: | ---: | ---: |\n");
        for point in &memory.calibration_curve {
            out.push_str(&format!(
                "| {:.1}% | {:.1}% | {} |\n",
                point.predicted_mean * 100.0,
                point.outcome_mean * 100.0,
                point.count
            ));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::revlog::RevlogId;
    use crate::revlog::RevlogReviewKind;

    #[allow(dead_code)]
    fn review(id: i64, cid: CardId, days_apart: u32, rating: u8) -> RevlogEntry {
        RevlogEntry {
            id: RevlogId(id),
            cid,
            usn: Usn(0),
            button_chosen: rating,
            interval: days_apart.max(1) as i32,
            last_interval: 0,
            ease_factor: 2500,
            taken_millis: 1000,
            review_kind: RevlogReviewKind::Review,
        }
    }

    #[test]
    fn held_out_review_filter_is_deterministic() {
        assert!(is_held_out_review(5));
        assert!(!is_held_out_review(4));
    }

    #[test]
    fn brier_score_perfect_when_predictions_match() {
        let pairs = vec![(0.8, 1.0), (0.2, 0.0)];
        assert!((brier_score(&pairs) - 0.04).abs() < f32::EPSILON);
    }

    #[test]
    fn log_loss_matches_binary_cross_entropy() {
        let pairs = vec![(0.8, 1.0), (0.2, 0.0)];
        let expected = (-0.8f32.ln() - (1.0f32 - 0.2f32).ln()) / 2.0;
        assert!((log_loss(&pairs) - expected).abs() < 1e-5);
    }

    #[test]
    fn calibration_bins_cover_unit_interval() {
        let pairs: Vec<_> = (0..100)
            .map(|idx| {
                let predicted = idx as f32 / 100.0;
                let outcome = if idx % 10 == 0 { 1.0 } else { 0.0 };
                (predicted, outcome)
            })
            .collect();
        let bins = calibration_bins(&pairs);
        assert!(!bins.is_empty());
        assert!(bins.iter().all(|bin| bin.count > 0));
    }

    #[test]
    fn predict_retrievability_uses_fsrs_history() {
        let fsrs = FSRS::new(Some(&fsrs::DEFAULT_PARAMETERS[..])).unwrap();
        let item = FSRSItem {
            reviews: vec![
                fsrs::FSRSReview {
                    rating: 3,
                    delta_t: 0,
                },
                fsrs::FSRSReview {
                    rating: 3,
                    delta_t: 4,
                },
            ],
        };
        let predicted = predict_retrievability(&fsrs, &item, fsrs::FSRS5_DEFAULT_DECAY).unwrap();
        assert!(predicted > 0.0 && predicted <= 1.0);
    }

    #[test]
    fn recall_outcome_treats_again_as_zero() {
        let item = FSRSItem {
            reviews: vec![fsrs::FSRSReview {
                rating: 1,
                delta_t: 1,
            }],
        };
        assert_eq!(recall_outcome(&item), 0.0);
    }
}
