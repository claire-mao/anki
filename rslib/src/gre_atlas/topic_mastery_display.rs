// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Per-topic mastery display fields for progress charts and related UI.
//!
//! # Mastery model
//!
//! Mastery is a **weighted blend of multiple evidence sources**, each in
//! `[0, 1]`, combined only over the signals that actually have data (weights
//! are renormalized so a single-signal topic is not penalized):
//!
//! | Signal | Source | Weight |
//! |--------|--------|--------|
//! | FSRS recall | `avg_retrievability` (present when `studied_cards > 0`) | 0.45 |
//! | Practice accuracy | correct / total practice attempts | 0.35 |
//! | Recent trend | accuracy over the most recent half of attempts (≥4) | 0.20 |
//!
//! `mastery = Σ(wᵢ · sᵢ) / Σ(wᵢ over present signals)`; `0` when no evidence.
//!
//! # Confidence (reported separately)
//!
//! Confidence in the mastery value is **independent** of the value itself and
//! is driven by how much corroborating evidence exists:
//!
//! - `sample = n / (n + 8)` where `n = total_reviews + practice_attempts`
//!   (n=1 → 0.11, n=8 → 0.50, n=24 → 0.75).
//! - `ci = 1 − (avg_retrievability_high − avg_retrievability_low)` — a wide FSRS
//!   interval (few reviews) lowers confidence; `0.5` when there are no reviews.
//! - `agreement = 1 − |FSRS − practice|` when both exist, else `0.85` (a single
//!   uncorroborated source can never be fully trusted).
//! - `confidence = clamp(sample · ci · agreement)`.
//!
//! **Sparse guarantee:** when `n < 3` the confidence is capped at `0.30`, so a
//! topic with one review is always reported as *Low* confidence with a wide
//! uncertainty band. As reviews/attempts accumulate the band narrows and
//! confidence stabilizes.

use std::collections::HashMap;

use anki_proto::stats::TopicMasteryEntry;

use crate::error::Result;
use crate::gre_atlas::signals::practice_stats_for_topic;
use crate::gre_atlas::storage::GreAtlasStorage;
use crate::gre_atlas::storage::PerformanceAttemptRow;

type TopicQuestionTypeStats = HashMap<(String, String), (u32, u32)>;

/// Weights for each evidence signal (see module docs).
const W_FSRS: f32 = 0.45;
const W_PRACTICE: f32 = 0.35;
const W_TREND: f32 = 0.20;
/// Smoothing constant for the sample-size confidence factor.
const SAMPLE_K: f32 = 8.0;
/// Minimum attempts before a recent-trend signal is computed.
const MIN_TREND_ATTEMPTS: usize = 4;
/// Evidence below this count can never be reported as more than Low confidence.
const SPARSE_EVIDENCE: u32 = 3;
/// How many recent attempts to scan for trend/recency, across all topics.
const RECENT_ATTEMPT_SCAN: u32 = 500;

pub(crate) fn enrich_topic_mastery_entries(
    entries: &mut [TopicMasteryEntry],
    storage: &GreAtlasStorage,
) -> Result<()> {
    let stats = storage.performance_stats_by_topic_question_type()?;
    let by_topic = stats_by_topic(&stats);
    let recent = storage.recent_attempts("", RECENT_ATTEMPT_SCAN)?;

    for entry in entries.iter_mut() {
        let (p_correct, p_total) =
            practice_stats_for_topic(&entry.topic_id, &by_topic).unwrap_or((0, 0));
        let practice_accuracy = if p_total > 0 {
            p_correct as f32 / p_total as f32
        } else {
            0.0
        };
        let trend = recent_trend(&recent, &entry.topic_id);

        let mastery = weighted_mastery(entry, practice_accuracy, p_total, trend);
        let evidence = entry.total_reviews + p_total;
        let confidence = mastery_confidence(entry, practice_accuracy, p_total, evidence);
        let (low, high) = mastery_band(mastery, confidence);

        entry.display_mastery = mastery;
        entry.practice_attempts = p_total;
        entry.practice_accuracy = practice_accuracy;
        entry.evidence_count = evidence;
        entry.mastery_confidence = confidence;
        entry.confidence_label = confidence_label(confidence, evidence).to_string();
        entry.mastery_low = low;
        entry.mastery_high = high;
    }
    Ok(())
}

/// Blend the available evidence signals into a single mastery value in [0, 1].
fn weighted_mastery(
    entry: &TopicMasteryEntry,
    practice_accuracy: f32,
    practice_total: u32,
    trend: Option<f32>,
) -> f32 {
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    if entry.studied_cards > 0 {
        weighted_sum += W_FSRS * entry.avg_retrievability;
        weight_total += W_FSRS;
    }
    if practice_total > 0 {
        weighted_sum += W_PRACTICE * practice_accuracy;
        weight_total += W_PRACTICE;
    }
    if let Some(trend) = trend {
        weighted_sum += W_TREND * trend;
        weight_total += W_TREND;
    }
    if weight_total <= 0.0 {
        return 0.0;
    }
    (weighted_sum / weight_total).clamp(0.0, 1.0)
}

/// Accuracy over the most recent half of a topic's practice attempts.
/// `recent` is ordered newest-first; `None` until there are enough attempts.
fn recent_trend(recent: &[PerformanceAttemptRow], topic_id: &str) -> Option<f32> {
    let topic_attempts: Vec<&PerformanceAttemptRow> = recent
        .iter()
        .filter(|a| attempt_matches_topic(&a.topic, topic_id))
        .collect();
    if topic_attempts.len() < MIN_TREND_ATTEMPTS {
        return None;
    }
    let half = (topic_attempts.len() / 2).max(1);
    let window = &topic_attempts[..half];
    let correct = window.iter().filter(|a| a.correct).count();
    Some(correct as f32 / window.len() as f32)
}

fn attempt_matches_topic(attempt_topic: &str, topic_id: &str) -> bool {
    attempt_topic == topic_id
        || (attempt_topic.len() > topic_id.len()
            && attempt_topic.starts_with(topic_id)
            && attempt_topic[topic_id.len()..].starts_with("::"))
        || (topic_id.len() > attempt_topic.len()
            && topic_id.starts_with(attempt_topic)
            && topic_id[attempt_topic.len()..].starts_with("::"))
}

/// Confidence in the mastery value (0–1). See module docs for the formula.
fn mastery_confidence(
    entry: &TopicMasteryEntry,
    practice_accuracy: f32,
    practice_total: u32,
    evidence: u32,
) -> f32 {
    let n = evidence as f32;
    let sample = n / (n + SAMPLE_K);

    let ci = if entry.total_reviews > 0 {
        let width = (entry.avg_retrievability_high - entry.avg_retrievability_low).clamp(0.0, 1.0);
        1.0 - width
    } else {
        0.5
    };

    let has_fsrs = entry.studied_cards > 0;
    let agreement = if has_fsrs && practice_total > 0 {
        1.0 - (entry.avg_retrievability - practice_accuracy).abs()
    } else {
        0.85
    };

    let mut confidence = (sample * ci * agreement).clamp(0.0, 1.0);
    if evidence < SPARSE_EVIDENCE {
        confidence = confidence.min(0.30);
    }
    confidence
}

fn confidence_label(confidence: f32, evidence: u32) -> &'static str {
    if evidence < SPARSE_EVIDENCE || confidence < 0.34 {
        "Low"
    } else if confidence < 0.67 {
        "Moderate"
    } else {
        "High"
    }
}

/// Uncertainty band around the mastery value: up to ±0.25 when confidence is 0,
/// shrinking to a point as confidence approaches 1.
fn mastery_band(mastery: f32, confidence: f32) -> (f32, f32) {
    let half_width = (1.0 - confidence) * 0.25;
    (
        (mastery - half_width).clamp(0.0, 1.0),
        (mastery + half_width).clamp(0.0, 1.0),
    )
}

fn stats_by_topic(stats: &TopicQuestionTypeStats) -> HashMap<String, (u32, u32)> {
    let mut by_topic = HashMap::new();
    for ((topic, _), (correct, total)) in stats {
        let entry = by_topic.entry(topic.clone()).or_insert((0, 0));
        entry.0 += correct;
        entry.1 += total;
    }
    by_topic
}

pub(crate) fn expected_question_types(topic_id: &str) -> Vec<&'static str> {
    if topic_id.starts_with("gre::verbal::reading::") {
        return vec!["reading_comprehension"];
    }
    if topic_id == "gre::verbal::text_completion" {
        return vec!["text_completion"];
    }
    if topic_id == "gre::verbal::sentence_equivalence" {
        return vec!["sentence_equivalence"];
    }
    if topic_id.starts_with("gre::verbal::vocabulary::") {
        return vec!["text_completion", "sentence_equivalence"];
    }
    if topic_id == "gre::awa::issue" {
        return vec!["awa_issue"];
    }
    if topic_id == "gre::awa::argument" {
        return vec!["awa_argument"];
    }
    if topic_id.starts_with("gre::quant::") {
        return vec!["mcq"];
    }
    vec!["mcq"]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::gre_atlas::gre_atlas_storage;
    use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
    use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
    use crate::gre_atlas::questions::metadata::EvaluationStatus;
    use crate::gre_atlas::questions::metadata::Provenance;
    use crate::gre_atlas::questions::metadata::QuestionMetadata;

    #[test]
    fn expected_question_types_follow_catalog_shape() {
        assert_eq!(
            expected_question_types("gre::verbal::reading::inference"),
            vec!["reading_comprehension"]
        );
        assert_eq!(
            expected_question_types("gre::quant::algebra::linear"),
            vec!["mcq"]
        );
        assert_eq!(
            expected_question_types("gre::awa::issue"),
            vec!["awa_issue"]
        );
    }

    #[test]
    fn enrich_topic_mastery_entries_sets_per_topic_display() -> crate::error::Result<()> {
        let mut entries = vec![
            TopicMasteryEntry {
                topic_id: "gre::quant::algebra::linear".into(),
                display_name: "Linear equations".into(),
                studied_cards: 2,
                avg_retrievability: 0.82,
                ..Default::default()
            },
            TopicMasteryEntry {
                topic_id: "gre::quant::geometry::triangles".into(),
                display_name: "Triangles".into(),
                studied_cards: 1,
                avg_retrievability: 0.61,
                ..Default::default()
            },
        ];
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = gre_atlas_storage(&mut col)?;
        enrich_topic_mastery_entries(&mut entries, storage)?;

        assert!((entries[0].display_mastery - 0.82).abs() < 0.001);
        assert!((entries[1].display_mastery - 0.61).abs() < 0.001);
        assert!((entries[0].display_mastery - entries[1].display_mastery).abs() > 0.01);
        Ok(())
    }

    #[test]
    fn sparse_evidence_reports_low_confidence_and_wide_band() -> crate::error::Result<()> {
        let mut entries = vec![TopicMasteryEntry {
            topic_id: "gre::quant::algebra::linear".into(),
            display_name: "Linear equations".into(),
            studied_cards: 1,
            total_reviews: 1,
            avg_retrievability: 0.9,
            avg_retrievability_low: 0.5,
            avg_retrievability_high: 1.0,
            ..Default::default()
        }];
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = gre_atlas_storage(&mut col)?;
        enrich_topic_mastery_entries(&mut entries, storage)?;

        let entry = &entries[0];
        assert_eq!(entry.evidence_count, 1);
        assert_eq!(entry.confidence_label, "Low");
        assert!(entry.mastery_confidence <= 0.30);
        // A single review yields a wide uncertainty band.
        assert!(entry.mastery_high - entry.mastery_low > 0.3);
        Ok(())
    }

    #[test]
    fn confidence_never_high_with_sparse_evidence_even_if_ci_tight() -> crate::error::Result<()> {
        let mut entries = vec![TopicMasteryEntry {
            topic_id: "gre::quant::algebra::linear".into(),
            studied_cards: 2,
            total_reviews: 2,
            avg_retrievability: 0.95,
            avg_retrievability_low: 0.94,
            avg_retrievability_high: 0.96,
            ..Default::default()
        }];
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = gre_atlas_storage(&mut col)?;
        enrich_topic_mastery_entries(&mut entries, storage)?;

        assert!(entries[0].evidence_count < 3);
        assert_eq!(entries[0].confidence_label, "Low");
        assert!(entries[0].mastery_confidence <= 0.30);
        Ok(())
    }

    #[test]
    fn ample_evidence_narrows_band_and_lifts_confidence() -> crate::error::Result<()> {
        let mut entries = vec![TopicMasteryEntry {
            topic_id: "gre::quant::algebra::linear".into(),
            studied_cards: 20,
            total_reviews: 40,
            avg_retrievability: 0.85,
            avg_retrievability_low: 0.80,
            avg_retrievability_high: 0.90,
            ..Default::default()
        }];
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = gre_atlas_storage(&mut col)?;
        enrich_topic_mastery_entries(&mut entries, storage)?;

        let entry = &entries[0];
        assert_ne!(entry.confidence_label, "Low");
        assert!(entry.mastery_confidence > 0.34);
        // Many reviews stabilize the estimate into a narrow band.
        assert!(entry.mastery_high - entry.mastery_low < 0.2);
        Ok(())
    }

    #[test]
    fn enrich_topic_mastery_entries_blends_memory_and_practice() -> crate::error::Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = gre_atlas_storage(&mut col)?;
        let draft = GeneratedQuestionDraft {
            id: "q-tc".into(),
            topic: "gre::verbal::text_completion".into(),
            section: "verbal".into(),
            format: "mcq".into(),
            stem: "Stem".into(),
            choices: vec!["A".into(), "B".into()],
            correct_answer: "A".into(),
            explanation: "Because.\n\n<!-- meta: {\"question_type\":\"text_completion\"} -->".into(),
            difficulty: Some(0.5),
            confidence: 0.8,
            attribution: QuestionAttribution {
                source_name: "GRE Atlas Practice Bank".into(),
                source_section: "Text completion".into(),
                generated_at_secs: 1,
            },
        };
        storage.insert_generated_question_with_meta(
            &draft,
            &QuestionMetadata {
                provenance: Provenance::OfflineTemplate,
                model_version: "template_v1".into(),
                source_document: String::new(),
                evaluation_status: EvaluationStatus::Approved,
            },
        )?;
        storage.record_attempt(
            "q-tc",
            "gre::verbal::text_completion",
            Some(0.5),
            "A",
            true,
            1000,
            None,
            None,
        )?;
        storage.record_attempt(
            "q-tc",
            "gre::verbal::text_completion",
            Some(0.5),
            "B",
            false,
            1000,
            None,
            None,
        )?;

        let mut entries = vec![TopicMasteryEntry {
            topic_id: "gre::verbal::text_completion".into(),
            display_name: "Text completion".into(),
            studied_cards: 2,
            avg_retrievability: 0.8,
            ..Default::default()
        }];
        enrich_topic_mastery_entries(&mut entries, storage)?;

        let entry = &entries[0];
        assert_eq!(entry.practice_attempts, 2);
        assert!((entry.practice_accuracy - 0.5).abs() < 0.001);
        // Blended: (0.45 × 0.8 FSRS + 0.35 × 0.5 practice) / 0.80 = 0.66875.
        assert!(
            (entry.display_mastery - 0.66875).abs() < 0.001,
            "{}",
            entry.display_mastery
        );
        Ok(())
    }
}
