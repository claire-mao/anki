// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Exam-length practice bank targets and automatic top-up from template
//! generation.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::error::Result;
use crate::gre_atlas::domain::GreCatalog;
use crate::gre_atlas::domain::GreSection;
use crate::gre_atlas::domain::TopicDef;
use crate::gre_atlas::questions::ai_gen::generate_question_for_topic_variant;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::questions::foundation::foundation_question_by_id;
use crate::gre_atlas::questions::foundation::FoundationQuestion;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::gre_atlas::storage::GreAtlasStorage;
use crate::gre_atlas::storage::StoredQuestion;
use crate::timestamp::TimestampSecs;

/// MCQ count in one scored GRE Quantitative Reasoning section.
pub const EXAM_QUANT_MCQ: u32 = 27;
/// MCQ count in one scored GRE Verbal Reasoning section.
pub const EXAM_VERBAL_MCQ: u32 = 27;
/// Prompt count per AWA task type on the real exam.
pub const EXAM_AWA_PER_TOPIC: u32 = 2;

/// Offline practice bank size targets (foundation seeds + template variants, no LLM).
pub const TARGET_PRACTICE_BANK_QUANT: u32 = 100;
pub const TARGET_PRACTICE_BANK_VERBAL: u32 = 100;
pub const TARGET_PRACTICE_BANK_AWA: u32 = 60;
pub const PRACTICE_BANK_QUESTION_TOTAL: u32 =
    TARGET_PRACTICE_BANK_QUANT + TARGET_PRACTICE_BANK_VERBAL + TARGET_PRACTICE_BANK_AWA;
/// Minimum template-backed items per catalog leaf (21 leaves × 5 = 105 floor).
pub const MIN_PRACTICE_BANK_PER_TOPIC: u32 = 5;
/// Questions returned to the practice UI in one session bootstrap.
pub const PRACTICE_QUESTION_LIST_LIMIT: u32 = PRACTICE_BANK_QUESTION_TOTAL;

/// Total target practice items across all scored GRE sections (Quant + Verbal +
/// AWA prompts).
pub fn exam_bank_question_count() -> u32 {
    GreCatalog::leaf_topics().map(target_count_for_topic).sum()
}

/// Target question count for a catalog leaf, proportional to official exam
/// weights within its section and scaled to [`TARGET_PRACTICE_BANK_*`].
pub fn target_count_for_topic(topic: &TopicDef) -> u32 {
    let section_target = match topic.section {
        GreSection::QuantitativeReasoning => TARGET_PRACTICE_BANK_QUANT,
        GreSection::VerbalReasoning => TARGET_PRACTICE_BANK_VERBAL,
        GreSection::AnalyticalWriting => TARGET_PRACTICE_BANK_AWA,
    };
    allocated_targets_for_section(topic.section, section_target)
        .into_iter()
        .find(|(leaf, _)| leaf.id == topic.id)
        .map(|(_, count)| count)
        .unwrap_or(MIN_PRACTICE_BANK_PER_TOPIC)
}

/// Split a section target across its leaf topics by exam weight. Uses largest-
/// remainder rounding so per-topic allocations sum exactly to `section_target`.
fn allocated_targets_for_section(
    section: GreSection,
    section_target: u32,
) -> Vec<(&'static TopicDef, u32)> {
    let leaves: Vec<_> = GreCatalog::leaf_topics_for_section(section).into_iter().collect();
    let weight_sum: f32 = leaves.iter().map(|leaf| leaf.exam_weight).sum();
    if weight_sum <= f32::EPSILON {
        return leaves
            .into_iter()
            .map(|leaf| (leaf, MIN_PRACTICE_BANK_PER_TOPIC))
            .collect();
    }

    let mut rows: Vec<(&'static TopicDef, u32, f32)> = leaves
        .iter()
        .map(|leaf| {
            let exact = (leaf.exam_weight / weight_sum) * section_target as f32;
            let base = exact.floor() as u32;
            let fraction = exact - base as f32;
            (*leaf, base, fraction)
        })
        .collect();

    let allocated: u32 = rows.iter().map(|(_, base, _)| base).sum();
    let mut remainder = section_target.saturating_sub(allocated);
    rows.sort_by(|a, b| {
        b.2
            .partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.id.cmp(b.0.id))
    });
    for row in &mut rows {
        if remainder == 0 {
            break;
        }
        row.1 += 1;
        remainder -= 1;
    }

    rows.into_iter()
        .map(|(leaf, count, _)| (leaf, count.max(MIN_PRACTICE_BANK_PER_TOPIC)))
        .collect()
}

/// Canonical offline practice bank: one deterministic list derived from bundled
/// foundation seeds using [`target_count_for_topic`] per catalog leaf.
pub fn load_practice_bank() -> Vec<FoundationQuestion> {
    let mut by_topic: HashMap<String, Vec<FoundationQuestion>> = HashMap::new();
    for question in crate::gre_atlas::questions::foundation::load_foundation_bank() {
        by_topic
            .entry(question.topic.clone())
            .or_default()
            .push(question);
    }
    for questions in by_topic.values_mut() {
        questions.sort_by(|left, right| left.id.cmp(&right.id));
    }

    let mut bank = Vec::with_capacity(PRACTICE_BANK_QUESTION_TOTAL as usize);
    for leaf in GreCatalog::leaf_topics() {
        let target = target_count_for_topic(leaf) as usize;
        let candidates = by_topic.get(leaf.id).map(Vec::as_slice).unwrap_or(&[]);
        assert!(
            candidates.len() >= target,
            "practice bank build: {} has {} foundation rows, want {target}",
            leaf.id,
            candidates.len()
        );
        bank.extend(candidates.iter().take(target).cloned());
    }
    validate_practice_bank(&bank);
    bank
}

/// Stable ids for [`load_practice_bank`]; used to filter DB rows for learners.
pub fn practice_bank_ids() -> &'static HashSet<String> {
    static IDS: OnceLock<HashSet<String>> = OnceLock::new();
    IDS.get_or_init(|| load_practice_bank().into_iter().map(|q| q.id).collect())
}

/// Fail loudly when canonical counts or membership rules are violated.
pub fn validate_practice_bank(bank: &[FoundationQuestion]) {
    let quant_count = bank.iter().filter(|q| q.section == "quant").count();
    let verbal_count = bank.iter().filter(|q| q.section == "verbal").count();
    let awa_count = bank.iter().filter(|q| q.section == "awa").count();
    let total_count = bank.len();
    assert_eq!(
        quant_count,
        TARGET_PRACTICE_BANK_QUANT as usize,
        "practice bank quant count"
    );
    assert_eq!(
        verbal_count,
        TARGET_PRACTICE_BANK_VERBAL as usize,
        "practice bank verbal count"
    );
    assert_eq!(
        awa_count,
        TARGET_PRACTICE_BANK_AWA as usize,
        "practice bank awa count"
    );
    assert_eq!(
        total_count,
        PRACTICE_BANK_QUESTION_TOTAL as usize,
        "practice bank total count"
    );
    assert_eq!(
        quant_count + verbal_count + awa_count,
        total_count,
        "practice bank section partition"
    );

    let mut ids = HashSet::with_capacity(total_count);
    for question in bank {
        assert!(
            ids.insert(question.id.clone()),
            "duplicate practice bank id: {}",
            question.id
        );
        assert!(
            matches!(question.section.as_str(), "quant" | "verbal" | "awa"),
            "{}: invalid section {:?}",
            question.id,
            question.section
        );
    }
}

/// Ensure every canonical practice-bank row exists and is listable at open time.
pub fn assert_practice_bank_listable(storage: &GreAtlasStorage) -> Result<()> {
    for question in load_practice_bank() {
        storage.insert_foundation_question_if_missing(&question)?;
    }
    let listed = storage.list_practice_bank_questions("", PRACTICE_QUESTION_LIST_LIMIT)?;
    validate_practice_bank_stored(&listed)
}

fn validate_practice_bank_stored(questions: &[StoredQuestion]) -> Result<()> {
    let quant_count = questions
        .iter()
        .filter(|q| q.section == "quant")
        .count();
    let verbal_count = questions
        .iter()
        .filter(|q| q.section == "verbal")
        .count();
    let awa_count = questions.iter().filter(|q| q.section == "awa").count();
    let total_count = questions.len();
    if quant_count != TARGET_PRACTICE_BANK_QUANT as usize {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "practice bank quant count {quant_count}, want {}",
                TARGET_PRACTICE_BANK_QUANT
            )),
        });
    }
    if verbal_count != TARGET_PRACTICE_BANK_VERBAL as usize {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "practice bank verbal count {verbal_count}, want {}",
                TARGET_PRACTICE_BANK_VERBAL
            )),
        });
    }
    if awa_count != TARGET_PRACTICE_BANK_AWA as usize {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "practice bank awa count {awa_count}, want {}",
                TARGET_PRACTICE_BANK_AWA
            )),
        });
    }
    if total_count != PRACTICE_BANK_QUESTION_TOTAL as usize {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "practice bank total count {total_count}, want {}",
                PRACTICE_BANK_QUESTION_TOTAL
            )),
        });
    }
    if quant_count + verbal_count + awa_count != total_count {
        return Err(crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!(
                "practice bank sections do not partition total ({quant_count}+{verbal_count}+{awa_count} != {total_count})"
            )),
        });
    }

    let mut ids = HashSet::with_capacity(total_count);
    for question in questions {
        if !ids.insert(question.id.clone()) {
            return Err(crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!(
                    "duplicate practice bank id listed: {}",
                    question.id
                )),
            });
        }
        if !practice_bank_ids().contains(&question.id) {
            return Err(crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!(
                    "unexpected question in practice bank listing: {}",
                    question.id
                )),
            });
        }
    }
    Ok(())
}

/// Insert any bundled foundation rows missing from the sidecar DB.
pub fn ensure_foundation_questions_present(storage: &GreAtlasStorage) -> Result<u32> {
    let mut added = 0u32;
    for question in crate::gre_atlas::questions::foundation::load_foundation_bank() {
        if storage.insert_foundation_question_if_missing(&question)? {
            added += 1;
        }
    }
    Ok(added)
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

/// Replace sync FK placeholder rows with bundled foundation content when ids
/// match. Safe to run on every storage open.
pub fn repair_sync_question_stubs(storage: &GreAtlasStorage) -> Result<u32> {
    let mut repaired = 0u32;
    for id in storage.list_sync_question_stub_ids()? {
        if let Some(question) = foundation_question_by_id(&id) {
            if storage.replace_sync_question_stub_from_foundation(&question)? {
                repaired += 1;
            }
        }
    }
    Ok(repaired)
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
    fn load_practice_bank_has_exact_section_totals() {
        let bank = load_practice_bank();
        validate_practice_bank(&bank);
    }

    #[test]
    fn exam_targets_cover_every_leaf() {
        assert_eq!(GreCatalog::leaf_topics().count(), 21);
        assert_eq!(exam_bank_question_count(), PRACTICE_BANK_QUESTION_TOTAL);
        let mut quant = 0u32;
        let mut verbal = 0u32;
        let mut awa = 0u32;
        for leaf in GreCatalog::leaf_topics() {
            let target = target_count_for_topic(leaf);
            assert!(
                target >= MIN_PRACTICE_BANK_PER_TOPIC,
                "missing target for {}",
                leaf.id
            );
            match leaf.section {
                GreSection::QuantitativeReasoning => quant += target,
                GreSection::VerbalReasoning => verbal += target,
                GreSection::AnalyticalWriting => awa += target,
            }
        }
        assert_eq!(quant, TARGET_PRACTICE_BANK_QUANT);
        assert_eq!(verbal, TARGET_PRACTICE_BANK_VERBAL);
        assert_eq!(awa, TARGET_PRACTICE_BANK_AWA);
    }

    #[test]
    fn assert_practice_bank_listable_on_fresh_collection() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        assert_practice_bank_listable(&storage)?;
        Ok(())
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
    fn repair_sync_question_stub_from_foundation() -> Result<()> {
        use crate::gre_atlas::questions::foundation::foundation_question_by_id;
        use crate::gre_atlas::storage::SYNC_QUESTION_STUB_STEM;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let foundation = foundation_question_by_id("gre-foundation-awa-arg-001")
            .expect("awa foundation question");
        storage.delete_question(&foundation.id)?;

        let session = storage.create_session("practice")?;
        storage.push_changes(&[crate::gre_atlas::storage::SyncAttemptRow {
            id: 0,
            question_id: foundation.id.clone(),
            topic: foundation.topic.clone(),
            difficulty: foundation.difficulty,
            answered_at_secs: TimestampSecs(1_700_000_400),
            answer: "A".into(),
            correct: true,
            response_time_ms: 500,
            confidence: None,
            session_id: Some(session.id),
            usn: 1,
            mtime_secs: TimestampSecs(1_700_000_400),
        }])?;

        assert!(storage
            .get_question(&foundation.id)?
            .is_some_and(|q| q.stem == SYNC_QUESTION_STUB_STEM));
        assert!(storage
            .list_questions(&foundation.topic, 100)?
            .iter()
            .all(|q| q.id != foundation.id));

        let repaired = repair_sync_question_stubs(&storage)?;
        assert_eq!(repaired, 1);
        let restored = storage.get_question(&foundation.id)?.expect("restored question");
        assert_ne!(restored.stem, SYNC_QUESTION_STUB_STEM);
        assert!(!restored.choices.is_empty());
        assert!(storage
            .list_questions(&foundation.topic, 100)?
            .iter()
            .any(|q| q.id == foundation.id));
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
