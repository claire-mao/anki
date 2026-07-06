// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::card::CardType;
use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::gre_deck_search_str;
use crate::gre_atlas::TOPIC_TAG_PREFIX;
use crate::search::SortMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicFlashcardSchedule {
    pub total_cards: u32,
    pub due_now: u32,
    pub new_count: u32,
    pub next_due_in_days: Option<u32>,
    pub pending_batches: u32,
    pub next_batch_in_days: Option<u32>,
}

impl Default for TopicFlashcardSchedule {
    fn default() -> Self {
        Self {
            total_cards: 0,
            due_now: 0,
            new_count: 0,
            next_due_in_days: None,
            pending_batches: 0,
            next_batch_in_days: None,
        }
    }
}

impl TopicFlashcardSchedule {
    pub fn hint(&self) -> String {
        if self.total_cards == 0 && self.pending_batches == 0 {
            return String::new();
        }

        let mut parts = Vec::new();
        if self.due_now > 0 {
            let noun = if self.due_now == 1 {
                "flashcard"
            } else {
                "flashcards"
            };
            parts.push(format!("{} {noun} ready now in Study", self.due_now));
        } else if self.new_count > 0 {
            parts.push(format!(
                "{} new flashcard{} ready in Study now",
                self.new_count,
                if self.new_count == 1 { "" } else { "s" }
            ));
        }

        if let Some(days) = self.next_batch_in_days.or(self.next_due_in_days) {
            let batch_label = match days {
                0 => "next batch due today".into(),
                1 => "next batch in 1 day".into(),
                _ => format!("next batch in {days} days"),
            };
            parts.push(batch_label);
        } else if self.pending_batches > 0 && parts.is_empty() {
            parts.push(format!(
                "{} staged flashcard batch{} after practice",
                self.pending_batches,
                if self.pending_batches == 1 { "" } else { "es" }
            ));
        }

        parts.join(" · ")
    }
}

pub fn gre_deck_flashcard_schedule(col: &mut Collection) -> Result<TopicFlashcardSchedule> {
    topic_flashcard_schedule_inner(col, None)
}

pub fn topic_flashcard_schedule(col: &mut Collection, topic_id: &str) -> Result<TopicFlashcardSchedule> {
    topic_flashcard_schedule_inner(col, Some(topic_id))
}

fn topic_flashcard_schedule_inner(
    col: &mut Collection,
    topic_id: Option<&str>,
) -> Result<TopicFlashcardSchedule> {
    let guard = col.search_cards_into_table(gre_deck_search_str(), SortMode::NoOrder)?;
    let timing = guard.col.timing_today()?;
    let learn_cutoff = timing.now.0 as u32 + guard.col.learn_ahead_secs();
    let cards = guard.col.storage.cards_with_tags_for_searched_cards()?;

    let mut total_cards = 0u32;
    let mut due_now = 0u32;
    let mut new_count = 0u32;
    let mut next_due_in_days: Option<u32> = None;

    for entry in cards {
        if let Some(topic_id) = topic_id {
            if !card_belongs_to_topic(&entry.tags, topic_id) {
                continue;
            }
        }
        total_cards += 1;
        let card = &entry.card;
        if (card.queue as i8) < 0 {
            continue;
        }

        match card.ctype {
            CardType::New => {
                new_count += 1;
                due_now += 1;
            }
            CardType::Learn | CardType::Relearn => {
                if card.original_or_current_due() <= learn_cutoff as i32 {
                    due_now += 1;
                } else {
                    update_next_due(&mut next_due_in_days, 0);
                }
            }
            CardType::Review => {
                let days_remaining =
                    card.original_or_current_due() as i32 - timing.days_elapsed as i32;
                if days_remaining <= 0 {
                    due_now += 1;
                } else {
                    update_next_due(&mut next_due_in_days, days_remaining as u32);
                }
            }
        }
    }

    Ok(TopicFlashcardSchedule {
        total_cards,
        due_now,
        new_count,
        next_due_in_days: if due_now > 0 {
            None
        } else {
            next_due_in_days
        },
        pending_batches: 0,
        next_batch_in_days: None,
    })
}

fn update_next_due(current: &mut Option<u32>, days: u32) {
    *current = Some(current.map_or(days, |existing| existing.min(days)));
}

fn card_belongs_to_topic(tags: &[String], topic_id: &str) -> bool {
    tags.iter().any(|tag| tag_in_topic_tree(tag, topic_id))
}

fn tag_in_topic_tree(tag: &str, topic_id: &str) -> bool {
    if !tag.starts_with(TOPIC_TAG_PREFIX) {
        return false;
    }
    tag == topic_id || tag.starts_with(&format!("{topic_id}::"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hint_prefers_due_now() {
        let schedule = TopicFlashcardSchedule {
            total_cards: 3,
            due_now: 2,
            new_count: 1,
            next_due_in_days: Some(4),
            pending_batches: 1,
            next_batch_in_days: Some(4),
        };
        assert!(schedule.hint().contains("ready now"));
        assert!(schedule.hint().contains("next batch"));
    }

    #[test]
    fn hint_shows_days_until_review() {
        let schedule = TopicFlashcardSchedule {
            total_cards: 2,
            due_now: 0,
            new_count: 0,
            next_due_in_days: Some(3),
            pending_batches: 0,
            next_batch_in_days: Some(3),
        };
        assert_eq!(schedule.hint(), "next batch in 3 days");
    }
}
