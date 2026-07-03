// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::PrepareDemoCollectionResponse;
use serde::Deserialize;

use crate::collection::Collection;
use crate::decks::DeckId;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::gre_deck_id;
use crate::gre_atlas::GRE_DECK_NAME;
use crate::gre_atlas::LEGACY_GRE_DECK_NAME;
use crate::services::BrainLiftService;

const DEMO_PRACTICE_ATTEMPTS: usize = 4;

#[derive(Debug, Deserialize)]
struct SeedFlashcard {
    front: String,
    back: String,
    topic: String,
}

impl Collection {
    pub fn gre_atlas_prepare_demo_collection(&mut self) -> Result<PrepareDemoCollectionResponse> {
        let deck_existed =
            gre_deck_id(self)?.is_some() || self.get_deck_id(LEGACY_GRE_DECK_NAME)?.is_some();
        let deck = self.get_or_create_normal_deck(GRE_DECK_NAME)?;
        let deck_id = deck.id;
        let cards_added = seed_flashcards_if_empty(self, deck_id)?;
        let practice_attempts_added = seed_practice_attempts_if_empty(self)?;
        let status = self.gre_atlas_gre_study_status()?;

        Ok(PrepareDemoCollectionResponse {
            deck_name: GRE_DECK_NAME.into(),
            deck_created: !deck_existed,
            cards_added,
            practice_attempts_added,
            due_new: status.new_count,
            due_learn: status.learn_count,
            due_review: status.review_count,
        })
    }

    fn gre_atlas_gre_study_status(
        &mut self,
    ) -> Result<anki_proto::brainlift::GreStudyStatusResponse> {
        BrainLiftService::get_gre_study_status(self)
    }
}

fn seed_flashcards_if_empty(col: &mut Collection, deck_id: DeckId) -> Result<u32> {
    if !col.storage.deck_is_empty(deck_id)? {
        return Ok(0);
    }

    let notetype = col.get_notetype_by_name("Basic")?.ok_or_else(|| {
        crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source("Basic notetype missing".into()),
        }
    })?;
    let seed: Vec<SeedFlashcard> = serde_json::from_str(include_str!("flashcards/seed_gre.json"))
        .map_err(|err| crate::error::AnkiError::InvalidInput {
        source: snafu::FromString::without_source(format!("demo flashcards: {err}")),
    })?;

    let mut added = 0u32;
    for card in seed {
        let mut note = notetype.new_note();
        note.set_field(0, &card.front)?;
        note.set_field(1, &card.back)?;
        note.tags = vec![card.topic];
        col.add_note(&mut note, deck_id)?;
        added += 1;
    }
    col.set_current_deck(deck_id)?;
    Ok(added)
}

fn seed_practice_attempts_if_empty(col: &mut Collection) -> Result<u32> {
    let storage = gre_atlas_storage(col)?;
    let (_correct, total) = storage.performance_summary()?;
    if total > 0 {
        return Ok(0);
    }

    let session = storage.create_session("demo")?;
    let questions = storage.list_questions("", DEMO_PRACTICE_ATTEMPTS as u32)?;
    let mut added = 0u32;
    for (index, question) in questions.iter().take(DEMO_PRACTICE_ATTEMPTS).enumerate() {
        let answer = if index % 2 == 0 {
            question.correct_answer.clone()
        } else {
            question
                .choices
                .first()
                .cloned()
                .filter(|choice| choice != &question.correct_answer)
                .unwrap_or_else(|| question.correct_answer.clone())
        };
        let correct = answer.trim() == question.correct_answer.trim();
        storage.record_attempt(
            &question.id,
            &question.topic,
            question.difficulty,
            &answer,
            correct,
            700 + (index as u32 * 120),
            None,
            Some(&session.id),
        )?;
        added += 1;
    }
    Ok(added)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;

    #[test]
    fn prepare_demo_seeds_deck_cards_and_practice() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("demo.anki2")).build()?;
        let response = col.gre_atlas_prepare_demo_collection()?;
        assert!(response.deck_created);
        assert_eq!(response.cards_added, 8);
        assert_eq!(response.practice_attempts_added, 4);
        assert!(response.due_new >= 8);

        let again = col.gre_atlas_prepare_demo_collection()?;
        assert!(!again.deck_created);
        assert_eq!(again.cards_added, 0);
        assert_eq!(again.practice_attempts_added, 0);
        Ok(())
    }
}
