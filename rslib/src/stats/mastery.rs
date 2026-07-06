// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::collections::HashSet;

use fsrs::FSRS;
use fsrs::FSRS5_DEFAULT_DECAY;

use crate::card::CardType;
use crate::config::BoolKey;
use crate::gre_atlas::abstention::sufficient_data_and_reason;
use crate::gre_atlas::compute_coverage;
use crate::gre_atlas::gre_deck_search_str;
use crate::gre_atlas::GreCatalog;
use crate::gre_atlas::TOPIC_TAG_PREFIX;
use crate::prelude::*;
use crate::scheduler::timing::SchedTimingToday;
use crate::search::SortMode;
use crate::timestamp::TimestampMillis;

const DEFAULT_TOPIC_PREFIX: &str = "gre::";
const DEFAULT_MASTERY_THRESHOLD: f32 = 0.9;

/// Welford online mean/variance for memory-efficient aggregation at scale.
#[derive(Default, Clone)]
struct OnlineStats {
    count: u32,
    mean: f64,
    m2: f64,
}

impl OnlineStats {
    fn push(&mut self, value: f32) {
        self.count += 1;
        let x = value as f64;
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
    }

    fn mean_and_ci(&self) -> (f32, f32, f32) {
        if self.count == 0 {
            return (0.0, 0.0, 0.0);
        }
        let mean = self.mean as f32;
        if self.count < 2 {
            let clamped = mean.clamp(0.0, 1.0);
            return (clamped, clamped, clamped);
        }
        let variance = self.m2 / (self.count - 1) as f64;
        let se = (variance / self.count as f64).sqrt();
        let margin = (1.96 * se) as f32;
        (
            mean.clamp(0.0, 1.0),
            (mean - margin).clamp(0.0, 1.0),
            (mean + margin).clamp(0.0, 1.0),
        )
    }
}

#[derive(Default, Clone)]
struct TopicAccumulator {
    total_cards: u32,
    studied_cards: u32,
    mastered_cards: u32,
    retrievability: OnlineStats,
    total_reviews: u32,
}

impl Collection {
    pub fn compute_topic_mastery(
        &mut self,
        req: anki_proto::stats::TopicMasteryRequest,
    ) -> Result<anki_proto::stats::TopicMasteryResponse> {
        if is_standard_gre_mastery_request(&req) {
            let collection_mod = self.gre_collection_revision()?;
            if let Some((cached_mod, cached)) = &self.state.gre_topic_mastery_cache {
                if *cached_mod == collection_mod {
                    return Ok(cached.clone());
                }
            }
            let response = self.compute_topic_mastery_uncached(req)?;
            self.state.gre_topic_mastery_cache = Some((collection_mod, response.clone()));
            return Ok(response);
        }
        self.compute_topic_mastery_uncached(req)
    }

    fn compute_topic_mastery_uncached(
        &mut self,
        req: anki_proto::stats::TopicMasteryRequest,
    ) -> Result<anki_proto::stats::TopicMasteryResponse> {
        let guard = self.search_cards_into_table(req.search.as_str(), SortMode::NoOrder)?;
        topic_mastery_inner(guard.col, req)
    }

    pub(crate) fn fsrs_for_mastery(&mut self) -> Result<&FSRS> {
        if self.state.mastery_fsrs.is_none() {
            self.state.mastery_fsrs = Some(FSRS::new(None)?);
        }
        Ok(self.state.mastery_fsrs.as_ref().unwrap())
    }
}

fn is_standard_gre_mastery_request(req: &anki_proto::stats::TopicMasteryRequest) -> bool {
    req.search == gre_deck_search_str()
        && req.topic_tag_prefix == TOPIC_TAG_PREFIX
        && req.mastery_threshold.is_none()
        && req.min_reviews == 1
}

fn topic_mastery_inner(
    col: &mut Collection,
    req: anki_proto::stats::TopicMasteryRequest,
) -> Result<anki_proto::stats::TopicMasteryResponse> {
    let cards = col.storage.cards_with_tags_for_searched_cards()?;

    let fsrs_enabled = col.get_config_bool(BoolKey::Fsrs);
    let timing = col.timing_today()?;
    let fsrs = col.fsrs_for_mastery()?;

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
    let mut studied_tags: HashSet<String> = HashSet::new();
    let mut unique_studied = 0u32;
    let mut unique_mastered = 0u32;
    let mut overall_retrievability = OnlineStats::default();
    let mut seen_in_topic: HashMap<String, HashSet<CardId>> = HashMap::new();

    for entry in &cards {
        let card = &entry.card;
        let retrievability = card_retrievability(card, &timing, fsrs);
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
            if !tag.starts_with(&topic_prefix) {
                continue;
            }
            if is_studied {
                studied_tags.insert(tag.clone());
            }
            let bucket = topic_bucket_id(tag);
            let seen = seen_in_topic.entry(bucket.clone()).or_default();
            if !seen.insert(card.id) {
                continue;
            }
            let acc = topics.entry(bucket).or_default();
            acc.total_cards += 1;
            if is_studied {
                acc.studied_cards += 1;
                acc.total_reviews += card.reps;
                if let Some(r) = retrievability {
                    acc.retrievability.push(r);
                }
            }
            if is_mastered {
                acc.mastered_cards += 1;
            }
        }
    }

    let studied_tag_refs: Vec<&str> = studied_tags.iter().map(String::as_str).collect();
    let catalog_coverage = compute_coverage(&studied_tag_refs);

    let mut topic_entries = build_topic_entries(&topics, &topic_prefix);
    if let Ok(storage) = crate::gre_atlas::gre_atlas_storage(col) {
        let _ = crate::gre_atlas::topic_mastery_display::enrich_topic_mastery_entries(
            &mut topic_entries,
            storage,
        );
    }
    let topics_studied = count_catalog_topics_studied(&topic_entries);
    let (overall_avg, _, _) = overall_retrievability.mean_and_ci();
    let (sufficient_data, abstain_reason) = sufficient_data_and_reason(
        fsrs_enabled,
        unique_studied,
        catalog_coverage.weighted_ratio,
    );

    Ok(anki_proto::stats::TopicMasteryResponse {
        topics: topic_entries,
        summary: Some(anki_proto::stats::TopicMasterySummary {
            topic_count: catalog_coverage.catalog_leaf_count,
            total_cards: cards.len() as u32,
            studied_cards: unique_studied,
            mastered_cards: unique_mastered,
            overall_avg_retrievability: overall_avg,
            coverage_ratio: catalog_coverage.weighted_ratio,
            sufficient_data,
            abstain_reason,
            topics_studied,
        }),
        computed_at_millis: TimestampMillis::now().0,
        fsrs_enabled,
    })
}

/// Catalog leaf topics with at least one reviewed flashcard.
fn count_catalog_topics_studied(entries: &[anki_proto::stats::TopicMasteryEntry]) -> u32 {
    let by_id: HashMap<&str, &anki_proto::stats::TopicMasteryEntry> = entries
        .iter()
        .map(|entry| (entry.topic_id.as_str(), entry))
        .collect();
    GreCatalog::leaf_topics()
        .filter(|leaf| {
            by_id
                .get(leaf.id)
                .is_some_and(|entry| entry.studied_cards > 0)
        })
        .count() as u32
}

fn topic_bucket_id(tag: &str) -> String {
    GreCatalog::nearest_topic_for_tag(tag)
        .map(|topic| topic.id.to_string())
        .unwrap_or_else(|| tag.to_string())
}

fn build_topic_entries(
    topics: &HashMap<String, TopicAccumulator>,
    topic_prefix: &str,
) -> Vec<anki_proto::stats::TopicMasteryEntry> {
    let mut extra_ids: Vec<_> = topics
        .keys()
        .filter(|id| id.starts_with(topic_prefix))
        .cloned()
        .collect();
    extra_ids.sort();

    let mut seen: HashSet<String> = HashSet::with_capacity(GreCatalog::topics().len());
    let mut entries = Vec::with_capacity(GreCatalog::topics().len() + extra_ids.len());

    for def in GreCatalog::topics() {
        seen.insert(def.id.to_string());
        entries.push(topic_entry_from_accumulator(
            def.id,
            def.display_name.to_string(),
            topics.get(def.id),
        ));
    }

    for id in extra_ids {
        if seen.contains(&id) {
            continue;
        }
        entries.push(topic_entry_from_accumulator(
            &id,
            GreCatalog::display_name_for_tag(&id),
            topics.get(&id),
        ));
    }

    entries.sort_by(|a, b| a.topic_id.cmp(&b.topic_id));
    entries
}

fn topic_entry_from_accumulator(
    topic_id: &str,
    display_name: String,
    acc: Option<&TopicAccumulator>,
) -> anki_proto::stats::TopicMasteryEntry {
    let acc = acc.cloned().unwrap_or_default();
    let (avg, low, high) = acc.retrievability.mean_and_ci();
    anki_proto::stats::TopicMasteryEntry {
        topic_id: topic_id.to_string(),
        display_name,
        total_cards: acc.total_cards,
        studied_cards: acc.studied_cards,
        mastered_cards: acc.mastered_cards,
        avg_retrievability: avg,
        avg_retrievability_low: low,
        avg_retrievability_high: high,
        total_reviews: acc.total_reviews,
        display_mastery: 0.0,
        practice_attempts: 0,
        // Multi-evidence fields filled by enrich_topic_mastery_entries.
        ..Default::default()
    }
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

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::gre_atlas::GreSection;
    use crate::scheduler::answering::test_helpers::PostAnswerState;
    use crate::scheduler::answering::CardAnswer;
    use crate::scheduler::answering::Rating;

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

    fn review_all_cards_good(col: &mut Collection) -> Result<()> {
        let card_ids: Vec<_> = col
            .storage
            .get_all_cards()
            .into_iter()
            .map(|c| c.id)
            .collect();
        for card_id in card_ids {
            let states = col.get_scheduling_states(card_id)?;
            col.answer_card(&mut CardAnswer {
                card_id,
                current_state: states.current,
                new_state: states.good,
                rating: Rating::Good,
                answered_at: TimestampMillis::now(),
                milliseconds_taken: 0,
                custom_data: None,
                from_queue: false,
            })?;
        }
        col.clear_study_queues();
        Ok(())
    }

    #[test]
    fn empty_collection() -> Result<()> {
        let mut col = Collection::new();
        let resp = col.compute_topic_mastery(Default::default())?;
        assert_eq!(
            resp.topics.len(),
            GreCatalog::topics().len(),
            "catalog topics should always be returned"
        );
        assert!(resp.topics.iter().all(|t| t.studied_cards == 0));
        let summary = resp.summary.unwrap();
        assert!(!summary.sufficient_data);
        assert!(!summary.abstain_reason.is_empty());
        assert_eq!(summary.coverage_ratio, 0.0);
        Ok(())
    }

    #[test]
    fn duplicate_topic_tags_count_card_once() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(
            &mut col,
            &[
                "gre::quant::algebra::linear",
                "gre::quant::algebra::linear::flashcard",
            ],
            DeckId(1),
        )?;
        graduate_card(&mut col)?;

        let resp = col.compute_topic_mastery(Default::default())?;
        let topic = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra::linear")
            .unwrap();
        assert_eq!(topic.total_cards, 1);
        assert_eq!(topic.studied_cards, 1);
        Ok(())
    }

    #[test]
    fn per_topic_mastery_is_independent() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra::linear"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::algebra::linear"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::geometry::triangles"], DeckId(1))?;
        graduate_card(&mut col)?;
        graduate_card(&mut col)?;

        let resp = col.compute_topic_mastery(Default::default())?;
        let algebra = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra::linear")
            .unwrap();
        let geometry = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::geometry::triangles")
            .unwrap();
        assert_eq!(algebra.studied_cards, 1);
        assert_eq!(geometry.studied_cards, 0);
        assert!(algebra.avg_retrievability > 0.0);
        assert_eq!(geometry.avg_retrievability, 0.0);
        let summary = resp.summary.unwrap();
        assert!(
            (algebra.avg_retrievability - summary.overall_avg_retrievability).abs() < 0.001,
            "single reviewed topic should match overall mean when only one topic has data",
        );
        Ok(())
    }

    #[test]
    fn single_topic_mastery() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        review_all_cards_good(&mut col)?;

        let resp = col.compute_topic_mastery(Default::default())?;
        let algebra = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra")
            .unwrap();
        assert_eq!(algebra.total_cards, 3);
        assert_eq!(algebra.studied_cards, 3);
        assert!(algebra.avg_retrievability > 0.0 && algebra.avg_retrievability <= 1.0);
        assert!(algebra.avg_retrievability_low <= algebra.avg_retrievability);
        assert!(algebra.avg_retrievability_high >= algebra.avg_retrievability);
        Ok(())
    }

    #[test]
    fn confidence_interval_collapses_for_single_card() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra::linear"], DeckId(1))?;
        graduate_card(&mut col)?;

        let resp = col.compute_topic_mastery(Default::default())?;
        let topic = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra::linear")
            .unwrap();
        assert_eq!(topic.studied_cards, 1);
        assert_eq!(topic.avg_retrievability_low, topic.avg_retrievability);
        assert_eq!(topic.avg_retrievability_high, topic.avg_retrievability);
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
        let algebra = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra")
            .unwrap();
        let geometry = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::geometry")
            .unwrap();
        assert_eq!(algebra.total_cards, 1);
        assert_eq!(geometry.total_cards, 1);
        Ok(())
    }

    #[test]
    fn mastered_threshold() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], DeckId(1))?;
        graduate_card(&mut col)?;
        graduate_card(&mut col)?;
        let resp_high = col.compute_topic_mastery(anki_proto::stats::TopicMasteryRequest {
            mastery_threshold: Some(0.5),
            ..Default::default()
        })?;
        let algebra = resp_high
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra")
            .unwrap();
        assert_eq!(algebra.mastered_cards, 1);

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
    fn topics_studied_counts_catalog_leaves_with_reviewed_cards() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        for tag in [
            "gre::quant::algebra::linear",
            "gre::verbal::text_completion",
        ] {
            add_tagged_note(&mut col, &[tag], DeckId(1))?;
            graduate_card(&mut col)?;
        }

        let resp = col.compute_topic_mastery(Default::default())?;
        let summary = resp.summary.unwrap();
        assert_eq!(summary.topics_studied, 2);
        assert!(summary.topics_studied <= summary.topic_count);
        Ok(())
    }

    #[test]
    fn topics_studied_ignores_practice_only_topics() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let storage = crate::gre_atlas::gre_atlas_storage(&mut col)?;
        let draft = crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft {
            id: "q-tc".into(),
            topic: "gre::verbal::text_completion".into(),
            section: "verbal".into(),
            format: "mcq".into(),
            stem: "Stem".into(),
            choices: vec!["A".into(), "B".into()],
            correct_answer: "A".into(),
            explanation: "Because.\n\n<!-- meta: {\"question_type\":\"text_completion\"} -->"
                .into(),
            difficulty: Some(0.5),
            confidence: 0.8,
            attribution: crate::gre_atlas::questions::ai_gen::QuestionAttribution {
                source_name: "GRE Atlas Practice Bank".into(),
                source_section: "Text completion".into(),
                generated_at_secs: 1,
            },
        };
        storage.insert_generated_question_with_meta(
            &draft,
            &crate::gre_atlas::questions::metadata::QuestionMetadata {
                provenance: crate::gre_atlas::questions::metadata::Provenance::OfflineTemplate,
                model_version: "template_v1".into(),
                source_document: String::new(),
                evaluation_status:
                    crate::gre_atlas::questions::metadata::EvaluationStatus::Approved,
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

        let resp = col.compute_topic_mastery(Default::default())?;
        let summary = resp.summary.unwrap();
        assert_eq!(summary.topics_studied, 0);
        Ok(())
    }

    #[test]
    fn catalog_coverage_in_summary() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        for tag in [
            "gre::quant::algebra::linear",
            "gre::verbal::text_completion",
        ] {
            add_tagged_note(&mut col, &[tag], DeckId(1))?;
            graduate_card(&mut col)?;
        }

        let resp = col.compute_topic_mastery(Default::default())?;
        let summary = resp.summary.unwrap();
        assert!(summary.coverage_ratio > 0.0);
        assert!(summary.coverage_ratio < 1.0);
        assert_eq!(
            summary.topic_count,
            GreCatalog::leaf_topics().count() as u32
        );
        Ok(())
    }

    #[test]
    fn unknown_tag_still_reported() -> Result<()> {
        let mut col = Collection::new();
        add_tagged_note(&mut col, &["gre::experimental::vocab"], DeckId(1))?;
        let resp = col.compute_topic_mastery(Default::default())?;
        assert!(resp
            .topics
            .iter()
            .any(|t| t.topic_id == "gre::experimental::vocab"));
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
    fn leaf_topic_uses_catalog_display_name() -> Result<()> {
        let mut col = Collection::new();
        add_tagged_note(&mut col, &["gre::quant::data_interpretation"], DeckId(1))?;
        let resp = col.compute_topic_mastery(Default::default())?;
        let topic = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::data_interpretation")
            .unwrap();
        assert_eq!(topic.display_name, "Data interpretation");
        Ok(())
    }

    #[test]
    fn gre_deck_search_used_by_eval_harness() -> Result<()> {
        use crate::gre_atlas::GRE_DECK_NAME;
        use crate::gre_atlas::TOPIC_TAG_PREFIX;

        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        let mut deck = Deck::new_normal();
        deck.name = NativeDeckName::from_human_name(GRE_DECK_NAME);
        col.add_deck(&mut deck)?;
        add_tagged_note(&mut col, &["gre::quant::algebra"], deck.id)?;
        col.set_current_deck(deck.id)?;
        graduate_card(&mut col)?;

        let resp = col.compute_topic_mastery(anki_proto::stats::TopicMasteryRequest {
            search: format!("deck:\"{GRE_DECK_NAME}\""),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            min_reviews: 1,
            ..Default::default()
        })?;
        let algebra = resp
            .topics
            .iter()
            .find(|t| t.topic_id == "gre::quant::algebra")
            .unwrap();
        assert_eq!(algebra.studied_cards, 1);
        Ok(())
    }

    #[test]
    fn topic_mastery_scales_to_thousands_of_cards() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let leaf_ids: Vec<&str> = GreCatalog::leaf_topics().map(|leaf| leaf.id).collect();
        for i in 0..5000 {
            let mut note = nt.new_note();
            note.tags = vec![leaf_ids[i % leaf_ids.len()].to_string()];
            col.add_note(&mut note, DeckId(1))?;
        }

        let start = Instant::now();
        let resp = col.compute_topic_mastery(Default::default())?;
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 5,
            "topic_mastery on 5000 cards took {:?}",
            elapsed
        );
        assert_eq!(resp.topics.len(), GreCatalog::topics().len());
        let summary = resp.summary.unwrap();
        assert_eq!(summary.total_cards, 5000);
        assert_eq!(summary.studied_cards, 0);
        assert_eq!(
            summary.topic_count,
            GreCatalog::leaf_topics().count() as u32
        );
        Ok(())
    }

    #[test]
    #[ignore = "manual benchmark; run with cargo test topic_mastery_benchmark -- --ignored"]
    fn topic_mastery_benchmark() -> Result<()> {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        let nt = col.get_notetype_by_name("Basic")?.unwrap();
        let leaf_ids: Vec<&str> = GreCatalog::leaf_topics()
            .filter(|leaf| leaf.section == GreSection::QuantitativeReasoning)
            .map(|leaf| leaf.id)
            .collect();
        for i in 0..1000 {
            let mut note = nt.new_note();
            note.tags = vec![leaf_ids[i % leaf_ids.len()].to_string()];
            col.add_note(&mut note, DeckId(1))?;
            if i % 5 == 0 {
                graduate_card(&mut col)?;
            }
        }
        let start = Instant::now();
        let _ = col.compute_topic_mastery(Default::default())?;
        eprintln!(
            "topic_mastery 1000 cards (200 reviewed): {:?}",
            start.elapsed()
        );
        Ok(())
    }
}
