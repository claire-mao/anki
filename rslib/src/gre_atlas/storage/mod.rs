// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::error::Result;
use crate::timestamp::TimestampSecs;

/// Placeholder stem written by `ensure_question_for_attempt` when a synced
/// attempt arrives before its question row. These rows must never be served in
/// practice; merge/repair replaces them when the real question is available.
pub(crate) const SYNC_QUESTION_STUB_STEM: &str = "[synced practice row]";

pub(crate) fn is_sync_question_stub_stem(stem: &str) -> bool {
    stem == SYNC_QUESTION_STUB_STEM
}

mod sync_write;

pub(crate) use sync_write::sync_execute;
pub(crate) use sync_write::SyncFkContext;

pub(crate) const SCHEMA_VERSION: u32 = 6;

/// Sidecar SQLite filename beside the collection profile (GRE Atlas practice
/// data).
pub const GRE_ATLAS_DB_NAME: &str = "greatlas.db";
/// Legacy sidecar filename; migrated to [`GRE_ATLAS_DB_NAME`] on open when
/// present.
pub const LEGACY_BRAINLIFT_DB_NAME: &str = "brainlift.db";

mod sync_bundle;

pub use sync_bundle::ApplyBundleResult;
pub use sync_bundle::SyncBundle;
pub use sync_bundle::SyncPredictionRow;
pub use sync_bundle::SyncQuestionRow;
pub use sync_bundle::SyncSessionRow;
pub use sync_bundle::META_LAST_DOWNLOADED_USN;

#[derive(Debug, Clone)]
pub struct StoredTopicFlashcardBatch {
    pub topic: String,
    pub batch_index: u32,
    pub release_at_secs: TimestampSecs,
    pub card_ids: Vec<crate::card::CardId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTopicFlashcardSummary {
    pub pending_batches: u32,
    pub next_batch_in_days: Option<u32>,
}

fn decode_card_ids_json(card_ids_json: &str) -> Result<Vec<crate::card::CardId>> {
    let raw: Vec<i64> = serde_json::from_str(card_ids_json).map_err(|err| {
        crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!("card_ids_json: {err}")),
        }
    })?;
    Ok(raw.into_iter().map(crate::card::CardId).collect())
}

#[derive(Debug, Clone)]
pub struct StoredQuestion {
    pub id: String,
    pub topic: String,
    pub section: String,
    pub format: String,
    pub stem: String,
    pub choices: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    pub difficulty: Option<f32>,
    pub source_name: Option<String>,
    pub source_section: Option<String>,
    pub generated_at_secs: Option<i64>,
    pub generation_confidence: Option<f32>,
    /// Specific source excerpt/section id that grounded generation.
    pub source_document: Option<String>,
    /// Real model id (e.g. `gpt-4o-mini`) or `template_v1` for offline
    /// templates. `None` for legacy/foundation rows.
    pub model_version: Option<String>,
    /// `ai_generated` or `offline_template`. `None` for legacy/foundation rows.
    pub provenance: Option<String>,
    /// Grounding gate result, e.g. `approved` or `rejected_hallucination`.
    pub evaluation_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PracticeSession {
    pub id: String,
    pub started_at_secs: TimestampSecs,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct PerformanceAttemptEvalRow {
    pub id: i64,
    pub topic: String,
    pub correct: bool,
}

#[derive(Debug, Clone)]
pub struct PerformanceAttemptRow {
    pub question_id: String,
    pub topic: String,
    pub difficulty: Option<f32>,
    pub answered_at_secs: TimestampSecs,
    pub answer: String,
    pub correct: bool,
    pub response_time_ms: u32,
    pub confidence: Option<u32>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceAttemptChartRow {
    pub answered_at_secs: TimestampSecs,
    pub correct: bool,
}

#[derive(Debug, Clone)]
pub struct SyncAttemptRow {
    pub id: i64,
    pub question_id: String,
    pub topic: String,
    pub difficulty: Option<f32>,
    pub answered_at_secs: TimestampSecs,
    pub answer: String,
    pub correct: bool,
    pub response_time_ms: u32,
    pub confidence: Option<u32>,
    pub session_id: Option<String>,
    pub usn: i32,
    pub mtime_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub current_usn: i32,
    pub pending_upload_count: u32,
    pub last_modified_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub attempt_id: i64,
    pub reason: String,
    pub kept: SyncAttemptRow,
    pub rejected: SyncAttemptRow,
}

#[derive(Debug, Clone)]
pub struct PushChangesResult {
    pub current_usn: i32,
    pub applied_count: u32,
    pub conflicts: Vec<SyncConflict>,
}

impl From<anki_proto::brainlift::BrainLiftSyncAttempt> for SyncAttemptRow {
    fn from(value: anki_proto::brainlift::BrainLiftSyncAttempt) -> Self {
        SyncAttemptRow {
            id: value.id,
            question_id: value.question_id,
            topic: value.topic,
            difficulty: value.difficulty,
            answered_at_secs: TimestampSecs(value.answered_at_secs),
            answer: value.answer,
            correct: value.correct,
            response_time_ms: value.response_time_ms,
            confidence: value.confidence,
            session_id: value.session_id.filter(|s| !s.is_empty()),
            usn: value.usn,
            mtime_secs: TimestampSecs(value.mtime_secs),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadinessPredictionRow {
    pub id: i64,
    pub predicted_at_secs: TimestampSecs,
    pub projected_score: f32,
    pub projected_score_low: Option<f32>,
    pub projected_score_high: Option<f32>,
    pub memory_score: f32,
    pub performance_score: f32,
    pub coverage_ratio: f32,
    pub confidence_level: String,
    pub model_version: String,
    pub outcome_score: Option<f32>,
    pub outcome_observed_at_secs: Option<TimestampSecs>,
    pub outcome_memory_score: Option<f32>,
    pub outcome_performance_score: Option<f32>,
    pub practice_correct: Option<u32>,
    pub practice_total: Option<u32>,
}

pub struct GreAtlasStorage {
    db: Connection,
}

impl fmt::Debug for GreAtlasStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreAtlasStorage").finish_non_exhaustive()
    }
}

impl GreAtlasStorage {
    pub fn open(collection_path: &Path) -> Result<Self> {
        let db_path = gre_atlas_db_path(collection_path);
        migrate_legacy_db_filename(&db_path)?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(&db_path)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.pragma_update(None, "journal_mode", "wal")?;
        db.pragma_update(None, "foreign_keys", true)?;
        db.set_prepared_statement_cache_capacity(20);

        let storage = Self { db };
        storage.migrate(collection_path)?;
        storage.seed_questions_if_empty()?;
        crate::gre_atlas::questions::ensure_foundation_questions_present(&storage)?;
        crate::gre_atlas::questions::purge_invalid_questions(&storage)?;
        crate::gre_atlas::questions::repair_sync_question_stubs(&storage)?;
        crate::gre_atlas::questions::ensure_exam_length_bank(&storage)?;
        crate::gre_atlas::questions::assert_practice_bank_listable(&storage)?;
        Ok(storage)
    }

    fn migrate(&self, collection_path: &Path) -> Result<()> {
        self.db.execute_batch(include_str!("schema.sql"))?;
        let ver = self.schema_version()?;
        if ver == 0 {
            self.set_meta("schema_version", &SCHEMA_VERSION.to_string())?;
        } else if ver < SCHEMA_VERSION {
            self.upgrade_from(ver)?;
            self.set_meta("schema_version", &SCHEMA_VERSION.to_string())?;
        }
        let collection_path_str = collection_path.to_string_lossy();
        self.set_meta("collection_path", &collection_path_str)?;
        Ok(())
    }

    fn schema_version(&self) -> Result<u32> {
        let ver: Option<String> = self
            .db
            .query_row(
                "SELECT val FROM bl_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(ver.and_then(|s| s.parse().ok()).unwrap_or(0))
    }

    fn upgrade_from(&self, ver: u32) -> Result<()> {
        if ver < 2 {
            self.db.execute_batch(include_str!("upgrade_1_to_2.sql"))?;
        }
        if ver < 3 {
            self.db.execute_batch(include_str!("upgrade_2_to_3.sql"))?;
        }
        if ver < 4 {
            self.db.execute_batch(include_str!("upgrade_3_to_4.sql"))?;
        }
        if ver < 5 {
            self.db.execute_batch(include_str!("upgrade_4_to_5.sql"))?;
        }
        if ver < 6 {
            self.db.execute_batch(include_str!("upgrade_5_to_6.sql"))?;
        }
        Ok(())
    }

    fn set_meta(&self, key: &str, val: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO bl_meta (key, val) VALUES (?, ?)",
            params![key, val],
        )?;
        Ok(())
    }

    fn seed_questions_if_empty(&self) -> Result<()> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM bl_question", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(());
        }

        for question in crate::gre_atlas::questions::foundation::load_foundation_bank() {
            self.insert_foundation_question_if_missing(&question)?;
        }
        Ok(())
    }

    pub(crate) fn insert_foundation_question_if_missing(
        &self,
        question: &crate::gre_atlas::questions::foundation::FoundationQuestion,
    ) -> Result<bool> {
        if self.question_exists(&question.id)? {
            return Ok(false);
        }
        let choices = question.choice_list();
        let now = TimestampSecs::now().0;
        self.db.execute(
            "INSERT INTO bl_question
            (id, topic, section, format, stem, choices_json, correct_answer, explanation,
             difficulty, source_name, source_section, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
            params![
                question.id,
                question.topic,
                question.section,
                question.format,
                question.stem_text(),
                serde_json::to_string(choices).unwrap(),
                question.correct_answer,
                question.stored_explanation(),
                question.difficulty,
                question.source_name(),
                question.subtopic.as_deref().unwrap_or("foundation"),
                now,
            ],
        )?;
        Ok(true)
    }

    pub fn create_session(&self, source: &str) -> Result<PracticeSession> {
        let now = TimestampSecs::now();
        let id = new_session_id();
        let source = if source.is_empty() {
            "practice"
        } else {
            source
        };
        let usn = self.next_usn()?;
        self.db.execute(
            "INSERT INTO bl_session (id, started_at_secs, source, usn, mtime_secs)
             VALUES (?, ?, ?, ?, ?)",
            params![id, now.0, source, usn, now.0],
        )?;
        Ok(PracticeSession {
            id,
            started_at_secs: now,
            source: source.to_string(),
        })
    }

    pub fn session_exists(&self, session_id: &str) -> Result<bool> {
        self.db
            .query_row(
                "SELECT 1 FROM bl_session WHERE id = ?",
                [session_id],
                |_| Ok(()),
            )
            .optional()
            .map(|opt| opt.is_some())
            .map_err(Into::into)
    }

    pub fn question_exists(&self, question_id: &str) -> Result<bool> {
        self.db
            .query_row(
                "SELECT 1 FROM bl_question WHERE id = ?",
                [question_id],
                |_| Ok(()),
            )
            .optional()
            .map(|opt| opt.is_some())
            .map_err(Into::into)
    }

    pub(crate) fn question_stem_is_sync_stub(&self, question_id: &str) -> Result<bool> {
        Ok(self
            .db
            .query_row(
                "SELECT stem FROM bl_question WHERE id = ?",
                [question_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .is_some_and(|stem| is_sync_question_stub_stem(&stem)))
    }

    pub fn session_count(&self) -> Result<u64> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM bl_session", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    pub fn list_questions(&self, topic_prefix: &str, limit: u32) -> Result<Vec<StoredQuestion>> {
        let limit = limit.max(1) as i64;
        // Deprioritize questions the learner has already answered: order by attempt
        // count (least-answered first), then hardest-first by difficulty, then
        // least-recently answered. A stable `id` tie-break keeps ordering
        // deterministic across devices, which mobile/desktop FFI parity relies on.
        let mut stmt = self.db.prepare_cached(
            "SELECT q.id, q.topic, q.section, q.format, q.stem, q.choices_json, q.correct_answer,
                    q.explanation, q.difficulty, q.source_name, q.source_section,
                    q.generated_at_secs, q.generation_confidence, q.source_document,
                    q.model_version, q.provenance, q.evaluation_status
             FROM bl_question q
             LEFT JOIN (
                 SELECT question_id, COUNT(*) AS attempts, MAX(answered_at_secs) AS last_answered
                 FROM bl_performance_attempt
                 GROUP BY question_id
             ) a ON a.question_id = q.id
             WHERE (?1 = '' OR q.topic = ?1 OR q.topic LIKE ?1 || '::%')
               AND q.stem != ?3
               -- Rejected AI candidates must never reach learners; legacy/
               -- foundation/approved rows have NULL or non-'rejected' status.
               AND (q.evaluation_status IS NULL OR q.evaluation_status NOT LIKE 'rejected%')
             ORDER BY COALESCE(a.attempts, 0) ASC,
                      CASE WHEN COALESCE(q.difficulty, 0.5) >= 0.5 THEN 0 ELSE 1 END ASC,
                      COALESCE(q.difficulty, 0.5) DESC,
                      COALESCE(a.last_answered, 0) ASC,
                      q.id ASC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(
            params![topic_prefix, limit, SYNC_QUESTION_STUB_STEM],
            row_to_stored_question,
        )?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// List canonical practice-bank questions (exactly 260 offline items).
    pub fn list_practice_bank_questions(
        &self,
        topic_prefix: &str,
        limit: u32,
    ) -> Result<Vec<StoredQuestion>> {
        let ids = crate::gre_atlas::questions::practice_bank_ids();
        Ok(self
            .list_questions(topic_prefix, u32::MAX)?
            .into_iter()
            .filter(|question| ids.contains(&question.id))
            .take(limit.max(1) as usize)
            .collect())
    }

    pub fn get_question(&self, question_id: &str) -> Result<Option<StoredQuestion>> {
        self.db
            .query_row(
                "SELECT id, topic, section, format, stem, choices_json, correct_answer,
                        explanation, difficulty, source_name, source_section, generated_at_secs,
                        generation_confidence, source_document, model_version, provenance,
                        evaluation_status
                 FROM bl_question WHERE id = ?",
                [question_id],
                row_to_stored_question,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn question_count(&self) -> Result<u64> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM bl_question", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    pub fn question_counts_by_topic(&self) -> Result<HashMap<String, u32>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT topic, COUNT(*) FROM bl_question
             WHERE stem != ?1
             GROUP BY topic",
        )?;
        let rows = stmt.query_map([SYNC_QUESTION_STUB_STEM], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;
        rows.collect::<std::result::Result<HashMap<_, _>, _>>()
            .map_err(Into::into)
    }

    pub fn topic_attempt_count(&self, topic_id: &str) -> Result<u32> {
        self.db
            .query_row(
                "SELECT COUNT(*) FROM bl_performance_attempt WHERE topic = ?",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    pub fn topic_flashcard_batches_exist(&self, topic_id: &str) -> Result<bool> {
        let count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM bl_topic_flashcard_batch WHERE topic = ?",
            [topic_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn insert_topic_flashcard_batch(
        &self,
        topic_id: &str,
        batch_index: u32,
        release_at_secs: TimestampSecs,
        card_ids: &[crate::card::CardId],
        released: bool,
    ) -> Result<()> {
        let card_ids_json = serde_json::to_string(
            &card_ids
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let now = TimestampSecs::now().0;
        self.db.execute(
            "INSERT INTO bl_topic_flashcard_batch
             (topic, batch_index, release_at_secs, card_ids_json, released, usn, mtime_secs)
             VALUES (?, ?, ?, ?, ?, 0, ?)",
            params![
                topic_id,
                batch_index,
                release_at_secs.0,
                card_ids_json,
                released as i32,
                now,
            ],
        )?;
        Ok(())
    }

    /// Pull forward unreleased batches that were scheduled with a longer interval.
    pub fn reconcile_flashcard_batch_intervals(&self, interval_days: u32) -> Result<()> {
        let interval_secs = (interval_days as i64) * 86_400;
        if interval_secs <= 0 {
            return Ok(());
        }
        let mut stmt = self.db.prepare_cached(
            "SELECT topic FROM bl_topic_flashcard_batch
             WHERE released = 0
             GROUP BY topic",
        )?;
        let topics = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for topic in topics {
            let anchor: Option<i64> = self
                .db
                .query_row(
                    "SELECT MIN(release_at_secs) FROM bl_topic_flashcard_batch
                     WHERE topic = ? AND batch_index = 0",
                    [&topic],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();
            let Some(anchor) = anchor else {
                continue;
            };
            self.db.execute(
                "UPDATE bl_topic_flashcard_batch
                 SET release_at_secs = ? + (? * batch_index),
                     mtime_secs = ?
                 WHERE topic = ? AND released = 0 AND batch_index > 0
                   AND release_at_secs > ? + (? * batch_index)",
                params![
                    anchor,
                    interval_secs,
                    TimestampSecs::now().0,
                    topic,
                    anchor,
                    interval_secs,
                ],
            )?;
        }
        Ok(())
    }

    pub fn pending_topic_flashcard_batches(
        &self,
        now: TimestampSecs,
    ) -> Result<Vec<StoredTopicFlashcardBatch>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT topic, batch_index, release_at_secs, card_ids_json
             FROM bl_topic_flashcard_batch
             WHERE released = 0 AND release_at_secs <= ?
             ORDER BY release_at_secs, batch_index",
        )?;
        let rows = stmt.query_map([now.0], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        rows.into_iter()
            .map(|row| {
                let (topic, batch_index, release_at_secs, card_ids_json) = row?;
                Ok(StoredTopicFlashcardBatch {
                    topic,
                    batch_index,
                    release_at_secs: TimestampSecs(release_at_secs),
                    card_ids: decode_card_ids_json(&card_ids_json)?,
                })
            })
            .collect()
    }

    pub fn pending_topic_flashcard_batch_summary(
        &self,
        topic_id: &str,
        now: TimestampSecs,
    ) -> Result<PendingTopicFlashcardSummary> {
        let mut stmt = self.db.prepare_cached(
            "SELECT release_at_secs FROM bl_topic_flashcard_batch
             WHERE topic = ? AND released = 0 AND release_at_secs > ?
             ORDER BY release_at_secs
             LIMIT 1",
        )?;
        let pending_batches: u32 = self.db.query_row(
            "SELECT COUNT(*) FROM bl_topic_flashcard_batch
             WHERE topic = ? AND released = 0 AND release_at_secs > ?",
            params![topic_id, now.0],
            |row| row.get(0),
        )?;
        let next_release_at: Option<i64> = stmt
            .query_row(params![topic_id, now.0], |row| row.get(0))
            .optional()?;
        let next_batch_in_days = next_release_at.map(|release_at| {
            ((release_at - now.0).max(0) as u32).div_ceil(86_400)
        });
        Ok(PendingTopicFlashcardSummary {
            pending_batches,
            next_batch_in_days,
        })
    }

    pub fn mark_topic_flashcard_batch_released(
        &self,
        topic_id: &str,
        batch_index: u32,
    ) -> Result<()> {
        let now = TimestampSecs::now().0;
        self.db.execute(
            "UPDATE bl_topic_flashcard_batch
             SET released = 1, mtime_secs = ?
             WHERE topic = ? AND batch_index = ?",
            params![now, topic_id, batch_index],
        )?;
        Ok(())
    }

    pub(crate) fn list_sync_question_stub_ids(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare_cached("SELECT id FROM bl_question WHERE stem = ?")?;
        let rows = stmt.query_map([SYNC_QUESTION_STUB_STEM], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(crate) fn replace_sync_question_stub_from_foundation(
        &self,
        question: &crate::gre_atlas::questions::foundation::FoundationQuestion,
    ) -> Result<bool> {
        let choices = question.choice_list();
        let usn = self.next_usn()?;
        let now = TimestampSecs::now().0;
        let updated = self.db.execute(
            "UPDATE bl_question
             SET topic = ?, section = ?, format = ?, stem = ?, choices_json = ?,
                 correct_answer = ?, explanation = ?, difficulty = ?,
                 source_name = ?, source_section = ?, usn = ?, mtime_secs = ?
             WHERE id = ? AND stem = ?",
            params![
                question.topic,
                question.section,
                question.format,
                question.stem_text(),
                serde_json::to_string(choices).unwrap(),
                question.correct_answer,
                question.stored_explanation(),
                question.difficulty,
                question.source_name(),
                question.subtopic.as_deref().unwrap_or("foundation"),
                usn,
                now,
                question.id,
                SYNC_QUESTION_STUB_STEM,
            ],
        )?;
        Ok(updated > 0)
    }

    pub fn insert_generated_question_if_new(
        &self,
        draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
    ) -> Result<bool> {
        self.insert_generated_question_if_new_with_meta(draft, &default_template_metadata(draft))
    }

    /// Insert a generated question with explicit provenance/eval metadata,
    /// skipping if a row with the same id already exists.
    pub fn insert_generated_question_if_new_with_meta(
        &self,
        draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
        meta: &crate::gre_atlas::questions::metadata::QuestionMetadata,
    ) -> Result<bool> {
        if self.question_exists(&draft.id)? {
            return Ok(false);
        }
        let now = TimestampSecs::now().0;
        let usn = self.next_usn()?;
        let rows = self.db.execute(
            "INSERT OR IGNORE INTO bl_question
            (id, topic, section, format, stem, choices_json, correct_answer, explanation,
             difficulty, source_name, source_section, generated_at_secs, generation_confidence,
             source_document, model_version, provenance, evaluation_status, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                draft.id,
                draft.topic,
                draft.section,
                draft.format,
                draft.stem,
                serde_json::to_string(&draft.choices).unwrap(),
                draft.correct_answer,
                draft.explanation,
                draft.difficulty,
                draft.attribution.source_name,
                draft.attribution.source_section,
                draft.attribution.generated_at_secs,
                draft.confidence,
                meta.source_document,
                meta.model_version,
                meta.provenance.as_str(),
                meta.evaluation_status.as_str(),
                usn,
                now,
            ],
        )?;
        Ok(rows > 0)
    }

    pub fn insert_generated_question(
        &self,
        draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
    ) -> Result<()> {
        self.insert_generated_question_with_meta(draft, &default_template_metadata(draft))
    }

    /// Insert a generated question with explicit provenance/eval metadata.
    pub fn insert_generated_question_with_meta(
        &self,
        draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
        meta: &crate::gre_atlas::questions::metadata::QuestionMetadata,
    ) -> Result<()> {
        let now = TimestampSecs::now().0;
        let usn = self.next_usn()?;
        self.db.execute(
            "INSERT INTO bl_question
            (id, topic, section, format, stem, choices_json, correct_answer, explanation,
             difficulty, source_name, source_section, generated_at_secs, generation_confidence,
             source_document, model_version, provenance, evaluation_status, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                draft.id,
                draft.topic,
                draft.section,
                draft.format,
                draft.stem,
                serde_json::to_string(&draft.choices).unwrap(),
                draft.correct_answer,
                draft.explanation,
                draft.difficulty,
                draft.attribution.source_name,
                draft.attribution.source_section,
                draft.attribution.generated_at_secs,
                draft.confidence,
                meta.source_document,
                meta.model_version,
                meta.provenance.as_str(),
                meta.evaluation_status.as_str(),
                usn,
                now,
            ],
        )?;
        Ok(())
    }

    /// Append a per-candidate evaluation row for pass/reject metrics. Rejected
    /// candidates are logged here but never inserted into `bl_question`.
    pub fn record_generation_eval(
        &self,
        row: &crate::gre_atlas::questions::eval_pipeline::GenerationEvalRecord,
    ) -> Result<()> {
        let now = TimestampSecs::now().0;
        self.db.execute(
            "INSERT INTO bl_generation_eval
            (candidate_id, topic, model_version, provenance, status, reason, confidence,
             evaluated_at_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                row.candidate_id,
                row.topic,
                row.model_version,
                row.provenance.as_str(),
                row.status.as_str(),
                row.reason,
                row.confidence,
                now,
            ],
        )?;
        Ok(())
    }

    /// Count generation-eval rows grouped by status (for metrics reporting).
    pub fn generation_eval_counts(&self) -> Result<HashMap<String, u32>> {
        let mut stmt = self
            .db
            .prepare_cached("SELECT status, COUNT(*) FROM bl_generation_eval GROUP BY status")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;
        rows.collect::<std::result::Result<HashMap<_, _>, _>>()
            .map_err(Into::into)
    }

    pub fn delete_question(&self, question_id: &str) -> Result<()> {
        let tx = self.db.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM bl_performance_attempt WHERE question_id = ?1",
            [question_id],
        )?;
        tx.execute("DELETE FROM bl_question WHERE id = ?1", [question_id])?;
        tx.commit()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_attempt(
        &self,
        question_id: &str,
        topic: &str,
        difficulty: Option<f32>,
        answer: &str,
        correct: bool,
        response_time_ms: u32,
        confidence: Option<u32>,
        session_id: Option<&str>,
    ) -> Result<()> {
        if let Some(session_id) = session_id {
            if !self.session_exists(session_id)? {
                return Err(crate::error::AnkiError::InvalidInput {
                    source: snafu::FromString::without_source(format!(
                        "unknown session_id: {session_id}"
                    )),
                });
            }
        }
        let now = TimestampSecs::now().0;
        let usn = self.next_usn()?;
        self.db.execute(
            "INSERT INTO bl_performance_attempt
            (question_id, topic, difficulty, answered_at_secs, answer, correct, response_time_ms,
             confidence, session_id, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                question_id,
                topic,
                difficulty,
                now,
                answer,
                i32::from(correct),
                response_time_ms,
                confidence,
                session_id,
                usn,
                now,
            ],
        )?;
        Ok(())
    }

    fn next_usn(&self) -> Result<i32> {
        let current: i32 = self
            .db
            .query_row(
                "SELECT CAST(val AS INTEGER) FROM bl_meta WHERE key = 'sync_usn'",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let next = current + 1;
        self.set_meta("sync_usn", &next.to_string())?;
        Ok(next)
    }

    pub fn sync_status(&self) -> Result<SyncStatus> {
        let current_usn: i32 = self
            .db
            .query_row(
                "SELECT CAST(val AS INTEGER) FROM bl_meta WHERE key = 'sync_usn'",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let last_pushed: i32 = self
            .db
            .query_row(
                "SELECT CAST(val AS INTEGER) FROM bl_meta WHERE key = 'last_pushed_usn'",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let pending_upload_count = if current_usn > last_pushed {
            (current_usn - last_pushed) as u32
        } else {
            0
        };
        let last_modified_secs: i64 = self.db.query_row(
            "SELECT COALESCE(MAX(mtime_secs), 0) FROM bl_performance_attempt",
            [],
            |row| row.get(0),
        )?;
        Ok(SyncStatus {
            current_usn,
            pending_upload_count,
            last_modified_secs: TimestampSecs(last_modified_secs),
        })
    }

    pub fn mark_synced_through(&self, usn: i32) -> Result<()> {
        self.set_meta("last_pushed_usn", &usn.to_string())
    }

    pub fn pull_changes(&self, after_usn: i32, limit: u32) -> Result<(Vec<SyncAttemptRow>, i32)> {
        let limit = limit.max(1) as i64;
        let mut stmt = self.db.prepare_cached(
            "SELECT id, question_id, topic, difficulty, answered_at_secs, answer, correct,
                    response_time_ms, confidence, session_id, usn, mtime_secs
             FROM bl_performance_attempt
             WHERE usn > ?
             ORDER BY usn
             LIMIT ?",
        )?;
        let rows = stmt.query_map(params![after_usn, limit], row_to_sync_attempt)?;
        let attempts = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        let status = self.sync_status()?;
        Ok((attempts, status.current_usn))
    }

    pub fn push_changes(&self, incoming: &[SyncAttemptRow]) -> Result<PushChangesResult> {
        let mut applied_count = 0u32;
        let mut conflicts = Vec::new();
        for row in incoming {
            self.ensure_fk_prerequisites_for_attempt(row)?;
            let (applied, row_conflicts) = self.merge_incoming_attempt(row)?;
            if applied {
                applied_count += 1;
            }
            conflicts.extend(row_conflicts);
        }
        let status = self.sync_status()?;
        Ok(PushChangesResult {
            current_usn: status.current_usn,
            applied_count,
            conflicts,
        })
    }

    /// Ensure FK parent rows exist before inserting or updating an attempt.
    pub(crate) fn ensure_fk_prerequisites_for_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        self.ensure_session_for_attempt(row)?;
        self.ensure_question_for_attempt(row)?;
        Ok(())
    }

    /// Ensure `bl_session` exists before inserting an attempt that references
    /// it. Used when session rows were not bundled (e.g. server USN
    /// filter).
    pub(crate) fn ensure_session_for_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        let Some(session_id) = row.session_id.as_deref() else {
            return Ok(());
        };
        if self.session_exists(session_id)? {
            return Ok(());
        }
        let usn = self.next_usn()?;
        sync_execute(
            &self.db,
            "bl_session",
            "ensure_session_for_attempt",
            &format!(
                "id={session_id} started_at_secs={} mtime_secs={}",
                row.answered_at_secs.0, row.mtime_secs.0
            ),
            "INSERT INTO bl_session (id, started_at_secs, source, usn, mtime_secs)
             VALUES (?, ?, 'practice', ?, ?)",
            params![session_id, row.answered_at_secs.0, usn, row.mtime_secs.0,],
            &SyncFkContext::default(),
        )?;
        Ok(())
    }

    /// Ensure `bl_question` exists before inserting an attempt that references
    /// it. Used when question rows were not bundled (e.g. server USN
    /// filter).
    pub(crate) fn ensure_question_for_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        if self.question_exists(&row.question_id)? {
            return Ok(());
        }
        let usn = self.next_usn()?;
        sync_execute(
            &self.db,
            "bl_question",
            "ensure_question_for_attempt",
            &format!(
                "id={} topic={} mtime_secs={}",
                row.question_id, row.topic, row.mtime_secs.0
            ),
            "INSERT INTO bl_question
            (id, topic, section, format, stem, choices_json, correct_answer, explanation,
             difficulty, usn, mtime_secs)
            VALUES (?, ?, 'sync', 'mcq', ?, '[]', ?, '', ?, ?, ?)",
            params![
                row.question_id,
                row.topic,
                SYNC_QUESTION_STUB_STEM,
                row.answer,
                row.difficulty,
                usn,
                row.mtime_secs.0,
            ],
            &SyncFkContext {
                question_id: Some(row.question_id.clone()),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    fn get_sync_attempt(&self, id: i64) -> Result<Option<SyncAttemptRow>> {
        self.db
            .query_row(
                "SELECT id, question_id, topic, difficulty, answered_at_secs, answer, correct,
                        response_time_ms, confidence, session_id, usn, mtime_secs
                 FROM bl_performance_attempt WHERE id = ?",
                [id],
                row_to_sync_attempt,
            )
            .optional()
            .map_err(Into::into)
    }

    fn find_sync_attempt_by_identity(
        &self,
        row: &SyncAttemptRow,
    ) -> Result<Option<SyncAttemptRow>> {
        let sql = "SELECT id, question_id, topic, difficulty, answered_at_secs, answer, correct,
                          response_time_ms, confidence, session_id, usn, mtime_secs
                   FROM bl_performance_attempt
                   WHERE question_id = ?1 AND answered_at_secs = ?2 AND
                         ((session_id IS NULL AND ?3 IS NULL) OR session_id = ?3)";
        self.db
            .query_row(
                sql,
                params![row.question_id, row.answered_at_secs.0, row.session_id,],
                row_to_sync_attempt,
            )
            .optional()
            .map_err(Into::into)
    }

    fn merge_incoming_attempt(&self, row: &SyncAttemptRow) -> Result<(bool, Vec<SyncConflict>)> {
        if row.id == 0 {
            if let Some(local) = self.find_sync_attempt_by_identity(row)? {
                return self.maybe_update_existing_attempt(&local, row);
            }
            self.insert_sync_attempt(row)?;
            return Ok((true, vec![]));
        }

        let existing = self.get_sync_attempt(row.id)?;
        match existing {
            None => {
                if let Some(local) = self.find_sync_attempt_by_identity(row)? {
                    return self.maybe_update_existing_attempt(&local, row);
                }
                self.insert_sync_attempt_with_id(row)?;
                Ok((true, vec![]))
            }
            Some(local) if attempt_identity_differs(&local, row) => {
                self.insert_sync_attempt(row)?;
                Ok((true, vec![]))
            }
            Some(local) => self.maybe_update_existing_attempt(&local, row),
        }
    }

    fn maybe_update_existing_attempt(
        &self,
        local: &SyncAttemptRow,
        row: &SyncAttemptRow,
    ) -> Result<(bool, Vec<SyncConflict>)> {
        if row.mtime_secs.0 > local.mtime_secs.0 {
            let mut updated = row.clone();
            updated.id = local.id;
            self.update_sync_attempt(&updated)?;
            Ok((true, vec![]))
        } else if row.mtime_secs.0 < local.mtime_secs.0 {
            Ok((
                false,
                vec![SyncConflict {
                    attempt_id: row.id,
                    reason: "Local copy is newer; remote change rejected".into(),
                    kept: local.clone(),
                    rejected: row.clone(),
                }],
            ))
        } else {
            Ok((false, vec![]))
        }
    }

    fn insert_sync_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        let detail = format!(
            "question_id={} session_id={:?} answered_at_secs={}",
            row.question_id, row.session_id, row.answered_at_secs.0
        );
        sync_execute(
            &self.db,
            "bl_performance_attempt",
            "insert_sync_attempt",
            &detail,
            "INSERT INTO bl_performance_attempt
            (question_id, topic, difficulty, answered_at_secs, answer, correct, response_time_ms,
             confidence, session_id, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                row.question_id,
                row.topic,
                row.difficulty,
                row.answered_at_secs.0,
                row.answer,
                i32::from(row.correct),
                row.response_time_ms,
                row.confidence,
                row.session_id,
                usn,
                row.mtime_secs.0,
            ],
            &SyncFkContext {
                question_id: Some(row.question_id.clone()),
                session_id: row.session_id.clone(),
                attempt_id: None,
            },
        )?;
        Ok(())
    }

    fn insert_sync_attempt_with_id(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        let detail = format!(
            "id={} question_id={} session_id={:?}",
            row.id, row.question_id, row.session_id
        );
        sync_execute(
            &self.db,
            "bl_performance_attempt",
            "insert_sync_attempt_with_id",
            &detail,
            "INSERT INTO bl_performance_attempt
            (id, question_id, topic, difficulty, answered_at_secs, answer, correct, response_time_ms,
             confidence, session_id, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                row.id,
                row.question_id,
                row.topic,
                row.difficulty,
                row.answered_at_secs.0,
                row.answer,
                i32::from(row.correct),
                row.response_time_ms,
                row.confidence,
                row.session_id,
                usn,
                row.mtime_secs.0,
            ],
            &SyncFkContext {
                question_id: Some(row.question_id.clone()),
                session_id: row.session_id.clone(),
                attempt_id: Some(row.id),
            },
        )?;
        Ok(())
    }

    fn update_sync_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        let detail = format!(
            "id={} question_id={} session_id={:?} mtime_secs={}",
            row.id, row.question_id, row.session_id, row.mtime_secs.0
        );
        sync_execute(
            &self.db,
            "bl_performance_attempt",
            "update_sync_attempt",
            &detail,
            "UPDATE bl_performance_attempt SET
                question_id = ?, topic = ?, difficulty = ?, answered_at_secs = ?, answer = ?,
                correct = ?, response_time_ms = ?, confidence = ?, session_id = ?,
                usn = ?, mtime_secs = ?
             WHERE id = ?",
            params![
                row.question_id,
                row.topic,
                row.difficulty,
                row.answered_at_secs.0,
                row.answer,
                i32::from(row.correct),
                row.response_time_ms,
                row.confidence,
                row.session_id,
                usn,
                row.mtime_secs.0,
                row.id,
            ],
            &SyncFkContext {
                question_id: Some(row.question_id.clone()),
                session_id: row.session_id.clone(),
                attempt_id: Some(row.id),
            },
        )?;
        Ok(())
    }

    pub fn performance_summary(&self) -> Result<(u32, u32)> {
        Ok(self.performance_stats()?.0)
    }

    /// Practice accuracy grouped by attempt topic and resolved question type.
    #[allow(clippy::type_complexity)]
    pub(crate) fn performance_stats_by_topic_question_type(
        &self,
    ) -> Result<HashMap<(String, String), (u32, u32)>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT a.topic, q.format, q.explanation, COALESCE(SUM(a.correct), 0), COUNT(*)
             FROM bl_performance_attempt a
             JOIN bl_question q ON q.id = a.question_id
             GROUP BY a.topic, q.format, q.explanation",
        )?;
        let mut stats = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)? as u32,
                row.get::<_, i64>(4)? as u32,
            ))
        })?;
        for row in rows {
            let (topic, format, explanation, correct, total) = row?;
            let question_type =
                crate::gre_atlas::questions::explanation::question_type_key(&format, &explanation);
            let entry = stats.entry((topic, question_type)).or_insert((0, 0));
            entry.0 += correct;
            entry.1 += total;
        }
        Ok(stats)
    }

    /// Single pass over practice attempts for totals and per-topic aggregates.
    #[allow(clippy::type_complexity)]
    pub fn performance_stats(&self) -> Result<((u32, u32), HashMap<String, (u32, u32)>)> {
        let mut stmt = self.db.prepare_cached(
            "SELECT topic, COALESCE(SUM(correct), 0), COUNT(*)
             FROM bl_performance_attempt
             GROUP BY topic",
        )?;
        let mut correct_total = 0u32;
        let mut count_total = 0u32;
        let mut by_topic = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as u32,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;
        for row in rows {
            let (topic, correct, total) = row?;
            correct_total += correct;
            count_total += total;
            by_topic.insert(topic, (correct, total));
        }
        Ok(((correct_total, count_total), by_topic))
    }

    pub(crate) fn list_performance_attempts_for_eval(
        &self,
    ) -> Result<Vec<PerformanceAttemptEvalRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, topic, correct
             FROM bl_performance_attempt
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PerformanceAttemptEvalRow {
                id: row.get(0)?,
                topic: row.get(1)?,
                correct: row.get::<_, i32>(2)? != 0,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn performance_by_topic(&self) -> Result<Vec<(String, u32, u32)>> {
        self.performance_stats().map(|(_, by_topic)| {
            by_topic
                .into_iter()
                .map(|(topic, (correct, total))| (topic, correct, total))
                .collect()
        })
    }

    pub fn recent_attempts(
        &self,
        topic_prefix: &str,
        limit: u32,
    ) -> Result<Vec<PerformanceAttemptRow>> {
        let limit = limit.max(1) as i64;
        let mut stmt = self.db.prepare_cached(
            "SELECT question_id, topic, difficulty, answered_at_secs, answer, correct,
                    response_time_ms, confidence, session_id
             FROM bl_performance_attempt
             WHERE (?1 = '' OR topic = ?1 OR topic LIKE ?1 || '::%')
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![topic_prefix, limit], |row| {
            Ok(PerformanceAttemptRow {
                question_id: row.get(0)?,
                topic: row.get(1)?,
                difficulty: row.get(2)?,
                answered_at_secs: TimestampSecs(row.get(3)?),
                answer: row.get(4)?,
                correct: row.get::<_, i32>(5)? != 0,
                response_time_ms: row.get(6)?,
                confidence: row.get(7)?,
                session_id: row.get(8)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn has_performance_attempts(&self, topic_prefix: &str) -> Result<bool> {
        let count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM bl_performance_attempt
             WHERE (?1 = '' OR topic = ?1 OR topic LIKE ?1 || '::%')",
            [topic_prefix],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn earliest_attempt_secs(&self, topic_prefix: &str) -> Result<Option<TimestampSecs>> {
        let value: Option<i64> = self.db.query_row(
            "SELECT MIN(answered_at_secs) FROM bl_performance_attempt
             WHERE (?1 = '' OR topic = ?1 OR topic LIKE ?1 || '::%')",
            [topic_prefix],
            |row| row.get(0),
        )?;
        Ok(value.map(TimestampSecs))
    }

    pub fn attempts_in_range(
        &self,
        since: TimestampSecs,
        until: TimestampSecs,
        topic_prefix: &str,
    ) -> Result<Vec<PerformanceAttemptChartRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT answered_at_secs, correct
             FROM bl_performance_attempt
             WHERE answered_at_secs >= ?1 AND answered_at_secs < ?2
               AND (?3 = '' OR topic = ?3 OR topic LIKE ?3 || '::%')
             ORDER BY answered_at_secs ASC, id ASC",
        )?;
        let rows = stmt.query_map(params![since.0, until.0, topic_prefix], |row| {
            Ok(PerformanceAttemptChartRow {
                answered_at_secs: TimestampSecs(row.get(0)?),
                correct: row.get::<_, i32>(1)? != 0,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn practice_stats_since(&self, since: TimestampSecs) -> Result<(u32, u32)> {
        let correct: i64 = self.db.query_row(
            "SELECT COALESCE(SUM(correct), 0) FROM bl_performance_attempt
             WHERE answered_at_secs >= ?1",
            [since.0],
            |row| row.get(0),
        )?;
        let total: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM bl_performance_attempt WHERE answered_at_secs >= ?1",
            [since.0],
            |row| row.get(0),
        )?;
        Ok((correct as u32, total as u32))
    }

    pub(crate) fn maybe_record_readiness_prediction(
        &self,
        snapshot: &crate::gre_atlas::calibration::ReadinessPredictionSnapshot,
    ) -> Result<()> {
        use crate::gre_atlas::calibration::MIN_PREDICTION_INTERVAL_SECS;
        use crate::gre_atlas::calibration::PREDICTION_SCORE_DELTA;
        use crate::gre_atlas::calibration::READINESS_MODEL_VERSION;

        if let Some(latest) = self.latest_readiness_prediction()? {
            let elapsed = TimestampSecs::now().0 - latest.predicted_at_secs.0;
            if elapsed < MIN_PREDICTION_INTERVAL_SECS
                && (latest.projected_score - snapshot.projected_score).abs()
                    < PREDICTION_SCORE_DELTA
            {
                return Ok(());
            }
        }

        let now = TimestampSecs::now().0;
        let usn = self.next_usn()?;
        self.db.execute(
            "INSERT INTO bl_readiness_prediction
            (predicted_at_secs, projected_score, projected_score_low, projected_score_high,
             memory_score, performance_score, coverage_ratio, confidence_level, model_version,
             usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                now,
                snapshot.projected_score,
                snapshot.projected_score_low,
                snapshot.projected_score_high,
                snapshot.memory_score,
                snapshot.performance_score,
                snapshot.coverage_ratio,
                snapshot.confidence_level,
                READINESS_MODEL_VERSION,
                usn,
                now,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn resolve_pending_outcomes(
        &self,
        inputs: &crate::gre_atlas::calibration::OutcomeInputs,
    ) -> Result<()> {
        use crate::gre_atlas::calibration::resolve_outcome_score;

        let now = TimestampSecs::now();
        let mut stmt = self.db.prepare_cached(
            "SELECT id, predicted_at_secs, projected_score, projected_score_low,
                    projected_score_high, memory_score, performance_score, coverage_ratio,
                    confidence_level, model_version, outcome_score, outcome_observed_at_secs,
                    outcome_memory_score, outcome_performance_score, practice_correct,
                    practice_total
             FROM bl_readiness_prediction
             WHERE outcome_score IS NULL
             ORDER BY predicted_at_secs ASC",
        )?;
        let rows = stmt.query_map([], row_to_readiness_prediction)?;
        let pending: Vec<_> = rows.collect::<Result<Vec<_>, _>>()?;

        for prediction in pending {
            let practice = self.practice_stats_since(prediction.predicted_at_secs)?;
            let outcome_inputs = crate::gre_atlas::calibration::OutcomeInputs {
                memory_score: inputs.memory_score,
                performance_score: inputs.performance_score,
                coverage_ratio: inputs.coverage_ratio,
                practice_correct: practice.0,
                practice_total: practice.1,
            };
            let Some(outcome_score) = resolve_outcome_score(&prediction, &outcome_inputs, now)
            else {
                continue;
            };
            let perf_score = if practice.1 > 0 {
                practice.0 as f32 / practice.1 as f32 * 100.0
            } else {
                inputs.performance_score
            };
            let usn = self.next_usn()?;
            self.db.execute(
                "UPDATE bl_readiness_prediction
                 SET outcome_score = ?, outcome_observed_at_secs = ?,
                     outcome_memory_score = ?, outcome_performance_score = ?,
                     practice_correct = ?, practice_total = ?, usn = ?, mtime_secs = ?
                 WHERE id = ?",
                params![
                    outcome_score,
                    now.0,
                    inputs.memory_score,
                    perf_score,
                    practice.0,
                    practice.1,
                    usn,
                    now.0,
                    prediction.id,
                ],
            )?;
        }
        Ok(())
    }

    pub(crate) fn list_readiness_predictions(&self) -> Result<Vec<ReadinessPredictionRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, predicted_at_secs, projected_score, projected_score_low,
                    projected_score_high, memory_score, performance_score, coverage_ratio,
                    confidence_level, model_version, outcome_score, outcome_observed_at_secs,
                    outcome_memory_score, outcome_performance_score, practice_correct,
                    practice_total
             FROM bl_readiness_prediction
             ORDER BY predicted_at_secs ASC",
        )?;
        let rows = stmt.query_map([], row_to_readiness_prediction)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn latest_readiness_prediction(&self) -> Result<Option<ReadinessPredictionRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, predicted_at_secs, projected_score, projected_score_low,
                    projected_score_high, memory_score, performance_score, coverage_ratio,
                    confidence_level, model_version, outcome_score, outcome_observed_at_secs,
                    outcome_memory_score, outcome_performance_score, practice_correct,
                    practice_total
             FROM bl_readiness_prediction
             ORDER BY predicted_at_secs DESC
             LIMIT 1",
        )?;
        stmt.query_row([], row_to_readiness_prediction)
            .optional()
            .map_err(Into::into)
    }
}

fn row_to_readiness_prediction(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ReadinessPredictionRow> {
    Ok(ReadinessPredictionRow {
        id: row.get(0)?,
        predicted_at_secs: TimestampSecs(row.get(1)?),
        projected_score: row.get(2)?,
        projected_score_low: row.get(3)?,
        projected_score_high: row.get(4)?,
        memory_score: row.get(5)?,
        performance_score: row.get(6)?,
        coverage_ratio: row.get(7)?,
        confidence_level: row.get(8)?,
        model_version: row.get(9)?,
        outcome_score: row.get(10)?,
        outcome_observed_at_secs: row.get::<_, Option<i64>>(11)?.map(TimestampSecs),
        outcome_memory_score: row.get(12)?,
        outcome_performance_score: row.get(13)?,
        practice_correct: row.get::<_, Option<i64>>(14)?.map(|v| v as u32),
        practice_total: row.get::<_, Option<i64>>(15)?.map(|v| v as u32),
    })
}

pub(crate) fn attempt_identity_differs(local: &SyncAttemptRow, remote: &SyncAttemptRow) -> bool {
    local.question_id != remote.question_id
        || local.answered_at_secs != remote.answered_at_secs
        || local.session_id != remote.session_id
}

fn row_to_sync_attempt(row: &rusqlite::Row<'_>) -> rusqlite::Result<SyncAttemptRow> {
    let correct: i32 = row.get(6)?;
    Ok(SyncAttemptRow {
        id: row.get(0)?,
        question_id: row.get(1)?,
        topic: row.get(2)?,
        difficulty: row.get(3)?,
        answered_at_secs: TimestampSecs(row.get(4)?),
        answer: row.get(5)?,
        correct: correct != 0,
        response_time_ms: row.get(7)?,
        confidence: row.get(8)?,
        session_id: row.get(9)?,
        usn: row.get(10)?,
        mtime_secs: TimestampSecs(row.get(11)?),
    })
}

fn row_to_stored_question(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredQuestion> {
    let choices_json: Option<String> = row.get(5)?;
    let choices: Vec<String> = choices_json
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(5, "choices".into(), rusqlite::types::Type::Text)
        })?
        .unwrap_or_default();
    Ok(StoredQuestion {
        id: row.get(0)?,
        topic: row.get(1)?,
        section: row.get(2)?,
        format: row.get(3)?,
        stem: row.get(4)?,
        choices,
        correct_answer: row.get(6)?,
        explanation: row.get(7)?,
        difficulty: row.get(8)?,
        source_name: row.get(9)?,
        source_section: row.get(10)?,
        generated_at_secs: row.get(11)?,
        generation_confidence: row.get(12)?,
        source_document: row.get(13)?,
        model_version: row.get(14)?,
        provenance: row.get(15)?,
        evaluation_status: row.get(16)?,
    })
}

/// Metadata for a template draft when no explicit metadata is provided (the
/// deterministic bank top-up path). Uses the draft's own model version so
/// callers that predate the metadata columns keep sensible provenance.
fn default_template_metadata(
    draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
) -> crate::gre_atlas::questions::metadata::QuestionMetadata {
    crate::gre_atlas::questions::metadata::QuestionMetadata::offline_template(
        crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION,
        draft.attribution.source_section.clone(),
    )
}

fn new_session_id() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}

pub fn gre_atlas_db_path(collection_path: &Path) -> PathBuf {
    if collection_path.to_string_lossy() == ":memory:" {
        return std::env::temp_dir().join(format!("anki-greatlas-{}.db", std::process::id()));
    }
    collection_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(GRE_ATLAS_DB_NAME)
}

fn migrate_legacy_db_filename(db_path: &Path) -> Result<()> {
    if db_path.is_file() {
        return Ok(());
    }
    let Some(parent) = db_path.parent() else {
        return Ok(());
    };
    let legacy_path = parent.join(LEGACY_BRAINLIFT_DB_NAME);
    if legacy_path.is_file() {
        std::fs::rename(&legacy_path, db_path)?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct SeedQuestion {
    id: String,
    topic: String,
    section: String,
    format: String,
    stem: String,
    choices: Vec<String>,
    correct_answer: String,
    explanation: String,
    #[serde(default)]
    difficulty: Option<f32>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;

    fn open_v1_db(dir: &Path) -> Result<Connection> {
        let db_path = dir.join(GRE_ATLAS_DB_NAME);
        let db = Connection::open(db_path)?;
        db.execute_batch(
            "CREATE TABLE bl_meta (key TEXT PRIMARY KEY, val TEXT NOT NULL);
             CREATE TABLE bl_question (
               id TEXT PRIMARY KEY, topic TEXT NOT NULL, section TEXT NOT NULL,
               format TEXT NOT NULL, stem TEXT NOT NULL, choices_json TEXT,
               correct_answer TEXT NOT NULL, explanation TEXT NOT NULL,
               difficulty REAL, usn INTEGER NOT NULL DEFAULT 0, mtime_secs INTEGER NOT NULL
             );
             CREATE TABLE bl_performance_attempt (
               id INTEGER PRIMARY KEY, question_id TEXT NOT NULL, topic TEXT NOT NULL,
               answered_at_secs INTEGER NOT NULL, answer TEXT NOT NULL, correct INTEGER NOT NULL,
               response_time_ms INTEGER NOT NULL, confidence INTEGER, session_id TEXT,
               usn INTEGER NOT NULL DEFAULT -1, mtime_secs INTEGER NOT NULL
             );
             INSERT INTO bl_meta (key, val) VALUES ('schema_version', '1');",
        )?;
        Ok(db)
    }

    #[test]
    fn list_questions_filters_by_topic_prefix() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let quant = storage.list_questions("gre::quant", 100)?;
        assert!(quant.iter().all(|q| q.topic.starts_with("gre::quant")));
        assert!(quant.len() >= 4);
        let verbal = storage.list_questions("gre::verbal", 100)?;
        assert!(verbal.iter().all(|q| q.topic.starts_with("gre::verbal")));
        assert!(verbal.len() >= 2);
        let linear = storage.list_questions("gre::quant::algebra::linear", 100)?;
        assert!(linear
            .iter()
            .all(|q| q.topic == "gre::quant::algebra::linear"));
        assert!(linear
            .iter()
            .any(|q| q.id.starts_with("gre-foundation-quant-lin")));
        Ok(())
    }

    #[test]
    fn list_questions_deprioritizes_answered_questions() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;

        let all = storage.list_questions("", 500)?;
        assert!(
            all.len() >= 2,
            "seed bank should contain multiple questions"
        );
        let answered_id = all[0].id.clone();
        let answered_topic = all[0].topic.clone();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &answered_id,
            &answered_topic,
            None,
            "answer",
            true,
            1000,
            None,
            Some(&session.id),
        )?;

        // Once answered, the question moves into a higher attempt-count tier and
        // must sort after every still-unanswered question.
        let ordered = storage.list_questions("", 500)?;
        let answered_pos = ordered
            .iter()
            .position(|q| q.id == answered_id)
            .expect("answered question should still be listed");
        assert_eq!(
            answered_pos,
            ordered.len() - 1,
            "answered question should be deprioritized to the end of the list"
        );
        Ok(())
    }

    #[test]
    fn seeds_sample_gre_questions() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let questions = storage.list_practice_bank_questions(
            "",
            crate::gre_atlas::questions::PRACTICE_QUESTION_LIST_LIMIT,
        )?;
        assert_eq!(
            questions.len(),
            crate::gre_atlas::questions::PRACTICE_BANK_QUESTION_TOTAL as usize
        );
        let quant = questions.iter().filter(|q| q.section == "quant").count();
        let verbal = questions.iter().filter(|q| q.section == "verbal").count();
        let awa = questions.iter().filter(|q| q.section == "awa").count();
        assert_eq!(
            quant,
            crate::gre_atlas::questions::TARGET_PRACTICE_BANK_QUANT as usize
        );
        assert_eq!(
            verbal,
            crate::gre_atlas::questions::TARGET_PRACTICE_BANK_VERBAL as usize
        );
        assert_eq!(
            awa,
            crate::gre_atlas::questions::TARGET_PRACTICE_BANK_AWA as usize
        );
        assert_eq!(quant + verbal + awa, questions.len());
        Ok(())
    }

    #[test]
    fn seed_bank_is_valid_and_covers_every_leaf_topic() {
        let seed = crate::gre_atlas::questions::foundation::load_foundation_bank();

        assert!(
            seed.len()
                >= crate::gre_atlas::questions::foundation::MIN_FOUNDATION_VERBAL
                    + crate::gre_atlas::questions::foundation::MIN_FOUNDATION_QUANT
                    + crate::gre_atlas::questions::foundation::MIN_FOUNDATION_AWA,
            "foundation bank should meet minimum counts, got {}",
            seed.len()
        );

        let mut ids = std::collections::HashSet::new();
        for q in &seed {
            assert!(ids.insert(q.id.as_str()), "duplicate seed id: {}", q.id);
            let choices = q.choice_list();
            assert!(!choices.is_empty(), "{} has no choices", q.id);
            assert!(
                crate::gre_atlas::questions::variants::correct_answer_in_choices(
                    &q.correct_answer,
                    choices
                ),
                "{}: correct_answer {:?} is not among its choices",
                q.id,
                q.correct_answer
            );
            assert!(
                crate::gre_atlas::GreCatalog::topic_by_id(&q.topic).is_some(),
                "{}: topic {:?} is not in the catalog",
                q.id,
                q.topic
            );
        }

        for leaf in crate::gre_atlas::GreCatalog::leaf_topics() {
            assert!(
                seed.iter().any(|q| q.topic == leaf.id),
                "no foundation question covers leaf topic {}",
                leaf.id
            );
        }
    }

    #[test]
    fn seeds_and_records_attempts() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let col = CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col.col_path)?;
        assert_eq!(storage.schema_version()?, SCHEMA_VERSION);
        let questions = storage.list_questions("", 10)?;
        assert!(!questions.is_empty());
        let q = &questions[0];
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            &q.correct_answer,
            true,
            1200,
            Some(4),
            Some(&session.id),
        )?;
        let attempts = storage.recent_attempts("", 1)?;
        let attempt = &attempts[0];
        assert_eq!(attempt.question_id, q.id);
        assert_eq!(attempt.topic, q.topic);
        assert_eq!(attempt.answer, q.correct_answer);
        assert!(attempt.correct);
        assert_eq!(attempt.response_time_ms, 1200);
        assert_eq!(attempt.confidence, Some(4));
        assert_eq!(attempt.session_id.as_deref(), Some(session.id.as_str()));
        assert!(attempt.answered_at_secs.0 > 0);
        let (correct, total) = storage.performance_summary()?;
        assert_eq!(correct, 1);
        assert_eq!(total, 1);
        Ok(())
    }

    #[test]
    fn migrates_schema_v1_to_current() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        open_v1_db(dir.path())?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        assert_eq!(storage.schema_version()?, SCHEMA_VERSION);
        let has_difficulty: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('bl_performance_attempt')
             WHERE name = 'difficulty'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(has_difficulty, 1);
        let session_table: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bl_session'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(session_table, 1);
        let prediction_table: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bl_readiness_prediction'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(prediction_table, 1);
        Ok(())
    }

    #[test]
    fn migrates_schema_v3_to_v4_adds_attribution_columns() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let db_path = dir.path().join(LEGACY_BRAINLIFT_DB_NAME);
        let db = Connection::open(&db_path)?;
        db.execute_batch(
            "CREATE TABLE bl_meta (key TEXT PRIMARY KEY, val TEXT NOT NULL);
             CREATE TABLE bl_question (
               id TEXT PRIMARY KEY, topic TEXT NOT NULL, section TEXT NOT NULL,
               format TEXT NOT NULL, stem TEXT NOT NULL, choices_json TEXT,
               correct_answer TEXT NOT NULL, explanation TEXT NOT NULL,
               difficulty REAL, usn INTEGER NOT NULL DEFAULT 0, mtime_secs INTEGER NOT NULL
             );
             INSERT INTO bl_meta (key, val) VALUES ('schema_version', '3');",
        )?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        assert_eq!(storage.schema_version()?, SCHEMA_VERSION);
        let attribution_cols: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('bl_question')
             WHERE name IN ('source_name', 'source_section', 'generated_at_secs', 'generation_confidence')",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(attribution_cols, 4);
        Ok(())
    }

    #[test]
    fn migrates_schema_v4_to_v5_adds_ai_metadata_columns_and_eval_table() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let db_path = dir.path().join(LEGACY_BRAINLIFT_DB_NAME);
        let db = Connection::open(&db_path)?;
        // A realistic v4 sidecar: bl_question exists with the v4 attribution
        // columns but none of the v5 AI-metadata columns.
        db.execute_batch(
            "CREATE TABLE bl_meta (key TEXT PRIMARY KEY, val TEXT NOT NULL);
             CREATE TABLE bl_question (
               id TEXT PRIMARY KEY, topic TEXT NOT NULL, section TEXT NOT NULL,
               format TEXT NOT NULL, stem TEXT NOT NULL, choices_json TEXT,
               correct_answer TEXT NOT NULL, explanation TEXT NOT NULL,
               difficulty REAL, source_name TEXT, source_section TEXT,
               generated_at_secs INTEGER, generation_confidence REAL,
               usn INTEGER NOT NULL DEFAULT 0, mtime_secs INTEGER NOT NULL
             );
             INSERT INTO bl_meta (key, val) VALUES ('schema_version', '4');",
        )?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        assert_eq!(storage.schema_version()?, SCHEMA_VERSION);
        let ai_cols: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('bl_question')
             WHERE name IN ('source_document', 'model_version', 'provenance', 'evaluation_status')",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(ai_cols, 4);
        let eval_table: i64 = storage.db.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bl_generation_eval'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(eval_table, 1);
        Ok(())
    }

    #[test]
    fn migration_is_idempotent_across_reopens() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        // Opening repeatedly must not error (no duplicate ALTERs, etc.) and must
        // stay at the current schema version.
        for _ in 0..3 {
            let storage = GreAtlasStorage::open(&col_path)?;
            assert_eq!(storage.schema_version()?, SCHEMA_VERSION);
            drop(storage);
        }
        Ok(())
    }

    #[test]
    fn stores_and_reads_ai_generation_metadata() -> Result<()> {
        use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
        use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
        use crate::gre_atlas::questions::metadata::EvaluationStatus;
        use crate::gre_atlas::questions::metadata::Provenance;
        use crate::gre_atlas::questions::metadata::QuestionMetadata;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;

        let draft = GeneratedQuestionDraft {
            id: "ai-llm-meta-test".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: "If 2x = 10, what is x?".into(),
            choices: vec!["3".into(), "4".into(), "5".into(), "6".into()],
            correct_answer: "5".into(),
            explanation: "Divide by 2.".into(),
            difficulty: Some(0.4),
            confidence: 0.8,
            attribution: QuestionAttribution {
                source_name: "ETS Official GRE Prep Material".into(),
                source_section: "Quantitative Reasoning — Linear equations".into(),
                generated_at_secs: 42,
            },
        };
        let meta = QuestionMetadata {
            provenance: Provenance::AiGenerated,
            model_version: "gpt-4o-mini".into(),
            source_document: "Quantitative Reasoning — Linear equations".into(),
            evaluation_status: EvaluationStatus::Approved,
        };
        storage.insert_generated_question_with_meta(&draft, &meta)?;

        let stored = storage.get_question("ai-llm-meta-test")?.unwrap();
        assert_eq!(stored.provenance.as_deref(), Some("ai_generated"));
        assert_eq!(stored.model_version.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(stored.evaluation_status.as_deref(), Some("approved"));
        assert_eq!(
            stored.source_document.as_deref(),
            Some("Quantitative Reasoning — Linear equations")
        );
        Ok(())
    }

    #[test]
    fn list_questions_excludes_rejected_candidates() -> Result<()> {
        use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
        use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
        use crate::gre_atlas::questions::metadata::EvaluationStatus;
        use crate::gre_atlas::questions::metadata::Provenance;
        use crate::gre_atlas::questions::metadata::QuestionMetadata;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;

        let draft = GeneratedQuestionDraft {
            id: "ai-rejected-row".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: "A rejected candidate that must never surface.".into(),
            choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            correct_answer: "1".into(),
            explanation: "n/a".into(),
            difficulty: Some(0.4),
            confidence: 0.1,
            attribution: QuestionAttribution {
                source_name: "ETS Official GRE Prep Material".into(),
                source_section: "Quantitative Reasoning — Linear equations".into(),
                generated_at_secs: 1,
            },
        };
        let meta = QuestionMetadata {
            provenance: Provenance::AiGenerated,
            model_version: "gpt-4o-mini".into(),
            source_document: "Quantitative Reasoning — Linear equations".into(),
            evaluation_status: EvaluationStatus::RejectedHallucination,
        };
        storage.insert_generated_question_with_meta(&draft, &meta)?;

        let listed = storage.list_questions("", u32::MAX)?;
        assert!(
            !listed.iter().any(|q| q.id == "ai-rejected-row"),
            "rejected AI candidate must not reach learners"
        );
        // But it is still directly retrievable for auditing.
        assert!(storage.get_question("ai-rejected-row")?.is_some());
        Ok(())
    }

    #[test]
    fn records_and_counts_generation_eval() -> Result<()> {
        use crate::gre_atlas::questions::eval_pipeline::GenerationEvalRecord;
        use crate::gre_atlas::questions::metadata::EvaluationStatus;
        use crate::gre_atlas::questions::metadata::Provenance;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;

        storage.record_generation_eval(&GenerationEvalRecord {
            candidate_id: "c1".into(),
            topic: "gre::quant::algebra::linear".into(),
            model_version: "gpt-4o-mini".into(),
            provenance: Provenance::AiGenerated,
            status: EvaluationStatus::Approved,
            reason: String::new(),
            confidence: Some(0.9),
        })?;
        storage.record_generation_eval(&GenerationEvalRecord {
            candidate_id: "c2".into(),
            topic: "gre::quant::algebra::linear".into(),
            model_version: "gpt-4o-mini".into(),
            provenance: Provenance::AiGenerated,
            status: EvaluationStatus::RejectedDuplicate,
            reason: "near-duplicate".into(),
            confidence: Some(0.2),
        })?;

        let counts = storage.generation_eval_counts()?;
        assert_eq!(counts.get("approved").copied(), Some(1));
        assert_eq!(counts.get("rejected_duplicate").copied(), Some(1));
        Ok(())
    }

    #[test]
    fn migrates_legacy_brainlift_db_filename() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let legacy_path = dir.path().join(LEGACY_BRAINLIFT_DB_NAME);
        let db = Connection::open(&legacy_path)?;
        // This test only exercises the filename rename, so pin the sidecar at
        // the current schema version (no upgrade path is under test here).
        db.execute_batch(&format!(
            "CREATE TABLE bl_meta (key TEXT PRIMARY KEY, val TEXT NOT NULL);
             INSERT INTO bl_meta (key, val) VALUES ('schema_version', '{SCHEMA_VERSION}');",
        ))?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let _storage = GreAtlasStorage::open(&col_path)?;
        assert!(
            dir.path().join(GRE_ATLAS_DB_NAME).is_file(),
            "expected legacy brainlift.db to be renamed to greatlas.db"
        );
        assert!(
            !legacy_path.is_file(),
            "legacy brainlift.db should be renamed away"
        );
        Ok(())
    }

    #[test]
    fn rejects_unknown_session_id() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        CollectionBuilder::new(&col_path).build()?;
        let storage = GreAtlasStorage::open(&col_path)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let err = storage
            .record_attempt(
                &q.id,
                &q.topic,
                None,
                "x",
                false,
                100,
                None,
                Some("missing-session"),
            )
            .unwrap_err();
        assert!(matches!(err, crate::error::AnkiError::InvalidInput { .. }));
        Ok(())
    }

    #[test]
    fn record_attempt_does_not_touch_collection_revlog() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let col = CollectionBuilder::new(&col_path).build()?;
        let revlog_before: i64 =
            col.storage
                .db
                .query_row("SELECT COUNT(*) FROM revlog", [], |row| row.get(0))?;
        let storage = GreAtlasStorage::open(&col.col_path)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            &q.correct_answer,
            true,
            500,
            None,
            Some(&session.id),
        )?;
        let revlog_after: i64 =
            col.storage
                .db
                .query_row("SELECT COUNT(*) FROM revlog", [], |row| row.get(0))?;
        assert_eq!(revlog_before, revlog_after);
        Ok(())
    }
}
