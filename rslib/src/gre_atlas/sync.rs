// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::BrainLiftSyncAttempt;
use anki_proto::brainlift::BrainLiftSyncBundle;
use anki_proto::brainlift::BrainLiftSyncBundlePushResponse;
use anki_proto::brainlift::BrainLiftSyncBundleResponse;
use anki_proto::brainlift::BrainLiftSyncPullResponse;
use anki_proto::brainlift::BrainLiftSyncPushResponse;
use anki_proto::brainlift::BrainLiftSyncStatus;
use anki_proto::brainlift::PerformGreAtlasSyncResponse;
use reqwest::Client;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::storage::SyncAttemptRow;
use crate::gre_atlas::sync_transport::bundle_to_proto;
use crate::gre_atlas::sync_transport::proto_to_bundle;
use crate::gre_atlas::sync_transport::GreAtlasSyncTransport;
use crate::sync::login::SyncAuth;

const DEFAULT_BUNDLE_LIMIT: u32 = 5000;

impl Collection {
    pub fn gre_atlas_sync_status(&mut self) -> Result<BrainLiftSyncStatus> {
        let storage = gre_atlas_storage(self)?;
        Ok(storage.sync_status()?.into())
    }

    pub fn gre_atlas_pull_changes(
        &mut self,
        after_usn: i32,
        limit: u32,
    ) -> Result<BrainLiftSyncPullResponse> {
        let storage = gre_atlas_storage(self)?;
        let (attempts, current_usn) = storage.pull_changes(after_usn, limit)?;
        Ok(BrainLiftSyncPullResponse {
            attempts: attempts.into_iter().map(sync_attempt_to_proto).collect(),
            current_usn,
        })
    }

    pub fn gre_atlas_push_changes(
        &mut self,
        incoming: Vec<BrainLiftSyncAttempt>,
    ) -> Result<BrainLiftSyncPushResponse> {
        let storage = gre_atlas_storage(self)?;
        let rows: Vec<SyncAttemptRow> = incoming.into_iter().map(SyncAttemptRow::from).collect();
        let result = storage.push_changes(&rows)?;
        Ok(BrainLiftSyncPushResponse {
            current_usn: result.current_usn,
            conflicts: result
                .conflicts
                .into_iter()
                .map(sync_conflict_to_proto)
                .collect(),
            applied_count: result.applied_count,
        })
    }

    pub fn gre_atlas_pull_sync_bundle(
        &mut self,
        after_usn: i32,
        limit: u32,
    ) -> Result<BrainLiftSyncBundleResponse> {
        let storage = gre_atlas_storage(self)?;
        let bundle = storage.pull_sync_bundle(after_usn, limit)?;
        Ok(BrainLiftSyncBundleResponse {
            bundle: Some(bundle_to_proto(&bundle)),
        })
    }

    pub fn gre_atlas_push_sync_bundle(
        &mut self,
        bundle: BrainLiftSyncBundle,
    ) -> Result<BrainLiftSyncBundlePushResponse> {
        let storage = gre_atlas_storage(self)?;
        let incoming = proto_to_bundle(bundle);
        let result = storage.apply_sync_bundle(&incoming)?;
        Ok(BrainLiftSyncBundlePushResponse {
            current_usn: result.current_usn,
            applied_count: result.applied_count,
            conflicts: result
                .conflicts
                .into_iter()
                .map(sync_conflict_to_proto)
                .collect(),
        })
    }

    /// Download remote GRE Atlas changes, merge locally, upload pending local
    /// changes, and refresh sync bookkeeping.
    pub async fn gre_atlas_perform_sync(
        &mut self,
        auth: SyncAuth,
        client: Client,
    ) -> Result<PerformGreAtlasSyncResponse> {
        let endpoint = auth
            .endpoint
            .clone()
            .unwrap_or_else(|| reqwest::Url::try_from("https://sync.ankiweb.net/").unwrap());
        let storage = gre_atlas_storage(self)?;

        if !crate::gre_atlas::sync_transport::endpoint_supports_gre_atlas_sync(&endpoint) {
            let status = storage.sync_status()?;
            let message = if status.pending_upload_count > 0 {
                crate::gre_atlas::sync_transport::GRE_ATLAS_SYNC_UNAVAILABLE_MSG.to_string()
            } else {
                String::new()
            };
            return Ok(PerformGreAtlasSyncResponse {
                success: status.pending_upload_count == 0,
                message,
                downloaded_count: 0,
                uploaded_count: 0,
                applied_count: 0,
                conflicts: vec![],
                status: Some(status.into()),
            });
        }

        let transport = GreAtlasSyncTransport::new(auth, client);

        let last_downloaded = storage.last_downloaded_usn()?;
        let last_pushed = storage.last_pushed_usn()?;

        let mut downloaded_count = 0u32;
        let mut applied_count = 0u32;
        let mut conflicts = Vec::new();

        if let Some(remote) = transport.download(last_downloaded).await? {
            downloaded_count = (remote.sessions.len()
                + remote.questions.len()
                + remote.attempts.len()
                + remote.predictions.len()) as u32;
            let merge = storage.apply_sync_bundle(&remote)?;
            applied_count += merge.applied_count;
            conflicts.extend(merge.conflicts.into_iter().map(sync_conflict_to_proto));
            storage.set_last_downloaded_usn(remote.current_usn)?;
        }

        let pending = storage.pull_sync_bundle(last_pushed, DEFAULT_BUNDLE_LIMIT)?;
        let uploaded_count = (pending.sessions.len()
            + pending.questions.len()
            + pending.attempts.len()
            + pending.predictions.len()) as u32;

        if uploaded_count > 0 {
            let uploaded = transport.upload(&pending).await?;
            applied_count += uploaded;
        }

        let status = storage.sync_status()?;
        storage.mark_synced_through(status.current_usn)?;
        let status = storage.sync_status()?;

        // Invalidate cached GRE signals after sync.
        self.state.gre_atlas_signals_cache = None;

        Ok(PerformGreAtlasSyncResponse {
            success: true,
            message: String::new(),
            downloaded_count,
            uploaded_count,
            applied_count,
            conflicts,
            status: Some(status.into()),
        })
    }

    pub fn gre_atlas_perform_sync_offline(&mut self) -> Result<PerformGreAtlasSyncResponse> {
        let status = gre_atlas_storage(self)?.sync_status()?;
        Ok(PerformGreAtlasSyncResponse {
            success: false,
            message: "Sync credentials not configured. Sign in on desktop with a self-hosted Anki sync server (AnkiWeb does not support GRE Atlas practice sync)."
                .into(),
            downloaded_count: 0,
            uploaded_count: 0,
            applied_count: 0,
            conflicts: vec![],
            status: Some(status.into()),
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
        session_id: row.session_id.clone(),
        usn: row.usn,
        mtime_secs: row.mtime_secs.0,
    }
}

fn sync_conflict_to_proto(
    c: crate::gre_atlas::storage::SyncConflict,
) -> anki_proto::brainlift::BrainLiftSyncConflict {
    anki_proto::brainlift::BrainLiftSyncConflict {
        attempt_id: c.attempt_id,
        reason: c.reason,
        kept: Some(sync_attempt_to_proto(c.kept)),
        rejected: Some(sync_attempt_to_proto(c.rejected)),
    }
}

impl From<crate::gre_atlas::storage::SyncStatus> for BrainLiftSyncStatus {
    fn from(status: crate::gre_atlas::storage::SyncStatus) -> Self {
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
    use crate::collection::CollectionBuilder;
    use crate::gre_atlas::gre_atlas_storage;
    use crate::gre_atlas::storage::SyncAttemptRow;
    use crate::timestamp::TimestampSecs;

    fn isolated_col() -> Result<Collection> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        CollectionBuilder::new(dir.path().join("test.anki2")).build()
    }

    #[test]
    fn pull_after_local_record_includes_change() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
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
        let status = col.gre_atlas_sync_status()?;
        assert!(status.current_usn > 0);
        let pulled = col.gre_atlas_pull_changes(0, 100)?;
        assert_eq!(pulled.attempts.len(), 1);
        assert_eq!(pulled.current_usn, status.current_usn);
        Ok(())
    }

    #[test]
    fn push_applies_remote_and_keeps_newer_on_conflict() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
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
        let local = col.gre_atlas_pull_changes(0, 10)?;
        let mut remote = local.attempts[0].clone();
        remote.answer = "remote-newer".into();
        remote.mtime_secs += 100;
        remote.correct = true;

        let push = col.gre_atlas_push_changes(vec![remote])?;
        assert_eq!(push.applied_count, 1);
        assert!(push.conflicts.is_empty());

        let updated = col.gre_atlas_pull_changes(0, 10)?;
        assert_eq!(updated.attempts[0].answer, "remote-newer");
        assert!(updated.attempts[0].correct);
        Ok(())
    }

    #[test]
    fn push_rejects_older_remote_with_conflict() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
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
        let local = col.gre_atlas_pull_changes(0, 10)?;
        let mut stale = local.attempts[0].clone();
        stale.answer = "stale".into();
        stale.mtime_secs -= 500;
        stale.correct = false;

        let push = col.gre_atlas_push_changes(vec![stale])?;
        assert_eq!(push.applied_count, 0);
        assert_eq!(push.conflicts.len(), 1);
        assert!(push.conflicts[0].reason.contains("newer"));
        Ok(())
    }

    /// Simulates desktop → mobile transfer using the full sync bundle (sessions
    /// first).
    #[test]
    fn bundle_sync_desktop_to_mobile_preserves_attempts() -> Result<()> {
        let mut desktop = isolated_col()?;
        let d_storage = gre_atlas_storage(&mut desktop)?;
        let q = d_storage.list_questions("", 1)?.pop().unwrap();
        let session = d_storage.create_session("practice")?;
        d_storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "desktop-answer",
            true,
            700,
            None,
            Some(&session.id),
        )?;
        let export = d_storage.pull_sync_bundle(0, 5000)?;

        let mut mobile = isolated_col()?;
        let m_storage = gre_atlas_storage(&mut mobile)?;
        let applied = m_storage.apply_sync_bundle(&export)?;
        assert!(applied.applied_count >= 2);
        assert!(m_storage.performance_summary()?.1 >= 1);
        Ok(())
    }

    /// Simulates bidirectional offline edits merged by mtime_secs.
    #[test]
    fn bundle_sync_bidirectional_last_write_wins() -> Result<()> {
        let mut col_a = isolated_col()?;
        let storage_a = gre_atlas_storage(&mut col_a)?;
        let q = storage_a.list_questions("", 1)?.pop().unwrap();
        let session_a = storage_a.create_session("practice")?;
        storage_a.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "device-a",
            true,
            600,
            None,
            Some(&session_a.id),
        )?;
        let bundle_a = storage_a.pull_sync_bundle(0, 5000)?;

        let mut col_b = isolated_col()?;
        let storage_b = gre_atlas_storage(&mut col_b)?;
        storage_b.apply_sync_bundle(&bundle_a)?;
        let session_b = storage_b.create_session("practice")?;
        storage_b.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "device-b",
            false,
            800,
            None,
            Some(&session_b.id),
        )?;
        assert!(storage_b.performance_summary()?.1 >= 2);

        let bundle_b = storage_b.pull_sync_bundle(0, 5000)?;
        storage_a.apply_sync_bundle(&bundle_b)?;
        assert!(storage_a.performance_summary()?.1 >= 2);
        Ok(())
    }

    #[test]
    fn mark_synced_through_clears_pending_upload_count() -> Result<()> {
        let mut col = isolated_col()?;
        {
            let storage = gre_atlas_storage(&mut col)?;
            let q = storage.list_questions("", 1)?.pop().unwrap();
            let session = storage.create_session("practice")?;
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                "A",
                true,
                500,
                None,
                Some(&session.id),
            )?;
        }
        let before = col.gre_atlas_sync_status()?;
        assert!(before.pending_upload_count > 0);
        gre_atlas_storage(&mut col)?.mark_synced_through(before.current_usn)?;
        let after = col.gre_atlas_sync_status()?;
        assert_eq!(after.pending_upload_count, 0);
        Ok(())
    }

    #[test]
    fn perform_sync_on_ankiweb_returns_unsupported_message() -> Result<()> {
        let mut col = isolated_col()?;
        {
            let storage = gre_atlas_storage(&mut col)?;
            let q = storage.list_questions("", 1)?.pop().unwrap();
            let session = storage.create_session("practice")?;
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                "A",
                true,
                500,
                None,
                Some(&session.id),
            )?;
        }

        let rt =
            tokio::runtime::Runtime::new().map_err(|e| crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(e.to_string()),
            })?;
        let auth = SyncAuth {
            hkey: "test-hkey".into(),
            endpoint: Some(
                reqwest::Url::parse("https://sync.ankiweb.net/").map_err(|e| {
                    crate::error::AnkiError::InvalidInput {
                        source: snafu::FromString::without_source(e.to_string()),
                    }
                })?,
            ),
            io_timeout_secs: Some(30),
        };
        let response = rt.block_on(col.gre_atlas_perform_sync(auth, reqwest::Client::new()))?;
        assert!(!response.success);
        assert!(
            response.message.contains("self-hosted"),
            "unexpected message: {}",
            response.message
        );
        assert!(response.status.as_ref().unwrap().pending_upload_count > 0);
        Ok(())
    }

    fn attempt_count(storage: &crate::gre_atlas::storage::GreAtlasStorage) -> Result<u32> {
        Ok(storage.performance_summary()?.1)
    }

    fn session_count(storage: &crate::gre_atlas::storage::GreAtlasStorage) -> Result<u64> {
        storage.session_count()
    }

    /// Profile A → B → A via `apply_sync_bundle`, including stale-session
    /// export.
    #[test]
    fn two_profile_roundtrip_preserves_attempts_without_fk_or_duplicate_sessions() -> Result<()> {
        let mut col_a = isolated_col()?;
        let storage_a = gre_atlas_storage(&mut col_a)?;
        let q = storage_a.list_questions("", 1)?.pop().unwrap();
        let session_a = storage_a.create_session("practice")?;
        storage_a.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            "profile-a-first",
            true,
            600,
            None,
            Some(&session_a.id),
        )?;

        let mut col_b = isolated_col()?;
        let storage_b = gre_atlas_storage(&mut col_b)?;
        storage_b.apply_sync_bundle(&storage_a.pull_sync_bundle(0, 5000)?)?;
        assert_eq!(attempt_count(storage_b)?, 1);
        assert_eq!(session_count(storage_b)?, 1);

        storage_a.mark_synced_through(storage_a.sync_status()?.current_usn)?;
        let q2 = storage_a.list_questions("", 2)?.into_iter().nth(1).unwrap();
        storage_a.record_attempt(
            &q2.id,
            &q2.topic,
            q2.difficulty,
            "profile-a-second",
            false,
            700,
            None,
            Some(&session_a.id),
        )?;
        let incremental = storage_a.pull_sync_bundle(storage_a.last_pushed_usn()?, 5000)?;
        assert!(
            incremental.sessions.iter().any(|s| s.id == session_a.id),
            "stale session must ride along with new attempt"
        );
        storage_b.apply_sync_bundle(&incremental)?;
        assert_eq!(attempt_count(storage_b)?, 2);
        assert_eq!(session_count(storage_b)?, 1);

        let session_b = storage_b.create_session("practice")?;
        let q_b = storage_b.list_questions("", 1)?.pop().unwrap();
        storage_b.record_attempt(
            &q_b.id,
            &q_b.topic,
            q_b.difficulty,
            "profile-b",
            true,
            800,
            None,
            Some(&session_b.id),
        )?;
        storage_a.apply_sync_bundle(&storage_b.pull_sync_bundle(0, 5000)?)?;
        assert_eq!(attempt_count(storage_a)?, 3);
        assert_eq!(session_count(storage_a)?, 2);
        Ok(())
    }

    #[test]
    fn push_changes_without_session_row_creates_session_stub() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let remote_session = "remote-only-session".to_string();
        let row = SyncAttemptRow {
            id: 0,
            question_id: q.id.clone(),
            topic: q.topic.clone(),
            difficulty: q.difficulty,
            answered_at_secs: TimestampSecs(1_700_000_200),
            answer: "remote".into(),
            correct: true,
            response_time_ms: 400,
            confidence: None,
            session_id: Some(remote_session.clone()),
            usn: 1,
            mtime_secs: TimestampSecs(1_700_000_200),
        };
        storage.push_changes(&[row])?;
        assert!(storage.session_exists(&remote_session)?);
        assert_eq!(attempt_count(storage)?, 1);
        Ok(())
    }

    #[test]
    fn push_cross_device_id_collision_inserts_as_new_attempt() -> Result<()> {
        let mut col = isolated_col()?;
        let q2 = {
            let storage = gre_atlas_storage(&mut col)?;
            let q1 = storage.list_questions("", 1)?.pop().unwrap();
            let q2 = storage.list_questions("", 2)?.into_iter().nth(1).unwrap();
            let session_local = storage.create_session("practice")?;
            storage.record_attempt(
                &q1.id,
                &q1.topic,
                q1.difficulty,
                "local-device",
                true,
                600,
                None,
                Some(&session_local.id),
            )?;
            q2
        };
        let local = col.gre_atlas_pull_changes(0, 10)?;
        assert_eq!(local.attempts.len(), 1);
        assert_eq!(local.attempts[0].id, 1);

        let remote_session = "remote-device-session".to_string();
        let remote = SyncAttemptRow {
            id: 1,
            question_id: q2.id.clone(),
            topic: q2.topic.clone(),
            difficulty: q2.difficulty,
            answered_at_secs: TimestampSecs(1_700_000_500),
            answer: "remote-device".into(),
            correct: false,
            response_time_ms: 700,
            confidence: None,
            session_id: Some(remote_session.clone()),
            usn: 99,
            mtime_secs: TimestampSecs(1_700_000_500),
        };
        let push = {
            let storage = gre_atlas_storage(&mut col)?;
            storage.push_changes(&[remote])?
        };
        let storage = gre_atlas_storage(&mut col)?;
        assert_eq!(push.applied_count, 1);
        assert!(push.conflicts.is_empty());
        assert_eq!(attempt_count(storage)?, 2);
        assert!(storage.session_exists(&remote_session)?);

        let all = col.gre_atlas_pull_changes(-1, 100)?;
        assert_eq!(all.attempts.len(), 2);
        let answers: Vec<_> = all.attempts.iter().map(|a| a.answer.as_str()).collect();
        assert!(answers.contains(&"local-device"));
        assert!(answers.contains(&"remote-device"));
        Ok(())
    }

    #[test]
    fn push_skips_duplicate_identity_with_different_id() -> Result<()> {
        let mut col = isolated_col()?;
        {
            let storage = gre_atlas_storage(&mut col)?;
            let q = storage.list_questions("", 1)?.pop().unwrap();
            let session = storage.create_session("practice")?;
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                "original",
                true,
                600,
                None,
                Some(&session.id),
            )?;
        }
        let local_attempt = gre_atlas_storage(&mut col)?.pull_changes(-1, 10)?.0[0].clone();
        let remapped = SyncAttemptRow {
            id: 99,
            ..local_attempt
        };
        let push = {
            let storage = gre_atlas_storage(&mut col)?;
            storage.push_changes(&[remapped])?
        };
        assert_eq!(push.applied_count, 0);
        assert_eq!(attempt_count(gre_atlas_storage(&mut col)?)?, 1);
        Ok(())
    }

    #[test]
    fn push_changes_without_question_row_creates_question_stub() -> Result<()> {
        let mut col = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let session = storage.create_session("practice")?;
        let row = SyncAttemptRow {
            id: 0,
            question_id: "remote-only-question".into(),
            topic: "gre::quant::algebra".into(),
            difficulty: Some(0.5),
            answered_at_secs: TimestampSecs(1_700_000_300),
            answer: "42".into(),
            correct: true,
            response_time_ms: 400,
            confidence: None,
            session_id: Some(session.id.clone()),
            usn: 1,
            mtime_secs: TimestampSecs(1_700_000_300),
        };
        storage.push_changes(&[row])?;
        assert!(storage.question_exists("remote-only-question")?);
        assert_eq!(attempt_count(storage)?, 1);
        Ok(())
    }
}
