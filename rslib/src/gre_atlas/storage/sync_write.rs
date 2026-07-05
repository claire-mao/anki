// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Instrumented SQLite writes on the GRE Atlas sync apply path.

use rusqlite::Connection;
use rusqlite::ErrorCode;
use rusqlite::Params;
use tracing::debug;

use crate::error::AnkiError;
use crate::error::Result;

pub(crate) fn log_sync_write(table: &str, op: &str, detail: &str) {
    debug!(target: "gre_atlas::sync", table, op, detail);
}

/// Optional FK child columns for richer constraint-violation diagnostics.
#[derive(Debug, Clone, Default)]
pub(crate) struct SyncFkContext {
    pub question_id: Option<String>,
    pub session_id: Option<String>,
    pub attempt_id: Option<i64>,
}

/// Execute a sync-path write with tracing and FK-aware error context.
pub(crate) fn sync_execute(
    db: &Connection,
    table: &str,
    op: &str,
    detail: &str,
    sql: &str,
    params: impl Params,
    fk: &SyncFkContext,
) -> Result<usize> {
    log_sync_write(table, op, detail);
    db.execute(sql, params).map_err(|err| {
        if is_foreign_key_violation(&err) {
            fk_violation_error(db, table, op, detail, sql, fk, &err)
        } else {
            err.into()
        }
    })
}

fn is_foreign_key_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == ErrorCode::ConstraintViolation && code.extended_code == 787
    )
}

fn fk_violation_error(
    db: &Connection,
    table: &str,
    op: &str,
    detail: &str,
    sql: &str,
    fk: &SyncFkContext,
    err: &rusqlite::Error,
) -> AnkiError {
    let mut parts = vec![
        format!("SQLite FOREIGN KEY constraint failed during {op} on {table}"),
        format!("sql: {sql}"),
        format!("detail: {detail}"),
    ];

    if let Some(question_id) = &fk.question_id {
        let parent_exists = db
            .query_row(
                "SELECT 1 FROM bl_question WHERE id = ?",
                [question_id.as_str()],
                |_| Ok(()),
            )
            .is_ok();
        parts.push(format!(
            "foreign_key: bl_performance_attempt.question_id -> bl_question.id \
             child.question_id={question_id} parent_exists={parent_exists}"
        ));
    }

    if let Some(session_id) = &fk.session_id {
        let parent_exists = db
            .query_row(
                "SELECT 1 FROM bl_session WHERE id = ?",
                [session_id.as_str()],
                |_| Ok(()),
            )
            .is_ok();
        parts.push(format!(
            "foreign_key: bl_performance_attempt.session_id -> bl_session.id \
             child.session_id={session_id} parent_exists={parent_exists}"
        ));
    }

    if let Some(attempt_id) = fk.attempt_id {
        parts.push(format!("child.attempt_id={attempt_id}"));
    }

    parts.push(format!("sqlite: {err}"));

    AnkiError::InvalidInput {
        source: snafu::FromString::without_source(parts.join("; ")),
    }
}
