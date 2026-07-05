// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use anki_proto::brainlift::BrainLiftSyncAttempt;
use anki_proto::brainlift::BrainLiftSyncPullRequest;
use anki_proto::brainlift::BrainLiftSyncPullResponse;
use anki_proto::brainlift::BrainLiftSyncPushRequest;
use anki_proto::brainlift::BrainLiftSyncPushResponse;
use anki_proto::brainlift::BrainLiftSyncStatus;
use anki_proto::brainlift::PerformGreAtlasSyncRequest;
use anki_proto::brainlift::PerformGreAtlasSyncResponse;
use anki_proto::generic::Empty;
use anki_proto::sync::SyncAuth;
use prost::Message;
use serde::Deserialize;
use serde::Serialize;

use crate::backend_method::invoke_proto;

const GRE_ATLAS_SERVICE: &str = "BackendBrainLiftService";

const DEFAULT_PULL_LIMIT: u32 = 500;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncStatusView {
    pub current_usn: i32,
    pub pending_upload_count: u32,
    pub last_modified_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncAttemptView {
    pub id: i64,
    pub question_id: String,
    pub topic: String,
    pub difficulty: Option<f32>,
    pub answered_at_secs: i64,
    pub answer: String,
    pub correct: bool,
    pub response_time_ms: u32,
    pub confidence: Option<u32>,
    pub session_id: Option<String>,
    pub usn: i32,
    pub mtime_secs: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncPullInput {
    pub after_usn: i32,
    #[serde(default = "default_pull_limit")]
    pub limit: u32,
}

fn default_pull_limit() -> u32 {
    DEFAULT_PULL_LIMIT
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncPullView {
    pub attempts: Vec<GreAtlasSyncAttemptView>,
    pub current_usn: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncPushInput {
    pub attempts: Vec<GreAtlasSyncAttemptView>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncConflictView {
    pub attempt_id: i64,
    pub reason: String,
    pub kept: GreAtlasSyncAttemptView,
    pub rejected: GreAtlasSyncAttemptView,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncPushView {
    pub current_usn: i32,
    pub applied_count: u32,
    pub conflicts: Vec<GreAtlasSyncConflictView>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasSyncAuthView {
    pub hkey: String,
    pub endpoint: Option<String>,
    pub io_timeout_secs: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasPerformSyncInput {
    pub auth: Option<GreAtlasSyncAuthView>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAtlasPerformSyncView {
    pub success: bool,
    pub message: String,
    pub downloaded_count: u32,
    pub uploaded_count: u32,
    pub applied_count: u32,
    pub conflicts: Vec<GreAtlasSyncConflictView>,
    pub status: Option<GreAtlasSyncStatusView>,
}

pub fn load_sync_status(backend: &Backend) -> Result<GreAtlasSyncStatusView, Vec<u8>> {
    let status = invoke_proto::<BrainLiftSyncStatus>(
        backend,
        GRE_ATLAS_SERVICE,
        "get_brain_lift_sync_status",
        &Empty::default().encode_to_vec(),
    )?;
    Ok(status.into())
}

pub fn pull_sync_changes(
    backend: &Backend,
    input: GreAtlasSyncPullInput,
) -> Result<GreAtlasSyncPullView, Vec<u8>> {
    let limit = if input.limit == 0 {
        DEFAULT_PULL_LIMIT
    } else {
        input.limit
    };
    let response = invoke_proto::<BrainLiftSyncPullResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "pull_brain_lift_changes",
        &BrainLiftSyncPullRequest {
            after_usn: input.after_usn,
            limit,
        }
        .encode_to_vec(),
    )?;
    Ok(response.into())
}

pub fn push_sync_changes(
    backend: &Backend,
    input: GreAtlasSyncPushInput,
) -> Result<GreAtlasSyncPushView, Vec<u8>> {
    let response = invoke_proto::<BrainLiftSyncPushResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "push_brain_lift_changes",
        &BrainLiftSyncPushRequest {
            attempts: input
                .attempts
                .into_iter()
                .map(GreAtlasSyncAttemptView::into_proto)
                .collect(),
        }
        .encode_to_vec(),
    )?;
    Ok(response.into())
}

pub fn perform_gre_atlas_sync(
    backend: &Backend,
    input: GreAtlasPerformSyncInput,
) -> Result<GreAtlasPerformSyncView, Vec<u8>> {
    let auth = input.auth.map(|a| SyncAuth {
        hkey: a.hkey,
        endpoint: a.endpoint,
        io_timeout_secs: a.io_timeout_secs,
    });
    let response = invoke_proto::<PerformGreAtlasSyncResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "perform_gre_atlas_sync",
        &PerformGreAtlasSyncRequest { auth }.encode_to_vec(),
    )?;
    Ok(response.into())
}

impl From<BrainLiftSyncStatus> for GreAtlasSyncStatusView {
    fn from(status: BrainLiftSyncStatus) -> Self {
        Self {
            current_usn: status.current_usn,
            pending_upload_count: status.pending_upload_count,
            last_modified_secs: status.last_modified_secs,
        }
    }
}

impl From<BrainLiftSyncPullResponse> for GreAtlasSyncPullView {
    fn from(response: BrainLiftSyncPullResponse) -> Self {
        Self {
            attempts: response
                .attempts
                .into_iter()
                .map(GreAtlasSyncAttemptView::from_proto)
                .collect(),
            current_usn: response.current_usn,
        }
    }
}

impl From<BrainLiftSyncPushResponse> for GreAtlasSyncPushView {
    fn from(response: BrainLiftSyncPushResponse) -> Self {
        Self {
            current_usn: response.current_usn,
            applied_count: response.applied_count,
            conflicts: response
                .conflicts
                .into_iter()
                .map(GreAtlasSyncConflictView::from_proto)
                .collect(),
        }
    }
}

impl From<PerformGreAtlasSyncResponse> for GreAtlasPerformSyncView {
    fn from(response: PerformGreAtlasSyncResponse) -> Self {
        Self {
            success: response.success,
            message: response.message,
            downloaded_count: response.downloaded_count,
            uploaded_count: response.uploaded_count,
            applied_count: response.applied_count,
            conflicts: response
                .conflicts
                .into_iter()
                .map(GreAtlasSyncConflictView::from_proto)
                .collect(),
            status: response.status.map(Into::into),
        }
    }
}

impl GreAtlasSyncPullView {
    #[cfg(test)]
    pub fn normalize_for_parity(mut self) -> Self {
        self.attempts
            .sort_by(|a, b| a.question_id.cmp(&b.question_id));
        for attempt in &mut self.attempts {
            attempt.id = 0;
            attempt.usn = 0;
            attempt.mtime_secs = 0;
            attempt.session_id = None;
        }
        self.current_usn = 0;
        self
    }
}

impl GreAtlasSyncAttemptView {
    fn from_proto(value: BrainLiftSyncAttempt) -> Self {
        Self {
            id: value.id,
            question_id: value.question_id,
            topic: value.topic,
            difficulty: value.difficulty,
            answered_at_secs: value.answered_at_secs,
            answer: value.answer,
            correct: value.correct,
            response_time_ms: value.response_time_ms,
            confidence: value.confidence,
            session_id: value.session_id,
            usn: value.usn,
            mtime_secs: value.mtime_secs,
        }
    }

    fn into_proto(self) -> BrainLiftSyncAttempt {
        BrainLiftSyncAttempt {
            id: self.id,
            question_id: self.question_id,
            topic: self.topic,
            difficulty: self.difficulty,
            answered_at_secs: self.answered_at_secs,
            answer: self.answer,
            correct: self.correct,
            response_time_ms: self.response_time_ms,
            confidence: self.confidence,
            session_id: self.session_id,
            usn: self.usn,
            mtime_secs: self.mtime_secs,
        }
    }
}

impl GreAtlasSyncConflictView {
    fn from_proto(value: anki_proto::brainlift::BrainLiftSyncConflict) -> Self {
        Self {
            attempt_id: value.attempt_id,
            reason: value.reason,
            kept: value
                .kept
                .map(GreAtlasSyncAttemptView::from_proto)
                .unwrap_or_else(|| GreAtlasSyncAttemptView {
                    id: value.attempt_id,
                    question_id: String::new(),
                    topic: String::new(),
                    difficulty: None,
                    answered_at_secs: 0,
                    answer: String::new(),
                    correct: false,
                    response_time_ms: 0,
                    confidence: None,
                    session_id: None,
                    usn: 0,
                    mtime_secs: 0,
                }),
            rejected: value
                .rejected
                .map(GreAtlasSyncAttemptView::from_proto)
                .unwrap_or_else(|| GreAtlasSyncAttemptView {
                    id: value.attempt_id,
                    question_id: String::new(),
                    topic: String::new(),
                    difficulty: None,
                    answered_at_secs: 0,
                    answer: String::new(),
                    correct: false,
                    response_time_ms: 0,
                    confidence: None,
                    session_id: None,
                    usn: 0,
                    mtime_secs: 0,
                }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn attempt_roundtrip_preserves_fields() {
        let view = GreAtlasSyncAttemptView {
            id: 7,
            question_id: "q1".into(),
            topic: "algebra".into(),
            difficulty: Some(0.5),
            answered_at_secs: 1_700_000_000,
            answer: "B".into(),
            correct: true,
            response_time_ms: 900,
            confidence: Some(4),
            session_id: Some("sess".into()),
            usn: 3,
            mtime_secs: 1_700_000_001,
        };
        let proto = view.clone().into_proto();
        let restored = GreAtlasSyncAttemptView::from_proto(proto);
        assert_eq!(view, restored);
    }
}
