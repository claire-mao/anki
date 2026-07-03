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

pub(crate) const SCHEMA_VERSION: u32 = 4;

/// Sidecar SQLite filename beside the collection profile (GRE Atlas practice
/// data).
pub const GRE_ATLAS_DB_NAME: &str = "greatlas.db";
/// Legacy sidecar filename; migrated to [`GRE_ATLAS_DB_NAME`] on open when
/// present.
pub const LEGACY_BRAINLIFT_DB_NAME: &str = "brainlift.db";

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
            session_id: value.session_id,
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
        db.set_prepared_statement_cache_capacity(20);

        let storage = Self { db };
        storage.migrate(collection_path)?;
        storage.seed_questions_if_empty()?;
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

        let seed: Vec<SeedQuestion> =
            serde_json::from_str(include_str!("../questions/seed_gre.json")).map_err(|err| {
                crate::error::AnkiError::InvalidInput {
                    source: snafu::FromString::without_source(format!("seed questions: {err}")),
                }
            })?;
        let now = TimestampSecs::now().0;
        for q in seed {
            self.db.execute(
                "INSERT INTO bl_question
                (id, topic, section, format, stem, choices_json, correct_answer, explanation,
                 difficulty, usn, mtime_secs)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
                params![
                    q.id,
                    q.topic,
                    q.section,
                    q.format,
                    q.stem,
                    serde_json::to_string(&q.choices).unwrap(),
                    q.correct_answer,
                    q.explanation,
                    q.difficulty,
                    now,
                ],
            )?;
        }
        Ok(())
    }

    pub fn create_session(&self, source: &str) -> Result<PracticeSession> {
        let now = TimestampSecs::now();
        let id = new_session_id();
        let source = if source.is_empty() {
            "practice"
        } else {
            source
        };
        self.db.execute(
            "INSERT INTO bl_session (id, started_at_secs, source, usn, mtime_secs)
             VALUES (?, ?, ?, -1, ?)",
            params![id, now.0, source, now.0],
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

    pub fn list_questions(&self, topic_prefix: &str, limit: u32) -> Result<Vec<StoredQuestion>> {
        let limit = limit.max(1) as i64;
        let mut stmt = self.db.prepare_cached(
            "SELECT id, topic, section, format, stem, choices_json, correct_answer, explanation,
                    difficulty, source_name, source_section, generated_at_secs, generation_confidence
             FROM bl_question
             WHERE (?1 = '' OR topic = ?1 OR topic LIKE ?1 || '::%')
             ORDER BY id
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![topic_prefix, limit], row_to_stored_question)?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn get_question(&self, question_id: &str) -> Result<Option<StoredQuestion>> {
        self.db
            .query_row(
                "SELECT id, topic, section, format, stem, choices_json, correct_answer,
                        explanation, difficulty, source_name, source_section, generated_at_secs,
                        generation_confidence
                 FROM bl_question WHERE id = ?",
                [question_id],
                row_to_stored_question,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn insert_generated_question(
        &self,
        draft: &crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft,
    ) -> Result<()> {
        let now = TimestampSecs::now().0;
        self.db.execute(
            "INSERT INTO bl_question
            (id, topic, section, format, stem, choices_json, correct_answer, explanation,
             difficulty, source_name, source_section, generated_at_secs, generation_confidence,
             usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
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
                now,
            ],
        )?;
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
            if row.id == 0 {
                self.insert_sync_attempt(row)?;
                applied_count += 1;
                continue;
            }
            let existing = self.get_sync_attempt(row.id)?;
            match existing {
                None => {
                    self.insert_sync_attempt_with_id(row)?;
                    applied_count += 1;
                }
                Some(local) => {
                    if row.mtime_secs.0 > local.mtime_secs.0 {
                        self.update_sync_attempt(row)?;
                        applied_count += 1;
                    } else if row.mtime_secs.0 < local.mtime_secs.0 {
                        conflicts.push(SyncConflict {
                            attempt_id: row.id,
                            reason: "Local copy is newer; remote change rejected".into(),
                            kept: local,
                            rejected: row.clone(),
                        });
                    }
                }
            }
        }
        let status = self.sync_status()?;
        Ok(PushChangesResult {
            current_usn: status.current_usn,
            applied_count,
            conflicts,
        })
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

    fn insert_sync_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        self.db.execute(
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
        )?;
        Ok(())
    }

    fn insert_sync_attempt_with_id(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        self.db.execute(
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
        )?;
        Ok(())
    }

    fn update_sync_attempt(&self, row: &SyncAttemptRow) -> Result<()> {
        let usn = self.next_usn()?;
        self.db.execute(
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
        )?;
        Ok(())
    }

    pub fn performance_summary(&self) -> Result<(u32, u32)> {
        Ok(self.performance_stats()?.0)
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
        self.db.execute(
            "INSERT INTO bl_readiness_prediction
            (predicted_at_secs, projected_score, projected_score_low, projected_score_high,
             memory_score, performance_score, coverage_ratio, confidence_level, model_version,
             usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, -1, ?)",
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
            self.db.execute(
                "UPDATE bl_readiness_prediction
                 SET outcome_score = ?, outcome_observed_at_secs = ?,
                     outcome_memory_score = ?, outcome_performance_score = ?,
                     practice_correct = ?, practice_total = ?, mtime_secs = ?
                 WHERE id = ?",
                params![
                    outcome_score,
                    now.0,
                    inputs.memory_score,
                    perf_score,
                    practice.0,
                    practice.1,
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
    })
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
        assert_eq!(linear.len(), 1);
        assert_eq!(linear[0].id, "gre-quant-alg-001");
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
        let questions = storage.list_questions("", 100)?;
        assert!(questions.len() >= 9);
        assert!(questions.iter().any(|q| q.section == "quant"));
        assert!(questions.iter().any(|q| q.section == "verbal"));
        assert!(questions.iter().any(|q| q.section == "awa"));
        Ok(())
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
    fn migrates_legacy_brainlift_db_filename() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let legacy_path = dir.path().join(LEGACY_BRAINLIFT_DB_NAME);
        let db = Connection::open(&legacy_path)?;
        db.execute_batch(
            "CREATE TABLE bl_meta (key TEXT PRIMARY KEY, val TEXT NOT NULL);
             INSERT INTO bl_meta (key, val) VALUES ('schema_version', '4');",
        )?;
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
