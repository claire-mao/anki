// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::BrainLiftSyncAttempt;
use anki_proto::brainlift::BrainLiftSyncConflict;
use anki_proto::brainlift::BrainLiftSyncPullResponse;
use anki_proto::brainlift::BrainLiftSyncPushResponse;
use anki_proto::brainlift::BrainLiftSyncStatus;

use crate::brainlift::brainlift_storage;
use crate::brainlift::storage::SyncAttemptRow;
use crate::collection::Collection;
use crate::error::Result;

impl Collection {
    pub fn brainlift_sync_status(&mut self) -> Result<BrainLiftSyncStatus> {
        let storage = brainlift_storage(self)?;
        Ok(storage.sync_status()?.into())
    }

    pub fn brainlift_pull_changes(
        &mut self,
        after_usn: i32,
        limit: u32,
    ) -> Result<BrainLiftSyncPullResponse> {
        let storage = brainlift_storage(self)?;
        let (attempts, current_usn) = storage.pull_changes(after_usn, limit)?;
        Ok(BrainLiftSyncPullResponse {
            attempts: attempts.into_iter().map(sync_attempt_to_proto).collect(),
            current_usn,
        })
    }

    pub fn brainlift_push_changes(
        &mut self,
        incoming: Vec<BrainLiftSyncAttempt>,
    ) -> Result<BrainLiftSyncPushResponse> {
        let storage = brainlift_storage(self)?;
        let rows: Vec<SyncAttemptRow> = incoming.into_iter().map(SyncAttemptRow::from).collect();
        let result = storage.push_changes(&rows)?;
        Ok(BrainLiftSyncPushResponse {
            current_usn: result.current_usn,
            conflicts: result
                .conflicts
                .into_iter()
                .map(|c| anki_proto::brainlift::BrainLiftSyncConflict {
                    attempt_id: c.attempt_id,
                    reason: c.reason,
                    kept: Some(sync_attempt_to_proto(c.kept)),
                    rejected: Some(sync_attempt_to_proto(c.rejected)),
                })
                .collect(),
            applied_count: result.applied_count,
        })
    }
}

fn sync_attempt_to_proto(row: SyncAttemptRow) -> BrainLiftSyncAttempt {
    BrainLiftSyncAttempt {
        id: row.id,
        question_id: row.question_id,
        topic: row.topic,
        difficulty: row.difficulty,
        answered_at_secs: row.answered_at_secs.0,
        answer: row.answer,
        correct: row.correct,
        response_time_ms: row.response_time_ms,
        confidence: row.confidence,
        session_id: row.session_id,
        usn: row.usn,
        mtime_secs: row.mtime_secs.0,
    }
}

impl From<crate::brainlift::storage::SyncStatus> for BrainLiftSyncStatus {
    fn from(status: crate::brainlift::storage::SyncStatus) -> Self {
        BrainLiftSyncStatus {
            current_usn: status.current_usn,
            pending_upload_count: status.pending_upload_count,
            last_modified_secs: status.last_modified_secs.0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::brainlift::brainlift_storage;
    use crate::collection::CollectionBuilder;

    fn isolated_col() -> Result<Collection> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        CollectionBuilder::new(dir.path().join("test.anki2")).build()
    }

    #[test]
    fn pull_after_local_record_includes_change() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = brainlift_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            &q.correct_answer,
            true,
            900,
            None,
            Some(&session.id),
        )?;
        let status = col.brainlift_sync_status()?;
        assert!(status.current_usn > 0);
        let pulled = col.brainlift_pull_changes(0, 100)?;
        assert_eq!(pulled.attempts.len(), 1);
        assert_eq!(pulled.current_usn, status.current_usn);
        Ok(())
    }

    #[test]
    fn push_applies_remote_and_keeps_newer_on_conflict() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = brainlift_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "local",
            false,
            500,
            None,
            Some(&session.id),
        )?;
        let local = col.brainlift_pull_changes(0, 10)?;
        let mut remote = local.attempts[0].clone();
        remote.answer = "remote-newer".into();
        remote.mtime_secs += 100;
        remote.correct = true;

        let push = col.brainlift_push_changes(vec![remote])?;
        assert_eq!(push.applied_count, 1);
        assert!(push.conflicts.is_empty());

        let updated = col.brainlift_pull_changes(0, 10)?;
        assert_eq!(updated.attempts[0].answer, "remote-newer");
        assert!(updated.attempts[0].correct);
        Ok(())
    }

    #[test]
    fn push_rejects_older_remote_with_conflict() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = brainlift_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "newer-local",
            true,
            500,
            None,
            Some(&session.id),
        )?;
        let local = col.brainlift_pull_changes(0, 10)?;
        let mut stale = local.attempts[0].clone();
        stale.answer = "stale".into();
        stale.mtime_secs -= 500;
        stale.correct = false;

        let push = col.brainlift_push_changes(vec![stale])?;
        assert_eq!(push.applied_count, 0);
        assert_eq!(push.conflicts.len(), 1);
        assert!(push.conflicts[0].reason.contains("newer"));
        Ok(())
    }
}
