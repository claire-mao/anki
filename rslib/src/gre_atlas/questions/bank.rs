// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Exam-length practice bank targets and automatic top-up from template generation.

use crate::error::Result;
use crate::gre_atlas::domain::GreCatalog;
use crate::gre_atlas::domain::GreSection;
use crate::gre_atlas::domain::TopicDef;
use crate::gre_atlas::questions::ai_gen::generate_question_for_topic_variant;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::gre_atlas::storage::GreAtlasStorage;
use crate::timestamp::TimestampSecs;

/// MCQ count in one scored GRE Quantitative Reasoning section.
pub const EXAM_QUANT_MCQ: u32 = 27;
/// MCQ count in one scored GRE Verbal Reasoning section.
pub const EXAM_VERBAL_MCQ: u32 = 27;
/// Prompt count per AWA task type on the real exam.
pub const EXAM_AWA_PER_TOPIC: u32 = 2;

/// Total target practice items across all scored GRE sections (Quant + Verbal +
/// AWA prompts).
pub fn exam_bank_question_count() -> u32 {
    GreCatalog::leaf_topics()
        .map(target_count_for_topic)
        .sum()
}

/// Target question count for a catalog leaf, proportional to official exam
/// weights.
pub fn target_count_for_topic(topic: &TopicDef) -> u32 {
    match topic.section {
        GreSection::AnalyticalWriting => EXAM_AWA_PER_TOPIC,
        GreSection::QuantitativeReasoning => {
            ((topic.exam_weight * EXAM_QUANT_MCQ as f32).round() as u32).max(1)
        }
        GreSection::VerbalReasoning => {
            ((topic.exam_weight * EXAM_VERBAL_MCQ as f32).round() as u32).max(1)
        }
    }
}

/// Remove generated questions whose correct answer is not among their choices.
/// Seed questions are preserved; invalid rows are deleted so the bank can be
/// refilled.
pub fn purge_invalid_questions(storage: &GreAtlasStorage) -> Result<u32> {
    let mut removed = 0u32;
    let questions = storage.list_questions("", u32::MAX)?;
    for question in questions {
        if question.source_name.as_deref() == Some(GENERATION_SOURCE_NAME)
            && !correct_answer_in_choices(&question.correct_answer, &question.choices)
        {
            storage.delete_question(&question.id)?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn correct_answer_in_choices(correct: &str, choices: &[String]) -> bool {
    crate::gre_atlas::questions::variants::correct_answer_in_choices(correct, choices)
}

/// Generate template questions for any leaf topic below its exam-length target.
/// Safe to call on every storage open; existing rows are preserved.
pub fn ensure_exam_length_bank(storage: &GreAtlasStorage) -> Result<u32> {
    let counts = storage.question_counts_by_topic()?;
    let base_now = TimestampSecs::now().0;
    let mut added = 0u32;
    let mut offset = 0i64;

    for leaf in GreCatalog::leaf_topics() {
        let target = target_count_for_topic(leaf);
        let have = counts.get(leaf.id).copied().unwrap_or(0);
        if have >= target {
            continue;
        }

        let mut variant = have;
        let mut attempts = 0u32;
        while variant < target && attempts < target.saturating_mul(3) {
            let now = TimestampSecs(base_now + offset);
            offset += 1;
            attempts += 1;

            match generate_question_for_topic_variant(leaf.id, variant, now) {
                GenerationOutcome::Accepted(draft) => {
                    if storage.insert_generated_question_if_new(&draft)? {
                        added += 1;
                        variant += 1;
                    }
                }
                GenerationOutcome::Rejected { .. } => {
                    variant += 1;
                }
            }
        }
    }

    Ok(added)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;

    #[test]
    fn exam_targets_cover_every_leaf() {
        assert_eq!(GreCatalog::leaf_topics().count(), 21);
        assert!(exam_bank_question_count() >= 54);
        for leaf in GreCatalog::leaf_topics() {
            assert!(
                target_count_for_topic(leaf) >= 1,
                "missing target for {}",
                leaf.id
            );
        }
    }

    #[test]
    fn ensure_exam_length_bank_tops_up_fresh_collection() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;

        let total = storage.question_count()?;
        assert!(
            total >= exam_bank_question_count() as u64,
            "bank should reach exam length, got {total}"
        );

        for leaf in GreCatalog::leaf_topics() {
            let count = storage
                .question_counts_by_topic()?
                .get(leaf.id)
                .copied()
                .unwrap_or(0);
            assert!(
                count >= target_count_for_topic(leaf),
                "{} has {count}, want {}",
                leaf.id,
                target_count_for_topic(leaf)
            );
        }
        Ok(())
    }

    #[test]
    fn purge_invalid_questions_removes_bad_generated_rows() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let before = storage.question_count()?;

        storage.insert_generated_question(
            &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft {
                id: "ai-test-invalid".into(),
                topic: "gre::quant::algebra::linear".into(),
                section: "quant".into(),
                format: "mcq".into(),
                stem: "Invalid test question".into(),
                choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
                correct_answer: "99".into(),
                explanation: "broken".into(),
                difficulty: Some(0.5),
                confidence: 0.9,
                attribution: crate::gre_atlas::questions::ai_gen::QuestionAttribution {
                    source_name: GENERATION_SOURCE_NAME.into(),
                    source_section: "test".into(),
                    generated_at_secs: 1,
                },
            },
        )?;
        assert_eq!(storage.question_count()?, before + 1);

        let removed = purge_invalid_questions(&storage)?;
        assert_eq!(removed, 1);
        assert_eq!(storage.question_count()?, before);
        assert!(storage.get_question("ai-test-invalid")?.is_none());
        Ok(())
    }

    #[test]
    fn purge_invalid_questions_removes_rows_with_existing_attempts() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let before = storage.question_count()?;
        let invalid_id = "ai-test-invalid-with-attempts";

        storage.insert_generated_question(
            &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft {
                id: invalid_id.into(),
                topic: "gre::quant::algebra::linear".into(),
                section: "quant".into(),
                format: "mcq".into(),
                stem: "Invalid test question".into(),
                choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
                correct_answer: "99".into(),
                explanation: "broken".into(),
                difficulty: Some(0.5),
                confidence: 0.9,
                attribution: crate::gre_atlas::questions::ai_gen::QuestionAttribution {
                    source_name: GENERATION_SOURCE_NAME.into(),
                    source_section: "test".into(),
                    generated_at_secs: 1,
                },
            },
        )?;
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            invalid_id,
            "gre::quant::algebra::linear",
            Some(0.5),
            "1",
            false,
            900,
            None,
            Some(&session.id),
        )?;

        let removed = purge_invalid_questions(&storage)?;
        assert_eq!(removed, 1);
        assert_eq!(storage.question_count()?, before);
        assert!(storage.get_question(invalid_id)?.is_none());
        let attempts = storage.recent_attempts("", 100)?;
        assert!(!attempts.iter().any(|a| a.question_id == invalid_id));
        Ok(())
    }

    #[test]
    fn open_purges_invalid_questions_with_attempts() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let invalid_id = "ai-test-invalid-on-reopen";

        storage.insert_generated_question(
            &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft {
                id: invalid_id.into(),
                topic: "gre::quant::algebra::linear".into(),
                section: "quant".into(),
                format: "mcq".into(),
                stem: "Invalid test question".into(),
                choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
                correct_answer: "99".into(),
                explanation: "broken".into(),
                difficulty: Some(0.5),
                confidence: 0.9,
                attribution: crate::gre_atlas::questions::ai_gen::QuestionAttribution {
                    source_name: GENERATION_SOURCE_NAME.into(),
                    source_section: "test".into(),
                    generated_at_secs: 1,
                },
            },
        )?;
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            invalid_id,
            "gre::quant::algebra::linear",
            Some(0.5),
            "1",
            false,
            900,
            None,
            Some(&session.id),
        )?;
        drop(storage);

        let storage = GreAtlasStorage::open(&col_path)?;
        assert!(storage.get_question(invalid_id)?.is_none());
        let attempts = storage.recent_attempts("", 100)?;
        assert!(!attempts.iter().any(|a| a.question_id == invalid_id));
        Ok(())
    }
}
