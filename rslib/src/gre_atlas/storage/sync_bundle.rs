// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Incremental GRE Atlas sidecar sync bundle: sessions, questions, attempts,
//! and readiness calibration rows. Merge order respects foreign keys
//! (sessions → questions → attempts → predictions).

use std::collections::HashSet;

use rusqlite::params;
use rusqlite::OptionalExtension;
use tracing::debug;

use super::sync_execute;
use super::GreAtlasStorage;
use super::SyncAttemptRow;
use super::SyncConflict;
use super::SyncFkContext;
use super::is_sync_question_stub_stem;
use crate::error::Result;
use crate::timestamp::TimestampSecs;

pub const META_LAST_DOWNLOADED_USN: &str = "last_downloaded_usn";

impl SyncBundle {
    /// Incremental export filter used by the sync server download handler.
    ///
    /// Rows with `usn > after_usn` are included, plus any `bl_session` /
    /// `bl_question` parents referenced by exported attempts even when those
    /// parents were created before the watermark (avoids orphan attempts).
    pub fn filter_after_usn(self, after_usn: i32) -> SyncBundle {
        let attempts: Vec<_> = self
            .attempts
            .into_iter()
            .filter(|row| row.usn > after_usn)
            .collect();
        let session_ids: HashSet<String> = attempts
            .iter()
            .filter_map(|row| row.session_id.clone())
            .collect();
        let question_ids: HashSet<String> =
            attempts.iter().map(|row| row.question_id.clone()).collect();
        let sessions: Vec<_> = self
            .sessions
            .into_iter()
            .filter(|row| row.usn > after_usn || session_ids.contains(&row.id))
            .collect();
        let questions: Vec<_> = self
            .questions
            .into_iter()
            .filter(|row| row.usn > after_usn || question_ids.contains(&row.id))
            .collect();
        let predictions: Vec<_> = self
            .predictions
            .into_iter()
            .filter(|row| row.usn > after_usn)
            .collect();
        SyncBundle {
            sessions,
            questions,
            attempts,
            predictions,
            current_usn: self.current_usn,
            last_modified_secs: self.last_modified_secs,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyncBundle {
    pub sessions: Vec<SyncSessionRow>,
    pub questions: Vec<SyncQuestionRow>,
    pub attempts: Vec<SyncAttemptRow>,
    pub predictions: Vec<SyncPredictionRow>,
    pub current_usn: i32,
    pub last_modified_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct SyncSessionRow {
    pub id: String,
    pub started_at_secs: TimestampSecs,
    pub ended_at_secs: Option<TimestampSecs>,
    pub source: String,
    pub usn: i32,
    pub mtime_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct SyncQuestionRow {
    pub id: String,
    pub topic: String,
    pub section: String,
    pub format: String,
    pub stem: String,
    pub choices_json: String,
    pub correct_answer: String,
    pub explanation: String,
    pub difficulty: Option<f32>,
    pub source_name: Option<String>,
    pub source_section: Option<String>,
    pub generated_at_secs: Option<i64>,
    pub generation_confidence: Option<f32>,
    pub source_document: Option<String>,
    pub model_version: Option<String>,
    pub provenance: Option<String>,
    pub evaluation_status: Option<String>,
    pub usn: i32,
    pub mtime_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct SyncPredictionRow {
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
    pub practice_correct: Option<i32>,
    pub practice_total: Option<i32>,
    pub usn: i32,
    pub mtime_secs: TimestampSecs,
}

#[derive(Debug, Clone)]
pub struct ApplyBundleResult {
    pub current_usn: i32,
    pub applied_count: u32,
    pub conflicts: Vec<SyncConflict>,
}

impl GreAtlasStorage {
    pub fn last_downloaded_usn(&self) -> Result<i32> {
        self.meta_i32(META_LAST_DOWNLOADED_USN)
    }

    pub fn set_last_downloaded_usn(&self, usn: i32) -> Result<()> {
        self.set_meta(META_LAST_DOWNLOADED_USN, &usn.to_string())
    }

    pub fn last_pushed_usn(&self) -> Result<i32> {
        self.meta_i32("last_pushed_usn")
    }

    fn meta_i32(&self, key: &str) -> Result<i32> {
        Ok(self
            .db
            .query_row(
                "SELECT CAST(val AS INTEGER) FROM bl_meta WHERE key = ?",
                [key],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    }

    /// Export rows changed since `after_usn` across all syncable tables.
    ///
    /// Attempt rows always include their referenced `bl_session` and
    /// `bl_question` rows even when those USNs are at or below `after_usn`
    /// (e.g. parent row created before the last push, new attempt added
    /// later).
    pub fn pull_sync_bundle(&self, after_usn: i32, limit: u32) -> Result<SyncBundle> {
        let limit = limit.max(1) as i64;
        let status = self.sync_status()?;
        let mut sessions = self.pull_sessions(after_usn, limit)?;
        let attempts = self.pull_attempts(after_usn, limit)?;
        self.ensure_referenced_sessions(&mut sessions, &attempts)?;
        let mut questions = self.pull_questions(after_usn, limit)?;
        self.ensure_referenced_questions(&mut questions, &attempts)?;
        Ok(SyncBundle {
            sessions,
            questions,
            attempts,
            predictions: self.pull_predictions(after_usn, limit)?,
            current_usn: status.current_usn,
            last_modified_secs: status.last_modified_secs,
        })
    }

    fn ensure_referenced_sessions(
        &self,
        sessions: &mut Vec<SyncSessionRow>,
        attempts: &[SyncAttemptRow],
    ) -> Result<()> {
        let mut known: HashSet<String> = sessions.iter().map(|s| s.id.clone()).collect();
        for attempt in attempts {
            if let Some(session_id) = &attempt.session_id {
                if known.insert(session_id.clone()) {
                    if let Some(row) = self.load_session_by_id(session_id)? {
                        sessions.push(row);
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_referenced_questions(
        &self,
        questions: &mut Vec<SyncQuestionRow>,
        attempts: &[SyncAttemptRow],
    ) -> Result<()> {
        let mut known: HashSet<String> = questions.iter().map(|q| q.id.clone()).collect();
        for attempt in attempts {
            if known.insert(attempt.question_id.clone()) {
                if let Some(row) = self.load_question_sync_row(&attempt.question_id)? {
                    questions.push(row);
                }
            }
        }
        Ok(())
    }

    fn load_question_sync_row(&self, question_id: &str) -> Result<Option<SyncQuestionRow>> {
        self.db
            .query_row(
                "SELECT id, topic, section, format, stem, choices_json, correct_answer, explanation,
                        difficulty, source_name, source_section, generated_at_secs,
                        generation_confidence, source_document, model_version, provenance,
                        evaluation_status, usn, mtime_secs
                 FROM bl_question WHERE id = ?",
                [question_id],
                |row| {
                    Ok(SyncQuestionRow {
                        id: row.get(0)?,
                        topic: row.get(1)?,
                        section: row.get(2)?,
                        format: row.get(3)?,
                        stem: row.get(4)?,
                        choices_json: row.get(5)?,
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
                        usn: row.get(17)?,
                        mtime_secs: TimestampSecs(row.get(18)?),
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_session_by_id(&self, session_id: &str) -> Result<Option<SyncSessionRow>> {
        self.db
            .query_row(
                "SELECT id, started_at_secs, ended_at_secs, source, usn, mtime_secs
                 FROM bl_session WHERE id = ?",
                [session_id],
                |row| {
                    Ok(SyncSessionRow {
                        id: row.get(0)?,
                        started_at_secs: TimestampSecs(row.get(1)?),
                        ended_at_secs: row.get::<_, Option<i64>>(2)?.map(TimestampSecs),
                        source: row.get(3)?,
                        usn: row.get(4)?,
                        mtime_secs: TimestampSecs(row.get(5)?),
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Merge an incoming bundle in FK-safe order. Uses last-write-wins on
    /// `mtime_secs` for every entity type.
    pub fn apply_sync_bundle(&self, bundle: &SyncBundle) -> Result<ApplyBundleResult> {
        debug!(
            target: "gre_atlas::sync",
            sessions = bundle.sessions.len(),
            questions = bundle.questions.len(),
            attempts = bundle.attempts.len(),
            predictions = bundle.predictions.len(),
            "apply_sync_bundle start"
        );
        let mut applied_count = 0u32;
        let mut conflicts = Vec::new();

        for row in &bundle.sessions {
            if self.merge_session(row)? {
                applied_count += 1;
            }
        }
        for row in &bundle.questions {
            if self.merge_question(row)? {
                applied_count += 1;
            }
        }
        for row in &bundle.attempts {
            self.ensure_fk_prerequisites_for_attempt(row)?;
        }
        let attempt_result = self.push_changes(&bundle.attempts)?;
        applied_count += attempt_result.applied_count;
        conflicts.extend(attempt_result.conflicts);
        for row in &bundle.predictions {
            if self.merge_prediction(row)? {
                applied_count += 1;
            }
        }

        let status = self.sync_status()?;
        Ok(ApplyBundleResult {
            current_usn: status.current_usn,
            applied_count,
            conflicts,
        })
    }

    fn pull_sessions(&self, after_usn: i32, limit: i64) -> Result<Vec<SyncSessionRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, started_at_secs, ended_at_secs, source, usn, mtime_secs
             FROM bl_session WHERE usn > ? ORDER BY usn LIMIT ?",
        )?;
        let rows = stmt.query_map(params![after_usn, limit], |row| {
            Ok(SyncSessionRow {
                id: row.get(0)?,
                started_at_secs: TimestampSecs(row.get(1)?),
                ended_at_secs: row.get::<_, Option<i64>>(2)?.map(TimestampSecs),
                source: row.get(3)?,
                usn: row.get(4)?,
                mtime_secs: TimestampSecs(row.get(5)?),
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn pull_questions(&self, after_usn: i32, limit: i64) -> Result<Vec<SyncQuestionRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, topic, section, format, stem, choices_json, correct_answer, explanation,
                    difficulty, source_name, source_section, generated_at_secs,
                    generation_confidence, source_document, model_version, provenance,
                    evaluation_status, usn, mtime_secs
             FROM bl_question WHERE usn > ? ORDER BY usn LIMIT ?",
        )?;
        let rows = stmt.query_map(params![after_usn, limit], |row| {
            Ok(SyncQuestionRow {
                id: row.get(0)?,
                topic: row.get(1)?,
                section: row.get(2)?,
                format: row.get(3)?,
                stem: row.get(4)?,
                choices_json: row.get(5)?,
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
                usn: row.get(17)?,
                mtime_secs: TimestampSecs(row.get(18)?),
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn pull_attempts(&self, after_usn: i32, limit: i64) -> Result<Vec<SyncAttemptRow>> {
        let (attempts, _) = self.pull_changes(after_usn, limit as u32)?;
        Ok(attempts)
    }

    fn pull_predictions(&self, after_usn: i32, limit: i64) -> Result<Vec<SyncPredictionRow>> {
        let mut stmt = self.db.prepare_cached(
            "SELECT id, predicted_at_secs, projected_score, projected_score_low,
                    projected_score_high, memory_score, performance_score, coverage_ratio,
                    confidence_level, model_version, outcome_score, outcome_observed_at_secs,
                    outcome_memory_score, outcome_performance_score, practice_correct,
                    practice_total, usn, mtime_secs
             FROM bl_readiness_prediction WHERE usn > ? ORDER BY usn LIMIT ?",
        )?;
        let rows = stmt.query_map(params![after_usn, limit], |row| {
            Ok(SyncPredictionRow {
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
                practice_correct: row.get(14)?,
                practice_total: row.get(15)?,
                usn: row.get(16)?,
                mtime_secs: TimestampSecs(row.get(17)?),
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn merge_session(&self, row: &SyncSessionRow) -> Result<bool> {
        let existing: Option<(i64, i64)> = self
            .db
            .query_row(
                "SELECT mtime_secs, started_at_secs FROM bl_session WHERE id = ?",
                [&row.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match existing {
            None => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_session",
                    "merge_session",
                    &format!(
                        "insert id={} started_at_secs={} mtime_secs={}",
                        row.id, row.started_at_secs.0, row.mtime_secs.0
                    ),
                    "INSERT INTO bl_session (id, started_at_secs, ended_at_secs, source, usn, mtime_secs)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        row.id,
                        row.started_at_secs.0,
                        row.ended_at_secs.map(|t| t.0),
                        row.source,
                        usn,
                        row.mtime_secs.0,
                    ],
                    &SyncFkContext::default(),
                )?;
                Ok(true)
            }
            Some((local_mtime, _)) if row.mtime_secs.0 > local_mtime => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_session",
                    "merge_session",
                    &format!("update id={} mtime_secs={}", row.id, row.mtime_secs.0),
                    "UPDATE bl_session SET started_at_secs = ?, ended_at_secs = ?, source = ?,
                     usn = ?, mtime_secs = ? WHERE id = ?",
                    params![
                        row.started_at_secs.0,
                        row.ended_at_secs.map(|t| t.0),
                        row.source,
                        usn,
                        row.mtime_secs.0,
                        row.id,
                    ],
                    &SyncFkContext::default(),
                )?;
                Ok(true)
            }
            Some((local_mtime, _)) if row.mtime_secs.0 < local_mtime => Ok(false),
            Some(_) => Ok(false),
        }
    }

    fn merge_question(&self, row: &SyncQuestionRow) -> Result<bool> {
        let existing_mtime: Option<i64> = self
            .db
            .query_row(
                "SELECT mtime_secs FROM bl_question WHERE id = ?",
                [&row.id],
                |r| r.get(0),
            )
            .optional()?;
        match existing_mtime {
            None => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_question",
                    "merge_question",
                    &format!(
                        "insert id={} topic={} mtime_secs={}",
                        row.id, row.topic, row.mtime_secs.0
                    ),
                    "INSERT INTO bl_question
                    (id, topic, section, format, stem, choices_json, correct_answer, explanation,
                     difficulty, source_name, source_section, generated_at_secs,
                     generation_confidence, source_document, model_version, provenance,
                     evaluation_status, usn, mtime_secs)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        row.id,
                        row.topic,
                        row.section,
                        row.format,
                        row.stem,
                        row.choices_json,
                        row.correct_answer,
                        row.explanation,
                        row.difficulty,
                        row.source_name,
                        row.source_section,
                        row.generated_at_secs,
                        row.generation_confidence,
                        row.source_document,
                        row.model_version,
                        row.provenance,
                        row.evaluation_status,
                        usn,
                        row.mtime_secs.0,
                    ],
                    &SyncFkContext {
                        question_id: Some(row.id.clone()),
                        ..Default::default()
                    },
                )?;
                Ok(true)
            }
            Some(local_mtime) if is_sync_question_stub_stem(&row.stem) => Ok(false),
            Some(local_mtime)
                if self.question_stem_is_sync_stub(&row.id)?
                    || row.mtime_secs.0 > local_mtime =>
            {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_question",
                    "merge_question",
                    &format!("update id={} mtime_secs={}", row.id, row.mtime_secs.0),
                    "UPDATE bl_question SET topic = ?, section = ?, format = ?, stem = ?,
                     choices_json = ?, correct_answer = ?, explanation = ?, difficulty = ?,
                     source_name = ?, source_section = ?, generated_at_secs = ?,
                     generation_confidence = ?, source_document = ?, model_version = ?,
                     provenance = ?, evaluation_status = ?, usn = ?, mtime_secs = ?
                     WHERE id = ?",
                    params![
                        row.topic,
                        row.section,
                        row.format,
                        row.stem,
                        row.choices_json,
                        row.correct_answer,
                        row.explanation,
                        row.difficulty,
                        row.source_name,
                        row.source_section,
                        row.generated_at_secs,
                        row.generation_confidence,
                        row.source_document,
                        row.model_version,
                        row.provenance,
                        row.evaluation_status,
                        usn,
                        row.mtime_secs.0,
                        row.id,
                    ],
                    &SyncFkContext {
                        question_id: Some(row.id.clone()),
                        ..Default::default()
                    },
                )?;
                Ok(true)
            }
            Some(local_mtime) if row.mtime_secs.0 < local_mtime => Ok(false),
            Some(_) => Ok(false),
        }
    }

    fn merge_prediction(&self, row: &SyncPredictionRow) -> Result<bool> {
        let existing: Option<(i64, i64)> = self
            .db
            .query_row(
                "SELECT mtime_secs, predicted_at_secs FROM bl_readiness_prediction WHERE id = ?",
                [row.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match existing {
            None if row.id == 0 => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_readiness_prediction",
                    "merge_prediction",
                    &format!(
                        "insert autoincrement predicted_at_secs={}",
                        row.predicted_at_secs.0
                    ),
                    "INSERT INTO bl_readiness_prediction
                    (predicted_at_secs, projected_score, projected_score_low, projected_score_high,
                     memory_score, performance_score, coverage_ratio, confidence_level,
                     model_version, outcome_score, outcome_observed_at_secs,
                     outcome_memory_score, outcome_performance_score, practice_correct,
                     practice_total, usn, mtime_secs)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        row.predicted_at_secs.0,
                        row.projected_score,
                        row.projected_score_low,
                        row.projected_score_high,
                        row.memory_score,
                        row.performance_score,
                        row.coverage_ratio,
                        row.confidence_level,
                        row.model_version,
                        row.outcome_score,
                        row.outcome_observed_at_secs.map(|t| t.0),
                        row.outcome_memory_score,
                        row.outcome_performance_score,
                        row.practice_correct,
                        row.practice_total,
                        usn,
                        row.mtime_secs.0,
                    ],
                    &SyncFkContext::default(),
                )?;
                Ok(true)
            }
            None => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_readiness_prediction",
                    "merge_prediction",
                    &format!(
                        "insert id={} predicted_at_secs={}",
                        row.id, row.predicted_at_secs.0
                    ),
                    "INSERT INTO bl_readiness_prediction
                    (id, predicted_at_secs, projected_score, projected_score_low,
                     projected_score_high, memory_score, performance_score, coverage_ratio,
                     confidence_level, model_version, outcome_score, outcome_observed_at_secs,
                     outcome_memory_score, outcome_performance_score, practice_correct,
                     practice_total, usn, mtime_secs)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        row.id,
                        row.predicted_at_secs.0,
                        row.projected_score,
                        row.projected_score_low,
                        row.projected_score_high,
                        row.memory_score,
                        row.performance_score,
                        row.coverage_ratio,
                        row.confidence_level,
                        row.model_version,
                        row.outcome_score,
                        row.outcome_observed_at_secs.map(|t| t.0),
                        row.outcome_memory_score,
                        row.outcome_performance_score,
                        row.practice_correct,
                        row.practice_total,
                        usn,
                        row.mtime_secs.0,
                    ],
                    &SyncFkContext::default(),
                )?;
                Ok(true)
            }
            Some((local_mtime, _)) if row.mtime_secs.0 > local_mtime => {
                let usn = self.next_usn()?;
                sync_execute(
                    &self.db,
                    "bl_readiness_prediction",
                    "merge_prediction",
                    &format!("update id={} mtime_secs={}", row.id, row.mtime_secs.0),
                    "UPDATE bl_readiness_prediction SET
                     predicted_at_secs = ?, projected_score = ?, projected_score_low = ?,
                     projected_score_high = ?, memory_score = ?, performance_score = ?,
                     coverage_ratio = ?, confidence_level = ?, model_version = ?,
                     outcome_score = ?, outcome_observed_at_secs = ?,
                     outcome_memory_score = ?, outcome_performance_score = ?,
                     practice_correct = ?, practice_total = ?, usn = ?, mtime_secs = ?
                     WHERE id = ?",
                    params![
                        row.predicted_at_secs.0,
                        row.projected_score,
                        row.projected_score_low,
                        row.projected_score_high,
                        row.memory_score,
                        row.performance_score,
                        row.coverage_ratio,
                        row.confidence_level,
                        row.model_version,
                        row.outcome_score,
                        row.outcome_observed_at_secs.map(|t| t.0),
                        row.outcome_memory_score,
                        row.outcome_performance_score,
                        row.practice_correct,
                        row.practice_total,
                        usn,
                        row.mtime_secs.0,
                        row.id,
                    ],
                    &SyncFkContext::default(),
                )?;
                Ok(true)
            }
            Some((local_mtime, _)) if row.mtime_secs.0 < local_mtime => Ok(false),
            Some(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;
    use crate::gre_atlas::gre_atlas_storage;

    fn isolated_col() -> crate::error::Result<crate::collection::Collection> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        CollectionBuilder::new(dir.path().join("test.anki2")).build()
    }

    #[test]
    fn bundle_merge_applies_session_before_attempt() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let session_id = "remote-session-1".to_string();
        let q = storage.list_questions("", 1)?.pop().unwrap();

        let bundle = SyncBundle {
            sessions: vec![SyncSessionRow {
                id: session_id.clone(),
                started_at_secs: TimestampSecs(1_700_000_000),
                ended_at_secs: None,
                source: "practice".into(),
                usn: 1,
                mtime_secs: TimestampSecs(1_700_000_000),
            }],
            attempts: vec![SyncAttemptRow {
                id: 0,
                question_id: q.id.clone(),
                topic: q.topic.clone(),
                difficulty: q.difficulty,
                answered_at_secs: TimestampSecs(1_700_000_100),
                answer: "synced".into(),
                correct: true,
                response_time_ms: 800,
                confidence: None,
                session_id: Some(session_id),
                usn: 2,
                mtime_secs: TimestampSecs(1_700_000_100),
            }],
            ..Default::default()
        };

        let result = storage.apply_sync_bundle(&bundle)?;
        assert!(result.applied_count >= 2);
        assert!(storage.performance_summary()?.1 >= 1);
        Ok(())
    }

    #[test]
    fn bundle_export_includes_stale_session_for_new_attempt() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.mark_synced_through(storage.sync_status()?.current_usn)?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "late-attempt",
            true,
            500,
            None,
            Some(&session.id),
        )?;

        let bundle = storage.pull_sync_bundle(storage.last_pushed_usn()?, 5000)?;
        assert_eq!(bundle.attempts.len(), 1);
        assert!(
            bundle.sessions.iter().any(|s| s.id == session.id),
            "session must be bundled with attempt even when session usn <= last_pushed"
        );
        assert!(
            bundle.questions.iter().any(|question| question.id == q.id),
            "question must be bundled with attempt even when question usn <= last_pushed"
        );
        Ok(())
    }

    /// Simulates server-side USN filtering that drops parent rows while keeping
    /// new attempts (iOS download path before apply).
    #[test]
    fn apply_filtered_incremental_bundle_stubs_missing_session_and_question() -> Result<()> {
        let mut desktop = isolated_col()?;
        let d_storage = gre_atlas_storage(&mut desktop)?;
        let q = d_storage.list_questions("", 1)?.pop().unwrap();
        let session = d_storage.create_session("practice")?;
        d_storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "first",
            true,
            500,
            None,
            Some(&session.id),
        )?;
        d_storage.mark_synced_through(d_storage.sync_status()?.current_usn)?;
        d_storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "second",
            false,
            600,
            None,
            Some(&session.id),
        )?;

        let incremental = d_storage.pull_sync_bundle(d_storage.last_pushed_usn()?, 5000)?;
        assert_eq!(incremental.attempts.len(), 1);

        // Server `filter_bundle_after_usn` keeps only rows with usn > after_usn.
        let filtered = SyncBundle {
            sessions: vec![],
            questions: vec![],
            attempts: incremental.attempts,
            ..Default::default()
        };

        let mut mobile = isolated_col()?;
        let m_storage = gre_atlas_storage(&mut mobile)?;
        // Mobile has a different question bank; drop the referenced question locally.
        m_storage.delete_question(&q.id)?;

        let result = m_storage.apply_sync_bundle(&filtered)?;
        assert!(result.applied_count >= 1);
        assert!(m_storage.session_exists(&session.id)?);
        assert!(m_storage.question_exists(&q.id)?);
        assert!(m_storage.performance_summary()?.1 >= 1);
        Ok(())
    }

    #[test]
    fn server_filter_includes_stale_parents_for_incremental_attempt() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "first",
            true,
            500,
            None,
            Some(&session.id),
        )?;
        storage.mark_synced_through(storage.sync_status()?.current_usn)?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "second",
            false,
            600,
            None,
            Some(&session.id),
        )?;

        let export = storage.pull_sync_bundle(0, 5000)?;
        let filtered = export.filter_after_usn(storage.last_pushed_usn()?);
        assert_eq!(filtered.attempts.len(), 1);
        assert!(
            filtered.sessions.iter().any(|s| s.id == session.id),
            "server filter must retain stale session for incremental attempt"
        );
        assert!(
            filtered
                .questions
                .iter()
                .any(|question| question.id == q.id),
            "server filter must retain stale question for incremental attempt"
        );
        Ok(())
    }

    #[test]
    fn apply_attempt_without_session_in_bundle_creates_session_stub() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session_id = "orphan-session".to_string();

        let bundle = SyncBundle {
            attempts: vec![SyncAttemptRow {
                id: 0,
                question_id: q.id.clone(),
                topic: q.topic.clone(),
                difficulty: q.difficulty,
                answered_at_secs: TimestampSecs(1_700_000_100),
                answer: "synced".into(),
                correct: true,
                response_time_ms: 800,
                confidence: None,
                session_id: Some(session_id.clone()),
                usn: 2,
                mtime_secs: TimestampSecs(1_700_000_100),
            }],
            ..Default::default()
        };

        storage.apply_sync_bundle(&bundle)?;
        assert!(storage.session_exists(&session_id)?);
        assert!(storage.performance_summary()?.1 >= 1);
        Ok(())
    }

    #[test]
    fn bundle_round_trip_preserves_question_attribution() -> Result<()> {
        use crate::gre_atlas::questions::ai_gen::GeneratedQuestionDraft;
        use crate::gre_atlas::questions::ai_gen::QuestionAttribution;
        use crate::gre_atlas::questions::metadata::EvaluationStatus;
        use crate::gre_atlas::questions::metadata::Provenance;
        use crate::gre_atlas::questions::metadata::QuestionMetadata;
        use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;

        let mut source_col = isolated_col()?;
        let source_storage = gre_atlas_storage(&mut source_col)?;
        let draft = GeneratedQuestionDraft {
            id: "sync-attribution-test".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: "If 3x = 15, what is x?".into(),
            choices: vec!["3".into(), "4".into(), "5".into(), "6".into()],
            correct_answer: "5".into(),
            explanation: "Divide both sides by 3.".into(),
            difficulty: Some(0.4),
            confidence: 0.82,
            attribution: QuestionAttribution {
                source_name: GENERATION_SOURCE_NAME.into(),
                source_section: "Quantitative Reasoning — Linear equations".into(),
                generated_at_secs: 1_700_000_000,
            },
        };
        let meta = QuestionMetadata {
            provenance: Provenance::OfflineTemplate,
            model_version: "template_v1".into(),
            source_document: "Quantitative Reasoning — Linear equations".into(),
            evaluation_status: EvaluationStatus::Approved,
        };
        source_storage.insert_generated_question_with_meta(&draft, &meta)?;
        source_storage.mark_synced_through(0)?;

        let bundle = source_storage.pull_sync_bundle(0, 5000)?;
        let synced = bundle
            .questions
            .iter()
            .find(|q| q.id == "sync-attribution-test")
            .expect("generated question in bundle");
        assert_eq!(synced.source_name.as_deref(), Some(GENERATION_SOURCE_NAME));
        assert_eq!(
            synced.provenance.as_deref(),
            Some(Provenance::OfflineTemplate.as_str())
        );
        assert_eq!(synced.generation_confidence, Some(0.82));

        let mut target_col = isolated_col()?;
        let target_storage = gre_atlas_storage(&mut target_col)?;
        target_storage.apply_sync_bundle(&bundle)?;

        let stored = target_storage
            .get_question("sync-attribution-test")?
            .expect("question applied");
        assert_eq!(stored.source_name.as_deref(), Some(GENERATION_SOURCE_NAME));
        assert_eq!(
            stored.provenance.as_deref(),
            Some(Provenance::OfflineTemplate.as_str())
        );
        assert_eq!(stored.generation_confidence, Some(0.82));
        assert_eq!(
            stored.source_document.as_deref(),
            Some("Quantitative Reasoning — Linear equations")
        );
        Ok(())
    }
}
