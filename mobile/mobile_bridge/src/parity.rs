// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::ffi::CString;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use anki::backend::Backend;
use anki::prelude::I18n;
use anki_proto::backend::BackendInit;
use anki_proto::brainlift::CreateSessionRequest;
use anki_proto::brainlift::CreateSessionResponse;
use anki_proto::brainlift::DashboardState;
use anki_proto::brainlift::GetDashboardRequest;
use anki_proto::brainlift::GetScoresResponse;
use anki_proto::brainlift::GetStudyPlanRequest;
use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::brainlift::ListQuestionsRequest;
use anki_proto::brainlift::ListQuestionsResponse;
use anki_proto::brainlift::ReadinessCalibrationResponse;
use anki_proto::brainlift::StudyPlanResponse;
use anki_proto::generic::Empty;
use anki_proto::stats::TopicMasteryRequest;
use anki_proto::stats::TopicMasteryResponse;
use prost::Message;

use crate::backend_method::backend_method;
use crate::backend_method::invoke;
use crate::demo_pages;
use crate::gre_pages;
use crate::study_pages;
use crate::sync_pages;
use crate::AnkiMobileBackend;
use crate::ANKI_MOBILE_BACKEND_ERROR;
use crate::ANKI_MOBILE_OK;

static PARITY_COUNTER: AtomicU64 = AtomicU64::new(0);

struct ParityHarness {
    mobile_backend: *mut AnkiMobileBackend,
    direct: Backend,
    mobile_path: PathBuf,
    direct_path: PathBuf,
}

impl ParityHarness {
    unsafe fn new(label: &str) -> Self {
        let id = PARITY_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "anki-mobile-parity-{label}-{}-{id}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let mobile_path = dir.join("mobile.anki2");
        let direct_path = dir.join("direct.anki2");
        let _ = std::fs::remove_file(&mobile_path);
        let _ = std::fs::remove_file(&direct_path);

        let init = BackendInit {
            preferred_langs: vec!["en".into()],
            server: false,
            locale_folder_path: String::new(),
        };
        let mut handle: *mut AnkiMobileBackend = std::ptr::null_mut();
        let code = crate::anki_mobile_backend_create(
            init.encode_to_vec().as_ptr(),
            init.encode_to_vec().len(),
            &mut handle,
        );
        assert_eq!(code, ANKI_MOBILE_OK);

        let harness = Self {
            mobile_backend: handle,
            direct: Backend::new(I18n::template_only(), false),
            mobile_path,
            direct_path,
        };
        harness.open_both_collections();
        harness
    }

    fn open_both_collections(&self) {
        unsafe {
            self.open_collection_on_mobile();
            self.open_collection_on_direct();
        }
    }

    unsafe fn mobile_backend(&self) -> &AnkiMobileBackend {
        &*self.mobile_backend
    }

    unsafe fn open_collection_on_mobile(&self) {
        let collection = CString::new(self.mobile_path.to_string_lossy().as_bytes()).unwrap();
        let media = CString::new(
            self.mobile_path
                .with_extension("media")
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();
        let media_db = CString::new(
            self.mobile_path
                .with_extension("mdb")
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();
        let mut out_bytes = std::ptr::null_mut();
        let mut out_len = 0;
        let code = crate::anki_mobile_open_collection(
            self.mobile_backend,
            collection.as_ptr(),
            media.as_ptr(),
            media_db.as_ptr(),
            &mut out_bytes,
            &mut out_len,
        );
        crate::anki_mobile_bytes_free(out_bytes, out_len);
        assert_eq!(code, ANKI_MOBILE_OK);
    }

    fn open_collection_on_direct(&self) {
        gre_pages::open_collection(
            &self.direct,
            &self.direct_path.to_string_lossy(),
            &self.direct_path.with_extension("media").to_string_lossy(),
            &self.direct_path.with_extension("mdb").to_string_lossy(),
        )
        .expect("direct open");
    }

    unsafe fn mobile_command(
        &self,
        service: &str,
        method: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, Vec<u8>> {
        let (service_idx, method_idx) = backend_method(service, method).unwrap_or_else(|err| {
            panic!(
                "missing {service}.{method}: {}",
                String::from_utf8_lossy(&err)
            )
        });
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len = 0usize;
        let code = crate::anki_mobile_backend_command(
            self.mobile_backend,
            service_idx,
            method_idx,
            input.as_ptr(),
            input.len(),
            &mut out_ptr,
            &mut out_len,
        );
        let out = Vec::from_raw_parts(out_ptr, out_len, out_len);
        match code {
            ANKI_MOBILE_OK => Ok(out),
            ANKI_MOBILE_BACKEND_ERROR => Err(out),
            other => panic!("unexpected status code {other}"),
        }
    }

    fn assert_proto_equal<M: Message + Default + PartialEq>(
        &self,
        service: &str,
        method: &str,
        input: &[u8],
        normalize: impl Fn(M) -> M,
    ) {
        unsafe {
            let mobile_out = self
                .mobile_command(service, method, input)
                .unwrap_or_else(|err| {
                    panic!(
                        "mobile {service}.{method} failed: {:?}",
                        String::from_utf8_lossy(&err)
                    )
                });
            let direct_out = invoke(&self.direct, service, method, input).unwrap_or_else(|err| {
                panic!(
                    "direct {service}.{method} failed: {:?}",
                    String::from_utf8_lossy(&err)
                )
            });
            let mobile_msg = M::decode(mobile_out.as_slice()).expect("decode mobile");
            let direct_msg = M::decode(direct_out.as_slice()).expect("decode direct");
            assert_eq!(normalize(mobile_msg), normalize(direct_msg));
        }
    }
}

impl Drop for ParityHarness {
    fn drop(&mut self) {
        unsafe {
            crate::anki_mobile_backend_destroy(self.mobile_backend);
            if let Some(dir) = self.mobile_path.parent() {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }
}

fn normalize_scores(mut response: GetScoresResponse) -> GetScoresResponse {
    if let Some(readiness) = response.readiness.as_mut() {
        readiness.last_updated_millis = 0;
    }
    response
}

fn normalize_dashboard(mut response: DashboardState) -> DashboardState {
    response.computed_at_millis = 0;
    if let Some(readiness) = response.readiness.as_mut() {
        readiness.last_updated_millis = 0;
    }
    response
}

fn normalize_study_plan(mut response: StudyPlanResponse) -> StudyPlanResponse {
    response.computed_at_millis = 0;
    response
}

fn normalize_calibration(
    mut response: ReadinessCalibrationResponse,
) -> ReadinessCalibrationResponse {
    response.computed_at_millis = 0;
    if let Some(readiness) = response.readiness.as_mut() {
        readiness.last_updated_millis = 0;
    }
    response
}

fn normalize_mastery(mut response: TopicMasteryResponse) -> TopicMasteryResponse {
    response.computed_at_millis = 0;
    response
}

fn normalize_session(mut response: CreateSessionResponse) -> CreateSessionResponse {
    response.session_id.clear();
    response.started_at_secs = 0;
    response
}

#[test]
fn mobile_bridge_matches_backend_get_scores() {
    let harness = unsafe { ParityHarness::new("scores") };
    harness.assert_proto_equal::<GetScoresResponse>(
        "BackendBrainLiftService",
        "get_scores",
        &Empty::default().encode_to_vec(),
        normalize_scores,
    );
}

#[test]
fn mobile_bridge_get_dashboard_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("dashboard") };
    let req = GetDashboardRequest {
        recent_activity_limit: 5,
        topic_insight_limit: 3,
    }
    .encode_to_vec();
    harness.assert_proto_equal::<DashboardState>(
        "BackendBrainLiftService",
        "get_dashboard",
        &req,
        normalize_dashboard,
    );
}

#[test]
fn mobile_bridge_get_study_plan_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("study-plan") };
    let req = GetStudyPlanRequest { limit: 3 }.encode_to_vec();
    harness.assert_proto_equal::<StudyPlanResponse>(
        "BackendBrainLiftService",
        "get_study_plan",
        &req,
        normalize_study_plan,
    );
}

#[test]
fn mobile_bridge_get_gre_study_status_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("study-status") };
    harness.assert_proto_equal::<GreStudyStatusResponse>(
        "BackendBrainLiftService",
        "get_gre_study_status",
        &Empty::default().encode_to_vec(),
        |response| response,
    );
}

#[test]
fn mobile_bridge_get_readiness_calibration_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("calibration") };
    harness.assert_proto_equal::<ReadinessCalibrationResponse>(
        "BackendBrainLiftService",
        "get_readiness_calibration",
        &Empty::default().encode_to_vec(),
        normalize_calibration,
    );
}

#[test]
fn mobile_bridge_topic_mastery_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("mastery") };
    let req = TopicMasteryRequest {
        search: r#"deck:"GRE Atlas" OR deck:"BrainLift GRE""#.into(),
        topic_tag_prefix: "gre::".into(),
        mastery_threshold: None,
        min_reviews: 0,
    }
    .encode_to_vec();
    harness.assert_proto_equal::<TopicMasteryResponse>(
        "BackendStatsService",
        "topic_mastery",
        &req,
        normalize_mastery,
    );
}

#[test]
fn mobile_bridge_list_questions_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("questions") };
    let req = ListQuestionsRequest {
        limit: 200,
        topic_prefix: String::new(),
    }
    .encode_to_vec();
    harness.assert_proto_equal::<ListQuestionsResponse>(
        "BackendBrainLiftService",
        "list_questions",
        &req,
        |response| response,
    );
}

#[test]
fn mobile_bridge_create_session_matches_direct_backend() {
    let harness = unsafe { ParityHarness::new("session") };
    let req = CreateSessionRequest {
        source: "practice".into(),
    }
    .encode_to_vec();
    harness.assert_proto_equal::<CreateSessionResponse>(
        "BackendBrainLiftService",
        "create_session",
        &req,
        normalize_session,
    );
}

#[test]
fn gre_dashboard_page_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("dashboard-page") };
    let mobile_view = gre_pages::load_dashboard_page(unsafe { harness.mobile_backend().backend() })
        .expect("mobile dashboard page");
    let direct_view =
        gre_pages::load_dashboard_page(&harness.direct).expect("direct dashboard page");
    assert_eq!(
        gre_pages::normalize_dashboard(mobile_view),
        gre_pages::normalize_dashboard(direct_view)
    );
}

#[test]
fn gre_progress_page_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("progress-page") };
    let mobile_view = gre_pages::load_progress_page(unsafe { harness.mobile_backend().backend() })
        .expect("mobile progress page");
    let direct_view = gre_pages::load_progress_page(&harness.direct).expect("direct progress page");
    assert_eq!(
        gre_pages::normalize_progress(mobile_view),
        gre_pages::normalize_progress(direct_view)
    );
}

#[test]
fn gre_explain_answer_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("explain-answer") };
    let bootstrap =
        gre_pages::load_practice_bootstrap(&harness.direct).expect("practice bootstrap");
    let question = bootstrap
        .questions
        .first()
        .expect("expected at least one practice question");
    let selected = question
        .choices
        .first()
        .cloned()
        .unwrap_or_else(|| "A".into());
    let input = gre_pages::GreExplainAnswerInput {
        question_id: question.id.clone(),
        selected_answer: selected,
    };
    let mobile_view = gre_pages::explain_practice_answer(
        unsafe { harness.mobile_backend().backend() },
        input.clone(),
    )
    .expect("mobile explain answer");
    let direct_view =
        gre_pages::explain_practice_answer(&harness.direct, input).expect("direct explain answer");
    assert_eq!(mobile_view, direct_view);
    assert!(!mobile_view.summary.is_empty());
    assert_eq!(
        mobile_view.choices.iter().filter(|c| c.is_correct).count(),
        1
    );
    assert!(!mobile_view.citation_source_name.is_empty());
}

#[test]
fn gre_practice_bootstrap_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("practice-page") };
    let mut mobile_view =
        gre_pages::load_practice_bootstrap(unsafe { harness.mobile_backend().backend() })
            .expect("mobile practice bootstrap");
    let mut direct_view =
        gre_pages::load_practice_bootstrap(&harness.direct).expect("direct practice bootstrap");
    mobile_view = gre_pages::normalize_practice_bootstrap(mobile_view);
    direct_view = gre_pages::normalize_practice_bootstrap(direct_view);
    assert_eq!(mobile_view, direct_view);
}

#[test]
fn gre_study_page_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("study-page") };
    let mobile_view = gre_pages::load_study_page(unsafe { harness.mobile_backend().backend() })
        .expect("mobile study page");
    let direct_view = gre_pages::load_study_page(&harness.direct).expect("direct study page");
    assert_eq!(mobile_view, direct_view);
}

#[test]
fn gre_study_review_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("study-review") };
    let mobile_view = study_pages::load_study_review(unsafe { harness.mobile_backend().backend() })
        .expect("mobile study review");
    let direct_view = study_pages::load_study_review(&harness.direct).expect("direct study review");
    assert_eq!(mobile_view, direct_view);
}

#[test]
fn gre_atlas_sync_status_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("sync-status") };
    let mobile_view = sync_pages::load_sync_status(unsafe { harness.mobile_backend().backend() })
        .expect("mobile sync status");
    let direct_view = sync_pages::load_sync_status(&harness.direct).expect("direct sync status");
    assert_eq!(mobile_view, direct_view);
}

#[test]
fn gre_atlas_sync_pull_matches_after_practice_attempt() {
    let harness = unsafe { ParityHarness::new("sync-pull") };
    for backend in [&harness.direct, unsafe {
        harness.mobile_backend().backend()
    }] {
        let bootstrap = gre_pages::load_practice_bootstrap(backend).expect("practice bootstrap");
        let question = bootstrap
            .questions
            .first()
            .expect("expected at least one practice question");
        gre_pages::record_practice_attempt(
            backend,
            gre_pages::GreRecordAttemptInput {
                question_id: question.id.clone(),
                answer: question
                    .choices
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "A".into()),
                response_time_ms: 800,
                session_id: bootstrap.session_id.clone(),
            },
        )
        .expect("record attempt");
    }

    let input = sync_pages::GreAtlasSyncPullInput {
        after_usn: 0,
        limit: 100,
    };
    let mobile_view =
        sync_pages::pull_sync_changes(unsafe { harness.mobile_backend().backend() }, input.clone())
            .expect("mobile sync pull")
            .normalize_for_parity();
    let direct_view = sync_pages::pull_sync_changes(&harness.direct, input)
        .expect("direct sync pull")
        .normalize_for_parity();
    assert_eq!(mobile_view, direct_view);
    assert!(!mobile_view.attempts.is_empty());
}

#[test]
fn prepare_demo_collection_matches_between_mobile_ffi_and_direct_backend() {
    let harness = unsafe { ParityHarness::new("prepare-demo") };
    let mobile_view =
        demo_pages::prepare_demo_collection(unsafe { harness.mobile_backend().backend() })
            .expect("mobile prepare demo");
    let direct_view =
        demo_pages::prepare_demo_collection(&harness.direct).expect("direct prepare demo");
    assert_eq!(mobile_view.deck_name, direct_view.deck_name);
    assert_eq!(mobile_view.cards_added, direct_view.cards_added);
    assert_eq!(mobile_view.cards_added, 8);
    assert_eq!(mobile_view.practice_attempts_added, 4);
    // greatlas.db is shared per collection folder in the parity harness, so the
    // second prepare call may skip practice seeding.
    assert!(direct_view.practice_attempts_added <= mobile_view.practice_attempts_added);
    assert!(mobile_view.due_total >= 4);
    assert_eq!(mobile_view.due_total, direct_view.due_total);
}
