// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::error::Result;
use crate::timestamp::TimestampSecs;

pub(crate) const SCHEMA_VERSION: &str = "1";

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
}

#[derive(Debug, Clone)]
pub struct PerformanceAttemptRow {
    pub question_id: String,
    pub topic: String,
    pub answered_at_secs: TimestampSecs,
    pub answer: String,
    pub correct: bool,
    pub response_time_ms: u32,
    pub confidence: Option<u32>,
}

pub struct BrainliftStorage {
    db: Connection,
}

impl fmt::Debug for BrainliftStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrainliftStorage").finish_non_exhaustive()
    }
}

impl BrainliftStorage {
    pub fn open(collection_path: &Path) -> Result<Self> {
        let db_path = brainlift_db_path(collection_path);
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
        self.set_meta("schema_version", SCHEMA_VERSION)?;
        let collection_path_str = collection_path.to_string_lossy();
        self.set_meta("collection_path", &collection_path_str)?;
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
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, 0, ?)",
                params![
                    q.id,
                    q.topic,
                    q.section,
                    q.format,
                    q.stem,
                    serde_json::to_string(&q.choices).unwrap(),
                    q.correct_answer,
                    q.explanation,
                    now,
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_questions(&self, limit: u32) -> Result<Vec<StoredQuestion>> {
        let limit = limit.max(1) as i64;
        let mut stmt = self.db.prepare_cached(
            "SELECT id, topic, section, format, stem, choices_json, correct_answer, explanation
             FROM bl_question
             ORDER BY id
             LIMIT ?",
        )?;

        let rows = stmt.query_map([limit], |row| {
            let choices_json: Option<String> = row.get(5)?;
            let choices: Vec<String> = choices_json
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        5,
                        "choices".into(),
                        rusqlite::types::Type::Text,
                    )
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
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn get_question(&self, question_id: &str) -> Result<Option<StoredQuestion>> {
        self.db
            .query_row(
                "SELECT id, topic, section, format, stem, choices_json, correct_answer, explanation
                 FROM bl_question WHERE id = ?",
                [question_id],
                |row| {
                    let choices_json: Option<String> = row.get(5)?;
                    let choices: Vec<String> = choices_json
                        .and_then(|s| serde_json::from_str(&s).ok())
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
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_attempt(
        &self,
        question_id: &str,
        topic: &str,
        answer: &str,
        correct: bool,
        response_time_ms: u32,
        confidence: Option<u32>,
        session_id: Option<&str>,
    ) -> Result<()> {
        let now = TimestampSecs::now().0;
        self.db.execute(
            "INSERT INTO bl_performance_attempt
            (question_id, topic, answered_at_secs, answer, correct, response_time_ms,
             confidence, session_id, usn, mtime_secs)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, -1, ?)",
            params![
                question_id,
                topic,
                now,
                answer,
                i32::from(correct),
                response_time_ms,
                confidence,
                session_id,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn performance_summary(&self) -> Result<(u32, u32)> {
        let correct: i64 = self.db.query_row(
            "SELECT COALESCE(SUM(correct), 0) FROM bl_performance_attempt",
            [],
            |row| row.get(0),
        )?;
        let total: i64 =
            self.db
                .query_row("SELECT COUNT(*) FROM bl_performance_attempt", [], |row| {
                    row.get(0)
                })?;
        Ok((correct as u32, total as u32))
    }

    pub fn recent_attempts(&self, limit: u32) -> Result<Vec<PerformanceAttemptRow>> {
        let limit = limit.max(1) as i64;
        let mut stmt = self.db.prepare_cached(
            "SELECT question_id, topic, answered_at_secs, answer, correct,
                    response_time_ms, confidence
             FROM bl_performance_attempt
             ORDER BY id DESC
             LIMIT ?",
        )?;

        let rows = stmt.query_map([limit], |row| {
            Ok(PerformanceAttemptRow {
                question_id: row.get(0)?,
                topic: row.get(1)?,
                answered_at_secs: TimestampSecs(row.get(2)?),
                answer: row.get(3)?,
                correct: row.get::<_, i32>(4)? != 0,
                response_time_ms: row.get(5)?,
                confidence: row.get(6)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

pub fn brainlift_db_path(collection_path: &Path) -> PathBuf {
    if collection_path.to_string_lossy() == ":memory:" {
        return PathBuf::from("brainlift-memory.db");
    }
    collection_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("brainlift.db")
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::CollectionBuilder;

    #[test]
    fn seeds_and_records_attempts() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let col = CollectionBuilder::new(&col_path).build()?;
        let storage = BrainliftStorage::open(&col.col_path)?;
        let questions = storage.list_questions(10)?;
        assert!(!questions.is_empty());
        let q = &questions[0];
        storage.record_attempt(&q.id, &q.topic, &q.correct_answer, true, 1200, None, None)?;
        let (correct, total) = storage.performance_summary()?;
        assert_eq!(correct, 1);
        assert_eq!(total, 1);
        Ok(())
    }
}
