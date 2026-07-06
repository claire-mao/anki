// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::brainlift::StartGreExtraStudyResponse;
use anki_proto::scheduler::custom_study_request::Value as CustomStudyValue;
use anki_proto::scheduler::CustomStudyRequest;

use crate::collection::Collection;
use crate::decks::tree::get_deck_in_tree;
use crate::error::OrInvalid;
use crate::error::Result;
use crate::gre_atlas::gre_deck_id;
use crate::gre_atlas::topic_flashcard_schedule::gre_deck_flashcard_schedule;
use crate::prelude::*;

pub const GRE_EXTRA_STUDY_BATCH: u32 = 8;
pub const GRE_EXTRA_STUDY_DAILY_CAP: u32 = 20;

pub(crate) fn enrich_gre_study_status(
    col: &mut Collection,
    mut status: GreStudyStatusResponse,
) -> Result<GreStudyStatusResponse> {
    let Some(deck_id) = gre_deck_id(col)? else {
        return Ok(status);
    };

    let available_new = gre_available_new_count(col, deck_id)?;
    let extend_new = gre_extend_new_today(col, deck_id)?;
    status.available_new_count = available_new;
    status.extra_study_available = extra_study_available(available_new, extend_new);
    if status.new_count + status.learn_count + status.review_count == 0 {
        status.next_review_in_days = gre_deck_flashcard_schedule(col)?
            .next_due_in_days
            .filter(|days| *days > 0);
    }
    Ok(status)
}

pub(crate) fn gre_atlas_start_extra_study(
    col: &mut Collection,
) -> Result<StartGreExtraStudyResponse> {
    let deck_id = gre_deck_id(col)?
        .or_invalid("GRE Atlas study deck not found")?;
    let available_new = gre_available_new_count(col, deck_id)?;
    let extend_new = gre_extend_new_today(col, deck_id)?;
    let cards_unlocked = extra_study_available(available_new, extend_new);
    if cards_unlocked == 0 {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(
                "No extra GRE flashcards available right now".into(),
            ),
        });
    }

    col.custom_study(CustomStudyRequest {
        deck_id: deck_id.0,
        value: Some(CustomStudyValue::NewLimitDelta(cards_unlocked as i32)),
    })?;

    let timing = col.timing_today()?;
    let learn_cutoff = timing.now.0 as u32 + col.learn_ahead_secs();
    let counts_map = col.due_counts(timing.days_elapsed, learn_cutoff)?;
    let (new_count, learn_count, review_count) = counts_map
        .get(&deck_id)
        .map(|counts| (counts.new, counts.learning, counts.review))
        .unwrap_or((0, 0, 0));

    Ok(StartGreExtraStudyResponse {
        cards_unlocked,
        new_count,
        learn_count,
        review_count,
    })
}

fn extra_study_available(available_new: u32, extend_new: u32) -> u32 {
    if available_new == 0 {
        return 0;
    }
    let remaining_cap = GRE_EXTRA_STUDY_DAILY_CAP.saturating_sub(extend_new);
    if remaining_cap == 0 {
        return 0;
    }
    available_new
        .min(remaining_cap)
        .min(GRE_EXTRA_STUDY_BATCH)
}

fn gre_available_new_count(col: &mut Collection, deck_id: crate::decks::DeckId) -> Result<u32> {
    let now = col.timing_today()?.now;
    let subtree =
        get_deck_in_tree(col.deck_tree(Some(now))?, deck_id).or_not_found(deck_id)?;
    Ok(subtree.new_uncapped)
}

fn gre_extend_new_today(col: &mut Collection, deck_id: crate::decks::DeckId) -> Result<u32> {
    let deck = col.get_deck(deck_id)?.or_not_found(deck_id)?;
    Ok(deck.normal()?.extend_new)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::services::BrainLiftService;

    #[test]
    fn extra_study_available_respects_daily_cap_and_batch() {
        assert_eq!(extra_study_available(20, 0), GRE_EXTRA_STUDY_BATCH);
        assert_eq!(extra_study_available(3, 0), 3);
        assert_eq!(extra_study_available(20, GRE_EXTRA_STUDY_DAILY_CAP), 0);
        assert_eq!(
            extra_study_available(20, GRE_EXTRA_STUDY_DAILY_CAP - 2),
            2
        );
        assert_eq!(extra_study_available(0, 0), 0);
    }

    #[test]
    fn gre_study_status_reports_extra_study_fields() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("extra.anki2")).build()?;
        let _ = col.gre_atlas_prepare_demo_collection()?;

        let status = col.get_gre_study_status()?;
        assert!(status.available_new_count >= status.new_count);
        assert!(status.available_new_count > 0);

        if status.extra_study_available > 0 {
            let response = col.start_gre_extra_study()?;
            assert_eq!(response.cards_unlocked, status.extra_study_available);
            assert!(response.new_count >= status.new_count);
        }
        Ok(())
    }
}
