// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
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
use anki_proto::collection::OpenCollectionRequest;
use anki_proto::generic::Empty;
use anki_proto::stats::TopicMasteryRequest;
use anki_proto::stats::TopicMasteryResponse;
use prost::Message;
use serde::Serialize;

use crate::backend_method::invoke_proto;

const GRE_DECK_NAME: &str = "BrainLift GRE";
const TOPIC_TAG_PREFIX: &str = "gre::";

const BRAINLIFT_SERVICE: &str = "BackendBrainLiftService";
const STATS_SERVICE: &str = "BackendStatsService";
const COLLECTION_SERVICE: &str = "BackendCollectionService";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreDashboardView {
    pub readiness_projected: Option<f32>,
    pub readiness_low: Option<f32>,
    pub readiness_high: Option<f32>,
    pub readiness_sufficient: bool,
    pub readiness_summary: String,
    pub estimated_gre_combined: Option<u32>,
    pub estimated_gre_low: Option<u32>,
    pub estimated_gre_high: Option<u32>,
    pub estimated_gre_preliminary: bool,
    pub daily_plan_headline: String,
    pub daily_plan_task_count: u32,
    pub study_plan_summary: String,
    pub weak_topic_name: Option<String>,
    pub deck_exists: bool,
    pub deck_name: String,
    pub due_new: u32,
    pub due_learn: u32,
    pub due_review: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreProgressView {
    pub memory_value: Option<f32>,
    pub memory_low: Option<f32>,
    pub memory_high: Option<f32>,
    pub memory_sufficient: bool,
    pub performance_value: Option<f32>,
    pub performance_low: Option<f32>,
    pub performance_high: Option<f32>,
    pub performance_sufficient: bool,
    pub readiness_projected: Option<f32>,
    pub readiness_low: Option<f32>,
    pub readiness_high: Option<f32>,
    pub readiness_sufficient: bool,
    pub estimated_gre_combined: Option<u32>,
    pub estimated_gre_low: Option<u32>,
    pub estimated_gre_high: Option<u32>,
    pub weighted_coverage: f32,
    pub studied_topics: u32,
    pub calibration_assessment: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrePracticeBootstrapView {
    pub session_id: String,
    pub question_count: u32,
    pub memory_value: Option<f32>,
    pub performance_value: Option<f32>,
    pub performance_sufficient: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreStudyView {
    pub deck_exists: bool,
    pub deck_name: String,
    pub due_new: u32,
    pub due_learn: u32,
    pub due_review: u32,
    pub due_total: u32,
}

pub fn open_collection(
    backend: &Backend,
    collection_path: &str,
    media_folder_path: &str,
    media_db_path: &str,
) -> Result<(), Vec<u8>> {
    let request = OpenCollectionRequest {
        collection_path: collection_path.into(),
        media_folder_path: media_folder_path.into(),
        media_db_path: media_db_path.into(),
    };
    crate::backend_method::invoke(
        backend,
        COLLECTION_SERVICE,
        "open_collection",
        &request.encode_to_vec(),
    )
    .map(|_| ())
}

pub fn load_dashboard_page(backend: &Backend) -> Result<GreDashboardView, Vec<u8>> {
    let dashboard = fetch_dashboard(backend, 5, 1)?;
    let plan = fetch_study_plan(backend, 3)?;
    let status = fetch_gre_study_status(backend)?;
    Ok(GreDashboardView::from_responses(dashboard, plan, status))
}

pub fn load_progress_page(backend: &Backend) -> Result<GreProgressView, Vec<u8>> {
    let scores = fetch_scores(backend)?;
    let dashboard = fetch_dashboard(backend, 5, 8)?;
    let mastery = fetch_topic_mastery(backend)?;
    let calibration = fetch_readiness_calibration(backend)?;
    Ok(GreProgressView::from_responses(
        scores,
        dashboard,
        mastery,
        calibration,
    ))
}

pub fn load_practice_bootstrap(backend: &Backend) -> Result<GrePracticeBootstrapView, Vec<u8>> {
    let session = invoke_proto::<CreateSessionResponse>(
        backend,
        BRAINLIFT_SERVICE,
        "create_session",
        &CreateSessionRequest {
            source: "practice".into(),
        }
        .encode_to_vec(),
    )?;
    let questions = invoke_proto::<ListQuestionsResponse>(
        backend,
        BRAINLIFT_SERVICE,
        "list_questions",
        &ListQuestionsRequest {
            limit: 200,
            topic_prefix: String::new(),
        }
        .encode_to_vec(),
    )?;
    let scores = fetch_scores(backend)?;
    Ok(GrePracticeBootstrapView::from_responses(session, questions, scores))
}

pub fn load_study_page(backend: &Backend) -> Result<GreStudyView, Vec<u8>> {
    let status = fetch_gre_study_status(backend)?;
    Ok(GreStudyView::from_status(status))
}

fn fetch_scores(backend: &Backend) -> Result<GetScoresResponse, Vec<u8>> {
    invoke_proto(
        backend,
        BRAINLIFT_SERVICE,
        "get_scores",
        &Empty::default().encode_to_vec(),
    )
}

fn fetch_dashboard(
    backend: &Backend,
    recent_activity_limit: u32,
    topic_insight_limit: u32,
) -> Result<DashboardState, Vec<u8>> {
    invoke_proto(
        backend,
        BRAINLIFT_SERVICE,
        "get_dashboard",
        &GetDashboardRequest {
            recent_activity_limit,
            topic_insight_limit,
        }
        .encode_to_vec(),
    )
}

fn fetch_study_plan(backend: &Backend, limit: u32) -> Result<StudyPlanResponse, Vec<u8>> {
    invoke_proto(
        backend,
        BRAINLIFT_SERVICE,
        "get_study_plan",
        &GetStudyPlanRequest { limit }.encode_to_vec(),
    )
}

fn fetch_gre_study_status(backend: &Backend) -> Result<GreStudyStatusResponse, Vec<u8>> {
    invoke_proto(
        backend,
        BRAINLIFT_SERVICE,
        "get_gre_study_status",
        &Empty::default().encode_to_vec(),
    )
}

fn fetch_readiness_calibration(
    backend: &Backend,
) -> Result<ReadinessCalibrationResponse, Vec<u8>> {
    invoke_proto(
        backend,
        BRAINLIFT_SERVICE,
        "get_readiness_calibration",
        &Empty::default().encode_to_vec(),
    )
}

fn fetch_topic_mastery(backend: &Backend) -> Result<TopicMasteryResponse, Vec<u8>> {
    invoke_proto(
        backend,
        STATS_SERVICE,
        "topic_mastery",
        &TopicMasteryRequest {
            search: format!("deck:\"{GRE_DECK_NAME}\""),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            mastery_threshold: None,
            min_reviews: 0,
        }
        .encode_to_vec(),
    )
}

impl GreDashboardView {
    fn from_responses(
        dashboard: DashboardState,
        plan: StudyPlanResponse,
        status: GreStudyStatusResponse,
    ) -> Self {
        let readiness = dashboard.readiness.unwrap_or_default();
        let estimated = dashboard.estimated_gre.unwrap_or_default();
        let daily = plan.daily_plan.unwrap_or_default();
        let weak_topic_name = dashboard
            .weak_topics
            .first()
            .map(|topic| topic.display_name.clone());
        Self {
            readiness_projected: readiness.projected_score,
            readiness_low: readiness.projected_score_low,
            readiness_high: readiness.projected_score_high,
            readiness_sufficient: readiness.sufficient_data,
            readiness_summary: if readiness.sufficient_data {
                readiness.evidence_summary
            } else {
                readiness.abstain_reason
            },
            estimated_gre_combined: estimated.combined_score,
            estimated_gre_low: estimated.combined_score_low,
            estimated_gre_high: estimated.combined_score_high,
            estimated_gre_preliminary: estimated.preliminary,
            daily_plan_headline: daily.headline,
            daily_plan_task_count: daily.tasks.len() as u32,
            study_plan_summary: plan.summary,
            weak_topic_name,
            deck_exists: status.deck_exists,
            deck_name: status.deck_name,
            due_new: status.new_count,
            due_learn: status.learn_count,
            due_review: status.review_count,
        }
    }
}

impl GreProgressView {
    fn from_responses(
        scores: GetScoresResponse,
        dashboard: DashboardState,
        mastery: TopicMasteryResponse,
        calibration: ReadinessCalibrationResponse,
    ) -> Self {
        let memory = scores.memory.unwrap_or_default();
        let performance = scores.performance.unwrap_or_default();
        let readiness = scores.readiness.unwrap_or_default();
        let estimated = scores.estimated_gre.unwrap_or_default();
        let coverage = dashboard.coverage.unwrap_or_default();
        let mastery_summary = mastery.summary.unwrap_or_default();
        let calibration_stats = calibration.calibration.unwrap_or_default();
        Self {
            memory_value: memory.value,
            memory_low: memory.value_low,
            memory_high: memory.value_high,
            memory_sufficient: memory.sufficient_data,
            performance_value: performance.value,
            performance_low: performance.value_low,
            performance_high: performance.value_high,
            performance_sufficient: performance.sufficient_data,
            readiness_projected: readiness.projected_score,
            readiness_low: readiness.projected_score_low,
            readiness_high: readiness.projected_score_high,
            readiness_sufficient: readiness.sufficient_data,
            estimated_gre_combined: estimated.combined_score,
            estimated_gre_low: estimated.combined_score_low,
            estimated_gre_high: estimated.combined_score_high,
            weighted_coverage: coverage.weighted_ratio,
            studied_topics: mastery_summary.studied_cards,
            calibration_assessment: calibration_stats.assessment,
        }
    }
}

impl GrePracticeBootstrapView {
    fn from_responses(
        session: CreateSessionResponse,
        questions: ListQuestionsResponse,
        scores: GetScoresResponse,
    ) -> Self {
        let memory = scores.memory.unwrap_or_default();
        let performance = scores.performance.unwrap_or_default();
        Self {
            session_id: session.session_id,
            question_count: questions.questions.len() as u32,
            memory_value: memory.value,
            performance_value: performance.value,
            performance_sufficient: performance.sufficient_data,
        }
    }
}

impl GreStudyView {
    fn from_status(status: GreStudyStatusResponse) -> Self {
        let due_total = status.new_count + status.learn_count + status.review_count;
        Self {
            deck_exists: status.deck_exists,
            deck_name: status.deck_name,
            due_new: status.new_count,
            due_learn: status.learn_count,
            due_review: status.review_count,
            due_total,
        }
    }
}

pub fn normalize_practice_bootstrap(mut view: GrePracticeBootstrapView) -> GrePracticeBootstrapView {
    view.session_id.clear();
    view
}
