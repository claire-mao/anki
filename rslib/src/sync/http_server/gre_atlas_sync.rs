// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! HTTP handlers for GRE Atlas sidecar sync on the self-hosted sync server.

use std::path::Path;
use std::sync::Arc;

use anki_io::read_file;
use anki_io::write_file;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use prost::Message;

use crate::gre_atlas::sync_transport::bundle_to_proto;
use crate::gre_atlas::sync_transport::proto_to_bundle;
use crate::sync::error::HttpResult;
use crate::sync::error::OrHttpErr;
use crate::sync::http_server::SimpleServer;
use crate::sync::request::SyncRequest;

const GRE_ATLAS_SYNC_FILE: &str = "gre_atlas_sync.pb";

#[derive(serde::Deserialize)]
struct GreAtlasDownloadRequest {
    after_usn: i32,
}

#[derive(serde::Serialize)]
struct GreAtlasDownloadResponse {
    bundle: Vec<u8>,
}

#[derive(serde::Serialize)]
struct GreAtlasUploadResponse {
    current_usn: i32,
    applied_count: u32,
}

pub fn gre_atlas_sync_router() -> Router<Arc<SimpleServer>> {
    Router::new()
        .route("/download", post(gre_atlas_download))
        .route("/upload", post(gre_atlas_upload))
}

async fn gre_atlas_download(
    State(server): State<Arc<SimpleServer>>,
    request: SyncRequest<Vec<u8>>,
) -> HttpResult<Response> {
    server
        .with_authenticated_user(request, |user, req| {
            let payload: GreAtlasDownloadRequest =
                serde_json::from_slice(&req.data).or_bad_request("parse download request")?;
            let path = user.folder.join(GRE_ATLAS_SYNC_FILE);
            let bundle = load_server_bundle(&path)?;
            let filtered = filter_bundle_after_usn(bundle, payload.after_usn);
            let bytes = bundle_to_proto(&filtered).encode_to_vec();
            let response = GreAtlasDownloadResponse { bundle: bytes };
            Ok(axum::Json(response).into_response())
        })
        .await
}

async fn gre_atlas_upload(
    State(server): State<Arc<SimpleServer>>,
    request: SyncRequest<Vec<u8>>,
) -> HttpResult<Response> {
    server
        .with_authenticated_user(request, |user, req| {
            let incoming = anki_proto::brainlift::BrainLiftSyncBundle::decode(req.data.as_slice())
                .or_bad_request("decode upload bundle")?;
            let incoming = proto_to_bundle(incoming);
            let path = user.folder.join(GRE_ATLAS_SYNC_FILE);
            let mut stored = load_server_bundle(&path)?;
            let applied = merge_server_bundle(&mut stored, &incoming);
            save_server_bundle(&path, &stored)?;
            let response = GreAtlasUploadResponse {
                current_usn: stored.current_usn,
                applied_count: applied,
            };
            Ok(axum::Json(response).into_response())
        })
        .await
}

fn load_server_bundle(path: &Path) -> HttpResult<crate::gre_atlas::storage::SyncBundle> {
    if !path.is_file() {
        return Ok(crate::gre_atlas::storage::SyncBundle::default());
    }
    let bytes = read_file(path).or_internal("read gre atlas sync file")?;
    if bytes.is_empty() {
        return Ok(crate::gre_atlas::storage::SyncBundle::default());
    }
    let proto = anki_proto::brainlift::BrainLiftSyncBundle::decode(bytes.as_slice())
        .or_internal("decode stored gre atlas bundle")?;
    Ok(proto_to_bundle(proto))
}

fn save_server_bundle(
    path: &Path,
    bundle: &crate::gre_atlas::storage::SyncBundle,
) -> HttpResult<()> {
    let bytes = bundle_to_proto(bundle).encode_to_vec();
    write_file(path, bytes).or_internal("write gre atlas sync file")?;
    Ok(())
}

fn filter_bundle_after_usn(
    mut bundle: crate::gre_atlas::storage::SyncBundle,
    after_usn: i32,
) -> crate::gre_atlas::storage::SyncBundle {
    bundle.sessions.retain(|r| r.usn > after_usn);
    bundle.questions.retain(|r| r.usn > after_usn);
    bundle.attempts.retain(|r| r.usn > after_usn);
    bundle.predictions.retain(|r| r.usn > after_usn);
    bundle
}

fn merge_server_bundle(
    stored: &mut crate::gre_atlas::storage::SyncBundle,
    incoming: &crate::gre_atlas::storage::SyncBundle,
) -> u32 {
    let mut applied = 0u32;
    for row in &incoming.sessions {
        if merge_session_row(stored, row) {
            applied += 1;
        }
    }
    for row in &incoming.questions {
        if merge_question_row(stored, row) {
            applied += 1;
        }
    }
    for row in &incoming.attempts {
        if merge_attempt_row(stored, row) {
            applied += 1;
        }
    }
    for row in &incoming.predictions {
        if merge_prediction_row(stored, row) {
            applied += 1;
        }
    }
    stored.current_usn = stored.current_usn.max(incoming.current_usn);
    if incoming.last_modified_secs.0 > stored.last_modified_secs.0 {
        stored.last_modified_secs = incoming.last_modified_secs;
    }
    applied
}

fn merge_session_row(
    stored: &mut crate::gre_atlas::storage::SyncBundle,
    row: &crate::gre_atlas::storage::SyncSessionRow,
) -> bool {
    if let Some(existing) = stored.sessions.iter_mut().find(|s| s.id == row.id) {
        if row.mtime_secs.0 > existing.mtime_secs.0 {
            *existing = row.clone();
            true
        } else {
            false
        }
    } else {
        stored.sessions.push(row.clone());
        true
    }
}

fn merge_question_row(
    stored: &mut crate::gre_atlas::storage::SyncBundle,
    row: &crate::gre_atlas::storage::SyncQuestionRow,
) -> bool {
    if let Some(existing) = stored.questions.iter_mut().find(|q| q.id == row.id) {
        if row.mtime_secs.0 > existing.mtime_secs.0 {
            *existing = row.clone();
            true
        } else {
            false
        }
    } else {
        stored.questions.push(row.clone());
        true
    }
}

fn merge_attempt_row(
    stored: &mut crate::gre_atlas::storage::SyncBundle,
    row: &crate::gre_atlas::storage::SyncAttemptRow,
) -> bool {
    if row.id == 0 {
        stored.attempts.push(row.clone());
        return true;
    }
    if let Some(existing) = stored.attempts.iter_mut().find(|a| a.id == row.id) {
        if row.mtime_secs.0 > existing.mtime_secs.0 {
            *existing = row.clone();
            true
        } else {
            false
        }
    } else {
        stored.attempts.push(row.clone());
        true
    }
}

fn merge_prediction_row(
    stored: &mut crate::gre_atlas::storage::SyncBundle,
    row: &crate::gre_atlas::storage::SyncPredictionRow,
) -> bool {
    if row.id == 0 {
        stored.predictions.push(row.clone());
        return true;
    }
    if let Some(existing) = stored.predictions.iter_mut().find(|p| p.id == row.id) {
        if row.mtime_secs.0 > existing.mtime_secs.0 {
            *existing = row.clone();
            true
        } else {
            false
        }
    } else {
        stored.predictions.push(row.clone());
        true
    }
}

trait OrInternal {
    type Output;
    fn or_internal(self, context: &str) -> HttpResult<Self::Output>;
}

impl<T, E: std::fmt::Display> OrInternal for Result<T, E> {
    type Output = T;

    fn or_internal(self, context: &str) -> HttpResult<T> {
        self.map_err(|e| {
            crate::sync::error::HttpError::new_without_source(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{context}: {e}"),
            )
        })
    }
}
