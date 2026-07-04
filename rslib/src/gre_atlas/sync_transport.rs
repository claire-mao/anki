// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! HTTP transport for GRE Atlas sidecar sync via the user's configured sync
//! server (`SyncAuth`). Endpoints live under `{endpoint}/gre/sync/`.

use std::time::Duration;

use anki_proto::brainlift::BrainLiftSyncBundle;
use prost::Message;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::Url;
use tracing::debug;

use crate::error::Result;
use crate::gre_atlas::storage::SyncBundle;
use crate::sync::login::SyncAuth;
use crate::sync::request::header_and_stream::SyncHeader;
use crate::sync::request::header_and_stream::SYNC_HEADER_NAME;

const DOWNLOAD_PATH: &str = "gre/sync/download";
const UPLOAD_PATH: &str = "gre/sync/upload";

/// Shown when the configured sync server does not expose GRE Atlas sidecar routes.
pub const GRE_ATLAS_SYNC_UNAVAILABLE_MSG: &str = "GRE Atlas practice sync requires a \
    self-hosted Anki sync server with GRE sync routes enabled. AnkiWeb does not support \
    this feature. Set a custom sync server in Anki preferences (Syncing), sign in, then \
    sync again.";

/// Returns false for AnkiWeb and other hosts known not to serve `/gre/sync/*`.
pub fn endpoint_supports_gre_atlas_sync(endpoint: &Url) -> bool {
    !endpoint.as_str().contains("ankiweb")
}

fn upload_http_error(status: StatusCode) -> String {
    if status == StatusCode::NOT_FOUND {
        format!("{GRE_ATLAS_SYNC_UNAVAILABLE_MSG} (HTTP 404)")
    } else {
        format!("GRE Atlas sync upload failed: HTTP {status}")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GreAtlasDownloadRequest {
    after_usn: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GreAtlasDownloadResponse {
    bundle: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GreAtlasUploadResponse {
    current_usn: i32,
    applied_count: u32,
}

pub struct GreAtlasSyncTransport {
    sync_key: String,
    session_key: String,
    endpoint: Url,
    client: Client,
    io_timeout: Duration,
}

impl GreAtlasSyncTransport {
    pub fn new(auth: SyncAuth, client: Client) -> Self {
        let io_timeout = Duration::from_secs(auth.io_timeout_secs.unwrap_or(30) as u64);
        GreAtlasSyncTransport {
            sync_key: auth.hkey,
            session_key: simple_session_id(),
            endpoint: auth
                .endpoint
                .unwrap_or_else(|| Url::try_from("https://sync.ankiweb.net/").unwrap()),
            client,
            io_timeout,
        }
    }

    pub fn from_profile_auth(
        hkey: String,
        endpoint: Option<String>,
        client: Client,
    ) -> Result<Self> {
        let endpoint = endpoint
            .map(|e| Url::parse(&e))
            .transpose()
            .map_err(|e| crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!("invalid sync endpoint: {e}")),
            })?;
        Ok(GreAtlasSyncTransport::new(
            SyncAuth {
                hkey,
                endpoint,
                io_timeout_secs: Some(30),
            },
            client,
        ))
    }

    pub async fn download(&self, after_usn: i32) -> Result<Option<SyncBundle>> {
        let body = serde_json::to_vec(&GreAtlasDownloadRequest { after_usn }).map_err(|e| {
            crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(e.to_string()),
            }
        })?;
        let response = self.post_zstd(DOWNLOAD_PATH, body).await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(format!(
                    "GRE Atlas sync download failed: HTTP {}",
                    response.status()
                )),
            });
        }
        let payload: GreAtlasDownloadResponse = response.json().await.map_err(transport_error)?;
        if payload.bundle.is_empty() {
            return Ok(None);
        }
        let proto = BrainLiftSyncBundle::decode(payload.bundle.as_slice()).map_err(|e| {
            crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(e.to_string()),
            }
        })?;
        Ok(Some(proto_to_bundle(proto)))
    }

    pub async fn upload(&self, bundle: &SyncBundle) -> Result<u32> {
        let proto = bundle_to_proto(bundle);
        let body = proto.encode_to_vec();
        let response = self.post_zstd(UPLOAD_PATH, body).await?;

        if !response.status().is_success() {
            return Err(crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(upload_http_error(response.status())),
            });
        }
        let payload: GreAtlasUploadResponse = response.json().await.map_err(transport_error)?;
        Ok(payload.applied_count)
    }

    /// POST with the standard sync envelope: `anki-sync` header + zstd body.
    async fn post_zstd(&self, path: &str, body: Vec<u8>) -> Result<reqwest::Response> {
        let url = self.endpoint.join(path).map_err(|e| {
            crate::error::AnkiError::InvalidInput {
                source: snafu::FromString::without_source(e.to_string()),
            }
        })?;
        let header = SyncHeader {
            sync_version: crate::sync::version::SyncVersion::latest(),
            sync_key: self.sync_key.clone(),
            client_ver: crate::version::sync_client_version().into(),
            session_key: self.session_key.clone(),
        };
        let compressed = compress_sync_body(body)?;
        println!(
            "GRE sync compressed: {} bytes, first 8 = {:02x?}",
            compressed.data.len(),
            &compressed.data[..compressed.data.len().min(8)]
        );
        debug!(
            url = %url,
            uncompressed_bytes = compressed.uncompressed_len,
            compressed_bytes = compressed.data.len(),
            "gre atlas sync request"
        );
        self.client
            .post(url)
            .header(&SYNC_HEADER_NAME, serde_json::to_string(&header).unwrap())
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(compressed.data)
            .timeout(self.io_timeout)
            .send()
            .await
            .map_err(transport_error)
    }
}

struct CompressedSyncBody {
    data: Vec<u8>,
    uncompressed_len: usize,
}

fn compress_sync_body(body: Vec<u8>) -> Result<CompressedSyncBody> {
    let uncompressed_len = body.len();
    let data = zstd::encode_all(body.as_slice(), 0).map_err(|e| {
        crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(format!("zstd encode: {e}")),
        }
    })?;
    Ok(CompressedSyncBody {
        data,
        uncompressed_len,
    })
}

fn transport_error(err: reqwest::Error) -> crate::error::AnkiError {
    crate::error::AnkiError::InvalidInput {
        source: snafu::FromString::without_source(format!("GRE Atlas sync transport: {err}")),
    }
}

fn simple_session_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:016x}", rng.gen::<u64>())
}

pub fn bundle_to_proto(bundle: &SyncBundle) -> BrainLiftSyncBundle {
    BrainLiftSyncBundle {
        sessions: bundle
            .sessions
            .iter()
            .map(|s| anki_proto::brainlift::BrainLiftSyncSession {
                id: s.id.clone(),
                started_at_secs: s.started_at_secs.0,
                ended_at_secs: s.ended_at_secs.map(|t| t.0),
                source: s.source.clone(),
                usn: s.usn,
                mtime_secs: s.mtime_secs.0,
            })
            .collect(),
        questions: bundle
            .questions
            .iter()
            .map(|q| {
                let choices: Vec<String> =
                    serde_json::from_str(&q.choices_json).unwrap_or_default();
                anki_proto::brainlift::BrainLiftSyncQuestion {
                    id: q.id.clone(),
                    topic: q.topic.clone(),
                    section: q.section.clone(),
                    format: q.format.clone(),
                    stem: q.stem.clone(),
                    choices,
                    correct_answer: q.correct_answer.clone(),
                    explanation: q.explanation.clone(),
                    difficulty: q.difficulty,
                    source_name: q.source_name.clone(),
                    source_section: q.source_section.clone(),
                    generated_at_secs: q.generated_at_secs,
                    generation_confidence: q.generation_confidence,
                    source_document: q.source_document.clone(),
                    model_version: q.model_version.clone(),
                    provenance: q.provenance.clone(),
                    evaluation_status: q.evaluation_status.clone(),
                    usn: q.usn,
                    mtime_secs: q.mtime_secs.0,
                }
            })
            .collect(),
        attempts: bundle
            .attempts
            .iter()
            .map(|a| anki_proto::brainlift::BrainLiftSyncAttempt {
                id: a.id,
                question_id: a.question_id.clone(),
                topic: a.topic.clone(),
                difficulty: a.difficulty,
                answered_at_secs: a.answered_at_secs.0,
                answer: a.answer.clone(),
                correct: a.correct,
                response_time_ms: a.response_time_ms,
                confidence: a.confidence,
                session_id: a.session_id.clone(),
                usn: a.usn,
                mtime_secs: a.mtime_secs.0,
            })
            .collect(),
        predictions: bundle
            .predictions
            .iter()
            .map(|p| anki_proto::brainlift::BrainLiftSyncPrediction {
                id: p.id,
                predicted_at_secs: p.predicted_at_secs.0,
                projected_score: p.projected_score,
                projected_score_low: p.projected_score_low,
                projected_score_high: p.projected_score_high,
                memory_score: p.memory_score,
                performance_score: p.performance_score,
                coverage_ratio: p.coverage_ratio,
                confidence_level: p.confidence_level.clone(),
                model_version: p.model_version.clone(),
                outcome_score: p.outcome_score,
                outcome_observed_at_secs: p.outcome_observed_at_secs.map(|t| t.0),
                outcome_memory_score: p.outcome_memory_score,
                outcome_performance_score: p.outcome_performance_score,
                practice_correct: p.practice_correct,
                practice_total: p.practice_total,
                usn: p.usn,
                mtime_secs: p.mtime_secs.0,
            })
            .collect(),
        current_usn: bundle.current_usn,
        last_modified_secs: bundle.last_modified_secs.0,
    }
}

pub fn proto_to_bundle(proto: BrainLiftSyncBundle) -> SyncBundle {
    SyncBundle {
        sessions: proto
            .sessions
            .into_iter()
            .map(|s| crate::gre_atlas::storage::SyncSessionRow {
                id: s.id,
                started_at_secs: crate::timestamp::TimestampSecs(s.started_at_secs),
                ended_at_secs: s.ended_at_secs.map(crate::timestamp::TimestampSecs),
                source: s.source,
                usn: s.usn,
                mtime_secs: crate::timestamp::TimestampSecs(s.mtime_secs),
            })
            .collect(),
        questions: proto
            .questions
            .into_iter()
            .map(|q| crate::gre_atlas::storage::SyncQuestionRow {
                id: q.id,
                topic: q.topic,
                section: q.section,
                format: q.format,
                stem: q.stem,
                choices_json: serde_json::to_string(&q.choices).unwrap_or_else(|_| "[]".into()),
                correct_answer: q.correct_answer,
                explanation: q.explanation,
                difficulty: q.difficulty,
                source_name: q.source_name,
                source_section: q.source_section,
                generated_at_secs: q.generated_at_secs,
                generation_confidence: q.generation_confidence,
                source_document: q.source_document,
                model_version: q.model_version,
                provenance: q.provenance,
                evaluation_status: q.evaluation_status,
                usn: q.usn,
                mtime_secs: crate::timestamp::TimestampSecs(q.mtime_secs),
            })
            .collect(),
        attempts: proto
            .attempts
            .into_iter()
            .map(|a| crate::gre_atlas::storage::SyncAttemptRow {
                id: a.id,
                question_id: a.question_id,
                topic: a.topic,
                difficulty: a.difficulty,
                answered_at_secs: crate::timestamp::TimestampSecs(a.answered_at_secs),
                answer: a.answer,
                correct: a.correct,
                response_time_ms: a.response_time_ms,
                confidence: a.confidence,
                session_id: a.session_id.filter(|s| !s.is_empty()),
                usn: a.usn,
                mtime_secs: crate::timestamp::TimestampSecs(a.mtime_secs),
            })
            .collect(),
        predictions: proto
            .predictions
            .into_iter()
            .map(|p| crate::gre_atlas::storage::SyncPredictionRow {
                id: p.id,
                predicted_at_secs: crate::timestamp::TimestampSecs(p.predicted_at_secs),
                projected_score: p.projected_score,
                projected_score_low: p.projected_score_low,
                projected_score_high: p.projected_score_high,
                memory_score: p.memory_score,
                performance_score: p.performance_score,
                coverage_ratio: p.coverage_ratio,
                confidence_level: p.confidence_level,
                model_version: p.model_version,
                outcome_score: p.outcome_score,
                outcome_observed_at_secs: p
                    .outcome_observed_at_secs
                    .map(crate::timestamp::TimestampSecs),
                outcome_memory_score: p.outcome_memory_score,
                outcome_performance_score: p.outcome_performance_score,
                practice_correct: p.practice_correct,
                practice_total: p.practice_total,
                usn: p.usn,
                mtime_secs: crate::timestamp::TimestampSecs(p.mtime_secs),
            })
            .collect(),
        current_usn: proto.current_usn,
        last_modified_secs: crate::timestamp::TimestampSecs(proto.last_modified_secs),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wiremock::matchers::header_exists;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;

    #[test]
    fn ankiweb_endpoint_does_not_support_gre_atlas_sync() {
        let url = Url::parse("https://sync.ankiweb.net/").unwrap();
        assert!(!endpoint_supports_gre_atlas_sync(&url));
    }

    #[test]
    fn custom_endpoint_supports_gre_atlas_sync() {
        let url = Url::parse("https://sync.example.com/").unwrap();
        assert!(endpoint_supports_gre_atlas_sync(&url));
    }

    #[test]
    fn upload_404_error_includes_self_hosted_hint() {
        let msg = upload_http_error(StatusCode::NOT_FOUND);
        assert!(msg.contains("self-hosted"));
        assert!(msg.contains("404"));
    }

    #[test]
    fn download_request_body_is_zstd_json() {
        let json = serde_json::to_vec(&GreAtlasDownloadRequest { after_usn: 0 }).unwrap();
        let compressed = compress_sync_body(json.clone()).unwrap();
        assert_ne!(compressed.data, json);
        let decoded = zstd::decode_all(compressed.data.as_slice()).unwrap();
        assert_eq!(decoded, br#"{"after_usn":0}"#);
    }

    #[tokio::test]
    async fn download_posts_zstd_body_with_sync_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gre/sync/download"))
            .and(header_exists("anki-sync"))
            .and(|req: &wiremock::Request| {
                req.headers
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    == Some("application/octet-stream")
            })
            .and(|req: &wiremock::Request| {
                zstd::decode_all(req.body.as_slice())
                    .map(|body| body == br#"{"after_usn":0}"#)
                    .unwrap_or(false)
            })
            .respond_with(ResponseTemplate::new(200).set_body_json(GreAtlasDownloadResponse {
                bundle: vec![],
            }))
            .mount(&mock_server)
            .await;

        let transport = GreAtlasSyncTransport::new(
            SyncAuth {
                hkey: "96fb1623e2b9a953fcc5cd9e8da27b6916ae0a09".into(),
                endpoint: Some(Url::parse(&format!("{}/", mock_server.uri())).unwrap()),
                io_timeout_secs: Some(30),
            },
            Client::new(),
        );
        let result = transport.download(0).await.unwrap();
        assert!(result.is_none());
    }
}
