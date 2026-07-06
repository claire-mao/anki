// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Staged release of bundled foundation flashcards after GRE practice.

use std::collections::HashMap;

use anki_proto::scheduler::bury_or_suspend_cards_request::Mode as BuryOrSuspendMode;

use crate::card::CardId;
use crate::collection::Collection;
use crate::decks::DeckId;
use crate::error::OrInvalid;
use crate::error::Result;
use crate::gre_atlas::flashcards::foundation_flashcards_for_topic;
use crate::gre_atlas::flashcards::FoundationFlashcard;
use crate::gre_atlas::flashcards::FLASHCARDS_PER_BATCH;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::gre_deck_id;
use crate::gre_atlas::storage::GreAtlasStorage;
use crate::gre_atlas::topic_flashcard_schedule::topic_flashcard_schedule;
use crate::gre_atlas::topic_flashcard_schedule::TopicFlashcardSchedule;
use crate::timestamp::TimestampSecs;

pub const FLASHCARD_BATCH_COUNT: u32 = 3;
/// Days after practice before each staged batch unsuspends (batch 0 is immediate).
pub const FLASHCARD_BATCH_INTERVAL_DAYS: u32 = 1;
pub const MIN_PRACTICE_ATTEMPTS_FOR_RELEASE: u32 = 3;

pub(crate) fn gre_atlas_process_flashcard_schedule(col: &mut Collection) -> Result<()> {
    {
        let storage = gre_atlas_storage(col)?;
        storage.reconcile_flashcard_batch_intervals(FLASHCARD_BATCH_INTERVAL_DAYS)?;
    }
    let due_batches = {
        let storage = gre_atlas_storage(col)?;
        storage.pending_topic_flashcard_batches(TimestampSecs::now())?
    };
    for batch in due_batches {
        if batch.card_ids.is_empty() {
            let storage = gre_atlas_storage(col)?;
            storage.mark_topic_flashcard_batch_released(&batch.topic, batch.batch_index)?;
            continue;
        }
        col.unbury_or_unsuspend_cards(&batch.card_ids)?;
        let storage = gre_atlas_storage(col)?;
        storage.mark_topic_flashcard_batch_released(&batch.topic, batch.batch_index)?;
    }
    Ok(())
}

pub(crate) fn gre_atlas_on_practice_attempt(col: &mut Collection, topic_id: &str) -> Result<()> {
    let attempt_count = {
        let storage = gre_atlas_storage(col)?;
        storage.topic_attempt_count(topic_id)?
    };
    if attempt_count < MIN_PRACTICE_ATTEMPTS_FOR_RELEASE {
        return Ok(());
    }

    let already_scheduled = {
        let storage = gre_atlas_storage(col)?;
        storage.topic_flashcard_batches_exist(topic_id)?
    };
    if already_scheduled {
        return Ok(());
    }

    schedule_topic_flashcards(col, topic_id)?;
    Ok(())
}

pub fn topic_flashcard_schedule_for_topic(
    col: &mut Collection,
    topic_id: &str,
) -> Result<TopicFlashcardSchedule> {
    gre_atlas_process_flashcard_schedule(col)?;
    let mut schedule = topic_flashcard_schedule(col, topic_id)?;
    let storage = gre_atlas_storage(col)?;
    enrich_schedule_with_pending(storage, topic_id, &mut schedule);
    Ok(schedule)
}

fn schedule_topic_flashcards(col: &mut Collection, topic_id: &str) -> Result<bool> {
    let templates = foundation_flashcards_for_topic(topic_id);
    if templates.is_empty() {
        return Ok(false);
    }

    let deck_id = gre_deck_id(col)?
        .or_invalid("GRE Atlas study deck not found")?;
    let now = TimestampSecs::now();
    let existing_guids = col.storage.all_notes_by_guid()?;
    let mut batches: HashMap<u32, Vec<CardId>> = HashMap::new();

    for (index, template) in templates.iter().enumerate() {
        let batch_index = (index / FLASHCARDS_PER_BATCH) as u32;
        if batch_index >= FLASHCARD_BATCH_COUNT {
            break;
        }
        let card_id = add_topic_flashcard(col, deck_id, template, &existing_guids)?;
        batches.entry(batch_index).or_default().push(card_id);
    }

    if batches.is_empty() {
        return Ok(false);
    }

    let mut batch_records = Vec::new();
    for batch_index in 0..FLASHCARD_BATCH_COUNT {
        let Some(card_ids) = batches.remove(&batch_index) else {
            continue;
        };
        let release_at_secs = if batch_index == 0 {
            now
        } else {
            now.adding_secs(
                (batch_index * FLASHCARD_BATCH_INTERVAL_DAYS * 86_400) as i64,
            )
        };
        let suspend = batch_index > 0;
        batch_records.push((batch_index, release_at_secs, card_ids, suspend));
    }

    for (_, _, card_ids, suspend) in &batch_records {
        if *suspend {
            col.bury_or_suspend_cards(card_ids, BuryOrSuspendMode::Suspend)?;
        }
    }

    let storage = gre_atlas_storage(col)?;
    for (batch_index, release_at_secs, card_ids, suspend) in batch_records {
        storage.insert_topic_flashcard_batch(
            topic_id,
            batch_index,
            release_at_secs,
            &card_ids,
            !suspend,
        )?;
    }

    Ok(true)
}

fn add_topic_flashcard(
    col: &mut Collection,
    deck_id: DeckId,
    template: &FoundationFlashcard,
    existing_guids: &HashMap<String, crate::notes::NoteId>,
) -> Result<CardId> {
    let guid = flashcard_guid(&template.question_id);
    if let Some(note_id) = existing_guids.get(&guid) {
        return col
            .storage
            .all_card_ids_of_note_in_template_order(*note_id)?
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!(
                    "flashcard note missing card for {}",
                    template.question_id
                )),
            });
    }

    let notetype = col.get_notetype_by_name("Basic")?.ok_or_else(|| {
        crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source("Basic notetype missing".into()),
        }
    })?;
    let mut note = notetype.new_note();
    note.guid = guid;
    note.set_field(0, &template.front)?;
    note.set_field(1, &template.back)?;
    note.tags = vec![
        template.topic.clone(),
        format!("{}::flashcard", template.topic),
        format!("gre-flashcard-{}", template.question_id),
    ];
    col.add_note(&mut note, deck_id)?;
    col.storage
        .all_card_ids_of_note_in_template_order(note.id)?
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "failed to add flashcard for {}",
                template.question_id
            )),
        })
}

fn flashcard_guid(question_id: &str) -> String {
    format!("grefc:{question_id}")
}

fn enrich_schedule_with_pending(
    storage: &GreAtlasStorage,
    topic_id: &str,
    schedule: &mut TopicFlashcardSchedule,
) {
    if let Ok(pending) = storage.pending_topic_flashcard_batch_summary(topic_id, TimestampSecs::now())
    {
        schedule.pending_batches = pending.pending_batches;
        if schedule.next_batch_in_days.is_none() {
            schedule.next_batch_in_days = pending.next_batch_in_days;
        }
        if schedule.next_due_in_days.is_none() {
            schedule.next_due_in_days = pending.next_batch_in_days;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::error::Result;
    use crate::gre_atlas::gre_atlas_storage;
    use crate::gre_atlas::topic_flashcard_schedule::gre_deck_flashcard_schedule;
    use crate::services::BrainLiftService;

    fn record_topic_attempts(col: &mut Collection, topic: &str, count: u32) -> Result<()> {
        let (question, session_id) = {
            let storage = gre_atlas_storage(col)?;
            let questions = storage.list_questions(topic, count.max(1))?;
            let question = questions
                .first()
                .ok_or_else(|| crate::error::AnkiError::InvalidInput {
                    source: snafu::FromString::without_source(format!(
                        "no practice questions for {topic}"
                    )),
                })?
                .clone();
            let session = storage.create_session("test")?;
            (question, session.id)
        };
        for _ in 0..count {
            {
                let storage = gre_atlas_storage(col)?;
                storage.record_attempt(
                    &question.id,
                    topic,
                    question.difficulty,
                    &question.correct_answer,
                    true,
                    900,
                    None,
                    Some(&session_id),
                )?;
            }
            gre_atlas_on_practice_attempt(col, topic)?;
        }
        Ok(())
    }

    #[test]
    fn schedules_foundation_flashcards_after_practice_threshold() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("release.anki2")).build()?;
        let _ = col.gre_atlas_prepare_demo_collection()?;

        let topic = "gre::quant::algebra::linear";
        record_topic_attempts(&mut col, topic, MIN_PRACTICE_ATTEMPTS_FOR_RELEASE - 1)?;
        let storage = gre_atlas_storage(&mut col)?;
        assert!(!storage.topic_flashcard_batches_exist(topic)?);

        record_topic_attempts(&mut col, topic, 1)?;
        let storage = gre_atlas_storage(&mut col)?;
        assert!(storage.topic_flashcard_batches_exist(topic)?);

        let schedule = topic_flashcard_schedule_for_topic(&mut col, topic)?;
        assert!(schedule.total_cards >= FLASHCARDS_PER_BATCH as u32);
        assert!(schedule.due_now >= 1);
        assert!(schedule.hint().contains("ready now"));
        Ok(())
    }

    #[test]
    fn extra_study_unlocks_newly_scheduled_flashcards() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("extra-release.anki2")).build()?;
        let _ = col.gre_atlas_prepare_demo_collection()?;
        col.update_default_deck_config(|config| {
            config.new_per_day = 0;
        });

        let topic = "gre::quant::algebra::linear";
        record_topic_attempts(&mut col, topic, MIN_PRACTICE_ATTEMPTS_FOR_RELEASE)?;

        let status = col.get_gre_study_status()?;
        assert!(status.available_new_count >= FLASHCARDS_PER_BATCH as u32);
        assert!(status.extra_study_available > 0);

        let response = col.start_gre_extra_study()?;
        assert!(response.new_count > 0);

        let deck_schedule = gre_deck_flashcard_schedule(&mut col)?;
        assert!(deck_schedule.due_now > 0);
        Ok(())
    }
}
