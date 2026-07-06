// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Template GRE flashcards derived from the bundled foundation practice bank.

use crate::gre_atlas::questions::foundation::load_foundation_bank;
use crate::gre_atlas::questions::foundation::FoundationQuestion;

pub const FLASHCARDS_PER_TOPIC: usize = 9;
pub const FLASHCARDS_PER_BATCH: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationFlashcard {
    pub question_id: String,
    pub topic: String,
    pub front: String,
    pub back: String,
}

pub fn foundation_flashcards_for_topic(topic_id: &str) -> Vec<FoundationFlashcard> {
    let mut questions: Vec<_> = load_foundation_bank()
        .into_iter()
        .filter(|question| question.topic == topic_id)
        .collect();
    questions.sort_by(|a, b| a.id.cmp(&b.id));
    questions
        .into_iter()
        .take(FLASHCARDS_PER_TOPIC)
        .map(flashcard_from_question)
        .collect()
}

fn flashcard_from_question(question: FoundationQuestion) -> FoundationFlashcard {
    FoundationFlashcard {
        question_id: question.id.clone(),
        topic: question.topic.clone(),
        front: truncate_stem(question.stem_text()),
        back: flashcard_back(&question),
    }
}

fn flashcard_back(question: &FoundationQuestion) -> String {
    let explanation = first_sentence(&question.explanation);
    if explanation.is_empty() {
        question.correct_answer.clone()
    } else {
        format!("{}\n\n{}", question.correct_answer, explanation)
    }
}

fn truncate_stem(stem: &str) -> String {
    const MAX_LEN: usize = 240;
    let trimmed = stem.trim();
    if trimmed.chars().count() <= MAX_LEN {
        return trimmed.to_string();
    }
    trimmed
        .chars()
        .take(MAX_LEN.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn first_sentence(text: &str) -> String {
    let trimmed = text.split("<!-- meta:").next().unwrap_or(text).trim();
    trimmed
        .split_once(". ")
        .map(|(first, _)| format!("{first}."))
        .unwrap_or_else(|| trimmed.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::domain::GreCatalog;

    #[test]
    fn foundation_flashcards_cover_leaf_topics() {
        let leaf = GreCatalog::leaf_topics().next().expect("catalog leaf");
        let cards = foundation_flashcards_for_topic(leaf.id);
        assert!(
            !cards.is_empty(),
            "expected template flashcards for {}",
            leaf.id
        );
        assert!(cards.iter().all(|card| card.topic == leaf.id));
        assert!(cards.iter().all(|card| !card.front.is_empty()));
        assert!(cards.iter().all(|card| !card.back.is_empty()));
    }
}
