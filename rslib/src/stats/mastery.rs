// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use fsrs::FSRS;
use fsrs::FSRS5_DEFAULT_DECAY;

use crate::card::CardType;
use crate::config::BoolKey;
use crate::prelude::*;
use crate::scheduler::timing::SchedTimingToday;
use crate::search::SortMode;
use crate::timestamp::TimestampMillis;

const DEFAULT_TOPIC_PREFIX: &str = "gre::";
const DEFAULT_MASTERY_THRESHOLD: f32 = 0.9;
const MIN_STUDIED_CARDS_FOR_SUFFICIENT: u32 = 200;
const MIN_COVERAGE_RATIO: f32 = 0.5;

#[derive(Default)]
struct TopicAccumulator {
    total_cards: u32,
    studied_cards: u32,
    mastered_cards: u32,
    retrievability_values: Vec<f32>,
    total_reviews: u32,
}

impl Collection {
    pub fn compute_topic_mastery(
        &mut self,
        req: anki_proto::stats::TopicMasteryRequest,
    ) -> Result<anki_proto::stats::TopicMasteryResponse> {
        let guard = self.search_cards_into_table(req.search.as_str(), SortMode::NoOrder)?;
        topic_mastery_inner(guard.col, req)
    }
}

fn topic_mastery_inner(
    col: &mut Collection,
    req: anki_proto::stats::TopicMasteryRequest,
) -> Result<anki_proto::stats::TopicMasteryResponse> {
    let cards = col.storage.cards_with_tags_for_searched_cards()?;

    let fsrs_enabled = col.get_config_bool(BoolKey::Fsrs);
    let timing = col.timing_today()?;
    let fsrs = FSRS::new(None)?;

    let topic_prefix = if req.topic_tag_prefix.is_empty() {
        DEFAULT_TOPIC_PREFIX.to_string()
    } else {
        req.topic_tag_prefix.clone()
    };
    let min_reviews = if req.min_reviews == 0 {
        1
    } else {
        req.min_reviews
    };
    let mastery_threshold = req.mastery_threshold.unwrap_or(DEFAULT_MASTERY_THRESHOLD);

    let mut topics: HashMap<String, TopicAccumulator> = HashMap::new();
    let mut unique_studied = 0u32;
    let mut unique_mastered = 0u32;
    let mut overall_retrievability = Vec::new();

    for entry in &cards {
        let card = &entry.card;
        let retrievability = card_retrievability(card, &timing, &fsrs);
        let is_studied = card.reps >= min_reviews;
        let is_mastered = is_studied
            && card.ctype == CardType::Review
            && retrievability
                .map(|r| r >= mastery_threshold)
                .unwrap_or(false);

        if is_studied {
            unique_studied += 1;
            if let Some(r) = retrievability {
                overall_retrievability.push(r);
            }
        }
        if is_mastered {
            unique_mastered += 1;
        }

        for tag in &entry.tags {
            if tag.starts_with(&topic_prefix) {
                let acc = topics.entry(tag.clone()).or_default();
                acc.total_cards += 1;
                if is_studied {
                    acc.studied_cards += 1;
                    acc.total_reviews += card.reps;
                    if let Some(r) = retrievability {
                        acc.retrievability_values.push(r);
                    }
                }
                if is_mastered {
                    acc.mastered_cards += 1;
                }
            }
        }
    }

    let mut topic_entries: Vec<anki_proto::stats::TopicMasteryEntry> = topics
        .into_iter()
        .map(|(topic_id, acc)| {
            let (avg, low, high) = mean_and_ci(&acc.retrievability_values);
            anki_proto::stats::TopicMasteryEntry {
                topic_id: topic_id.clone(),
                display_name: topic_display_name(&topic_id, &topic_prefix),
                total_cards: acc.total_cards,
                studied_cards: acc.studied_cards,
                mastered_cards: acc.mastered_cards,
                avg_retrievability: avg,
                avg_retrievability_low: low,
                avg_retrievability_high: high,
                total_reviews: acc.total_reviews,
            }
        })
        .collect();
    topic_entries.sort_by(|a, b| a.topic_id.cmp(&b.topic_id));

    let topic_count = topic_entries.len() as u32;
    let topics_with_studied = topic_entries.iter().filter(|t| t.studied_cards > 0).count() as u32;
    let coverage_ratio = if topic_count > 0 {
        topics_with_studied as f32 / topic_count as f32
    } else {
        0.0
    };

    let (overall_avg, _, _) = mean_and_ci(&overall_retrievability);
    let (sufficient_data, abstain_reason) =
        sufficient_data_and_reason(fsrs_enabled, unique_studied, coverage_ratio);

    Ok(anki_proto::stats::TopicMasteryResponse {
        topics: topic_entries,
        summary: Some(anki_proto::stats::TopicMasterySummary {
            topic_count,
            total_cards: cards.len() as u32,
            studied_cards: unique_studied,
            mastered_cards: unique_mastered,
            overall_avg_retrievability: overall_avg,
            coverage_ratio,
            sufficient_data,
            abstain_reason,
        }),
        computed_at_millis: TimestampMillis::now().0,
        fsrs_enabled,
    })
}

fn card_retrievability(card: &Card, timing: &SchedTimingToday, fsrs: &FSRS) -> Option<f32> {
    card.memory_state.map(|state| {
        let elapsed = card.seconds_since_last_review(timing).unwrap_or_default();
        fsrs.current_retrievability_seconds(
            state.into(),
            elapsed,
            card.decay.unwrap_or(FSRS5_DEFAULT_DECAY),
        )
    })
}

fn mean_and_ci(values: &[f32]) -> (f32, f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    if values.len() < 2 {
        return (mean, mean, mean);
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / (n - 1.0);
    let se = (variance / n).sqrt();
    let margin = 1.96 * se;
    (
        mean.clamp(0.0, 1.0),
        (mean - margin).clamp(0.0, 1.0),
        (mean + margin).clamp(0.0, 1.0),
    )
}

fn topic_display_name(topic_id: &str, prefix: &str) -> String {
    let rest = topic_id.strip_prefix(prefix).unwrap_or(topic_id);
    rest.replace("::", " / ")
}

fn sufficient_data_and_reason(
    fsrs_enabled: bool,
    unique_studied: u32,
    coverage_ratio: f32,
) -> (bool, String) {
    let mut reasons = Vec::new();
    if !fsrs_enabled {
        reasons.push("FSRS is not enabled".to_string());
    }
    if unique_studied < MIN_STUDIED_CARDS_FOR_SUFFICIENT {
        reasons.push(format!(
            "Fewer than {MIN_STUDIED_CARDS_FOR_SUFFICIENT} studied cards in scope (current: {unique_studied})"
        ));
    }
    if coverage_ratio < MIN_COVERAGE_RATIO {
        reasons.push(format!(
            "Topic coverage below {:.0}% (current: {:.0}%)",
            MIN_COVERAGE_RATIO * 100.0,
            coverage_ratio * 100.0
        ));
    }
    let sufficient_data = reasons.is_empty();
    (sufficient_data, reasons.join("; "))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::scheduler::answering::test_helpers::PostAnswerState;

    fn add_tagged_note(col: &mut Collection, tags: &[&str], deck_id: DeckId) -> Result<NoteId> {
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let mut note = nt.new_note();
        note.tags = tags.iter().map(|t| t.to_string()).collect();
        col.add_note(&mut note, deck_id)?;
        Ok(note.id)
    }

    fn graduate_card(col: &mut Collection) -> Result<PostAnswerState> {
        col.storage.db.execute_batch("update cards set due=0")?;
        col.clear_study_queues();
        Ok(col.answer_good())
    }

    #[test]
    fn empty_collection() -> Result<()> {
        let mut col = Collection::new();
        let resp = col.compute_topic_mastery(Default::default())?;
        assert!(resp.topics.is_empty());
        let summary = resp.summary.unwrap();
        assert!(!summary.sufficient_data);
        assert!(!summary.abstain_reason.is_empty());
        Ok(())
    }

    #[test]
    fn single_topic_mastery() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        graduate_card(&mut col)?;

        let resp = col.compute_topic_mastery(Default::default())?;
        assert_eq!(resp.topics.len(), 1);
        let topic = &resp.topics[0];
        assert_eq!(topic.topic_id, "gre::quant::algebra");
        assert_eq!(topic.total_cards, 3);
        assert!(topic.studied_cards > 0);
        assert!(topic.avg_retrievability > 0.0 && topic.avg_retrievability <= 1.0);
        Ok(())
    }

    #[test]
    fn multi_tag_card_counts_in_both() -> Result<()> {
        let mut col = Collection::new();
        add_tagged_note(
            &mut col,
            &["gre::quant::algebra", "gre::quant::geometry"],
            DeckId(1),
        )?;
        let resp = col.compute_topic_mastery(Default::default())?;
        assert_eq!(resp.topics.len(), 2);
        assert!(resp
            .topics
            .iter()
            .all(|t| t.total_cards == 1 && t.topic_id.starts_with("gre::quant::")));
        Ok(())
    }

    #[test]
    fn mastered_threshold() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        // Graduate through learning steps.
        graduate_card(&mut col)?;
        graduate_card(&mut col)?;
        let resp_high = col.compute_topic_mastery(anki_proto::stats::TopicMasteryRequest {
            mastery_threshold: Some(0.5),
            ..Default::default()
        })?;
        assert_eq!(resp_high.topics[0].mastered_cards, 1);

        add_tagged_note(&mut col, &["gre::quant::geometry"], DeckId(1))?;
        graduate_card(&mut col)?;
        let resp_strict = col.compute_topic_mastery(anki_proto::stats::TopicMasteryRequest {
            mastery_threshold: Some(0.99),
            ..Default::default()
        })?;
        let geometry = resp_strict
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::geometry")
            .unwrap();
        assert_eq!(geometry.mastered_cards, 0);
        Ok(())
    }

    #[test]
    fn cards_with_tags_for_search_returns_tags() -> Result<()> {
        let mut col = Collection::new();
        add_tagged_note(&mut col, &["gre::verbal::vocab"], DeckId(1))?;
        let guard = col.search_cards_into_table("", SortMode::NoOrder)?;
        let cards = guard.col.storage.cards_with_tags_for_searched_cards()?;
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].tags, vec!["gre::verbal::vocab"]);
        Ok(())
    }

    #[test]
    #[ignore = "manual benchmark; run with cargo test topic_mastery_benchmark -- --ignored"]
    fn topic_mastery_benchmark() -> Result<()> {
        use std::time::Instant;

        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        for i in 0..1000 {
            let mut note = nt.new_note();
            note.tags = vec![format!("gre::quant::topic{}", i % 20)];
            col.add_note(&mut note, DeckId(1))?;
        }
        let start = Instant::now();
        let _ = col.compute_topic_mastery(Default::default())?;
        eprintln!("topic_mastery 1000 cards: {:?}", start.elapsed());
        Ok(())
    }
}
