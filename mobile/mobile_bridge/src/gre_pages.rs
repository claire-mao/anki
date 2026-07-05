// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use anki_proto::brainlift::AbstentionRequirement;
use anki_proto::brainlift::AnswerExplanation;
use anki_proto::brainlift::CreateSessionRequest;
use anki_proto::brainlift::CreateSessionResponse;
use anki_proto::brainlift::DashboardCoverage;
use anki_proto::brainlift::DashboardState;
use anki_proto::brainlift::DashboardTopicInsight;
use anki_proto::brainlift::ExplainAnswerRequest;
use anki_proto::brainlift::ExplainAnswerResponse;
use anki_proto::brainlift::GetDashboardRequest;
use anki_proto::brainlift::GetScoresResponse;
use anki_proto::brainlift::GetStudyPlanRequest;
use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::brainlift::ListQuestionsRequest;
use anki_proto::brainlift::ListQuestionsResponse;
use anki_proto::brainlift::PerformanceAttempt;
use anki_proto::brainlift::Question;
use anki_proto::brainlift::ReadinessCalibrationResponse;
use anki_proto::brainlift::RecordAttemptRequest;
use anki_proto::brainlift::RecordAttemptResponse;
use anki_proto::brainlift::StudyPlanDailyTask;
use anki_proto::brainlift::StudyPlanResponse;
use anki_proto::collection::OpenCollectionRequest;
use anki_proto::generic::Empty;
use anki_proto::stats::TopicMasteryEntry;
use anki_proto::stats::TopicMasteryRequest;
use anki_proto::stats::TopicMasteryResponse;
use prost::Message;
use rand::seq::SliceRandom;
use serde::Deserialize;
use serde::Serialize;

use crate::backend_method::invoke_proto;

const GRE_DECK_NAME: &str = "GRE Atlas";
const LEGACY_GRE_DECK_NAME: &str = "BrainLift GRE";
const TOPIC_TAG_PREFIX: &str = "gre::";

const GRE_ATLAS_SERVICE: &str = "BackendBrainLiftService";
const STATS_SERVICE: &str = "BackendStatsService";
const COLLECTION_SERVICE: &str = "BackendCollectionService";

const HOME_RECENT_ACTIVITY_LIMIT: u32 = 5;
const HOME_TOPIC_INSIGHT_LIMIT: u32 = 3;
const HOME_STUDY_PLAN_LIMIT: u32 = 3;
const PROGRESS_TOPIC_INSIGHT_LIMIT: u32 = 8;
const PRACTICE_QUESTION_LIMIT: u32 = 200;
const PRACTICE_TREND_WINDOW: usize = 5;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreCoverageSectionView {
    pub label: String,
    pub percent: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreCoverageView {
    pub weighted_ratio: f32,
    pub unweighted_ratio: f32,
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    pub sections: Vec<GreCoverageSectionView>,
    pub uncovered_study_labels: Vec<String>,
    pub readiness_available: bool,
    pub readiness_reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreTopicInsightView {
    pub topic_id: String,
    pub display_name: String,
    pub section: String,
    pub exam_weight: f32,
    pub memory_score: Option<f32>,
    pub practice_accuracy: Option<f32>,
    pub studied_cards: u32,
    pub covered: bool,
    pub reason: String,
    pub study_label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAttemptView {
    pub topic: String,
    pub correct: bool,
    pub response_time_ms: u32,
    pub answered_at_secs: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAbstentionRequirementView {
    pub id: String,
    pub label: String,
    pub status: String,
    pub next_step: String,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreDailyTaskView {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub target_count: u32,
    pub topic_id: Option<String>,
    pub topic_display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreQuestionView {
    pub id: String,
    pub topic: String,
    pub section: String,
    pub format: String,
    pub stem: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreTopicMasteryView {
    pub topic_id: String,
    pub display_name: String,
    pub avg_retrievability: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreDashboardView {
    pub computed_at_millis: i64,
    pub readiness_projected: Option<f32>,
    pub readiness_low: Option<f32>,
    pub readiness_high: Option<f32>,
    pub readiness_sufficient: bool,
    pub readiness_summary: String,
    pub readiness_evidence_summary: String,
    pub readiness_abstain_reason: String,
    pub readiness_abstention_requirements: Vec<GreAbstentionRequirementView>,
    pub readiness_confidence_level: String,
    pub memory_value: Option<f32>,
    pub memory_low: Option<f32>,
    pub memory_high: Option<f32>,
    pub memory_sufficient: bool,
    pub memory_detail: String,
    pub memory_abstain_reason: String,
    pub memory_abstention_requirements: Vec<GreAbstentionRequirementView>,
    pub memory_studied_cards: u32,
    pub performance_value: Option<f32>,
    pub performance_low: Option<f32>,
    pub performance_high: Option<f32>,
    pub performance_sufficient: bool,
    pub performance_detail: String,
    pub performance_abstain_reason: String,
    pub performance_abstention_requirements: Vec<GreAbstentionRequirementView>,
    pub performance_attempt_count: u32,
    pub estimated_gre_combined: Option<u32>,
    pub estimated_gre_low: Option<u32>,
    pub estimated_gre_high: Option<u32>,
    pub estimated_gre_preliminary: bool,
    pub coverage: GreCoverageView,
    pub daily_plan_headline: String,
    pub daily_plan_task_count: u32,
    pub daily_plan_rationale: String,
    pub daily_plan_tasks: Vec<GreDailyTaskView>,
    pub study_plan_summary: String,
    pub weak_topic: Option<GreTopicInsightView>,
    pub weak_topic_name: Option<String>,
    pub recommended_topics: Vec<GreTopicInsightView>,
    pub recent_activity: Vec<GreAttemptView>,
    pub recent_accuracy_trend: Vec<f32>,
    pub deck_exists: bool,
    pub deck_name: String,
    pub due_new: u32,
    pub due_learn: u32,
    pub due_review: u32,
    pub due_total: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreProgressView {
    pub computed_at_millis: i64,
    pub memory_value: Option<f32>,
    pub memory_low: Option<f32>,
    pub memory_high: Option<f32>,
    pub memory_sufficient: bool,
    pub memory_detail: String,
    pub performance_value: Option<f32>,
    pub performance_low: Option<f32>,
    pub performance_high: Option<f32>,
    pub performance_sufficient: bool,
    pub performance_detail: String,
    pub performance_attempt_count: u32,
    pub readiness_projected: Option<f32>,
    pub readiness_low: Option<f32>,
    pub readiness_high: Option<f32>,
    pub readiness_sufficient: bool,
    pub readiness_summary: String,
    pub readiness_confidence_level: String,
    pub estimated_gre_combined: Option<u32>,
    pub estimated_gre_low: Option<u32>,
    pub estimated_gre_high: Option<u32>,
    pub estimated_gre_preliminary: bool,
    pub estimated_gre_confidence: String,
    pub weighted_coverage: f32,
    pub unweighted_coverage: f32,
    pub catalog_leaf_count: u32,
    pub covered_leaf_count: u32,
    pub coverage: GreCoverageView,
    pub studied_cards: u32,
    pub topic_count: u32,
    pub mastered_cards: u32,
    pub calibration_assessment: String,
    pub calibration_well_calibrated: bool,
    pub practice_trend: Vec<f32>,
    pub recent_activity: Vec<GreAttemptView>,
    pub weak_topics: Vec<GreTopicInsightView>,
    pub topic_mastery: Vec<GreTopicMasteryView>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrePracticeQueuesView {
    pub all: Vec<String>,
    pub quant: Vec<String>,
    pub verbal: Vec<String>,
    pub awa: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrePracticeScoreStripView {
    pub memory_value: Option<f32>,
    pub memory_low: Option<f32>,
    pub memory_high: Option<f32>,
    pub memory_sufficient: bool,
    pub memory_detail: String,
    pub performance_value: Option<f32>,
    pub performance_low: Option<f32>,
    pub performance_high: Option<f32>,
    pub performance_sufficient: bool,
    pub performance_detail: String,
    pub performance_attempt_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreRecordAttemptInput {
    pub question_id: String,
    pub answer: String,
    pub response_time_ms: u32,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreRecordAttemptResultView {
    pub correct: bool,
    pub explanation: String,
    pub topic: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreExplainAnswerInput {
    pub question_id: String,
    pub selected_answer: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAnswerChoiceExplanationView {
    pub choice: String,
    pub is_correct: bool,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreAnswerExplanationView {
    pub summary: String,
    pub choices: Vec<GreAnswerChoiceExplanationView>,
    pub correct_answer: String,
    pub citation_source_name: String,
    pub citation_source_section: String,
    pub citation_excerpt: String,
    pub provenance: String,
    pub provenance_note: String,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrePracticeBootstrapView {
    pub session_id: String,
    pub question_count: u32,
    pub questions: Vec<GreQuestionView>,
    pub queue: Vec<String>,
    pub queues_by_section: GrePracticeQueuesView,
    pub quant_count: u32,
    pub verbal_count: u32,
    pub awa_count: u32,
    pub memory_value: Option<f32>,
    pub memory_low: Option<f32>,
    pub memory_high: Option<f32>,
    pub memory_sufficient: bool,
    pub memory_detail: String,
    pub performance_value: Option<f32>,
    pub performance_low: Option<f32>,
    pub performance_high: Option<f32>,
    pub performance_sufficient: bool,
    pub performance_detail: String,
    pub performance_attempt_count: u32,
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
    let dashboard = fetch_dashboard(
        backend,
        HOME_RECENT_ACTIVITY_LIMIT,
        HOME_TOPIC_INSIGHT_LIMIT,
    )?;
    let plan = fetch_study_plan(backend, HOME_STUDY_PLAN_LIMIT)?;
    let status = fetch_gre_study_status(backend)?;
    Ok(GreDashboardView::from_responses(dashboard, plan, status))
}

pub fn load_progress_page(backend: &Backend) -> Result<GreProgressView, Vec<u8>> {
    let scores = fetch_scores(backend)?;
    let dashboard = fetch_dashboard(
        backend,
        HOME_RECENT_ACTIVITY_LIMIT,
        PROGRESS_TOPIC_INSIGHT_LIMIT,
    )?;
    let mastery = fetch_topic_mastery(backend, 1)?;
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
        GRE_ATLAS_SERVICE,
        "create_session",
        &CreateSessionRequest {
            source: "practice".into(),
        }
        .encode_to_vec(),
    )?;
    let questions = invoke_proto::<ListQuestionsResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "list_questions",
        &ListQuestionsRequest {
            limit: PRACTICE_QUESTION_LIMIT,
            topic_prefix: String::new(),
        }
        .encode_to_vec(),
    )?;
    let scores = fetch_scores(backend)?;
    Ok(GrePracticeBootstrapView::from_responses(
        session, questions, scores,
    ))
}

pub fn record_practice_attempt(
    backend: &Backend,
    input: GreRecordAttemptInput,
) -> Result<GreRecordAttemptResultView, Vec<u8>> {
    let response = invoke_proto::<RecordAttemptResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "record_attempt",
        &RecordAttemptRequest {
            question_id: input.question_id,
            answer: input.answer,
            response_time_ms: input.response_time_ms,
            confidence: None,
            session_id: Some(input.session_id),
        }
        .encode_to_vec(),
    )?;
    Ok(GreRecordAttemptResultView {
        correct: response.correct,
        explanation: response.explanation,
        topic: response.topic,
    })
}

pub fn explain_practice_answer(
    backend: &Backend,
    input: GreExplainAnswerInput,
) -> Result<GreAnswerExplanationView, Vec<u8>> {
    let response = invoke_proto::<ExplainAnswerResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "explain_answer",
        &ExplainAnswerRequest {
            question_id: input.question_id,
            selected_answer: input.selected_answer,
        }
        .encode_to_vec(),
    )?;
    let explanation = response
        .explanation
        .ok_or_else(|| b"missing explanation".to_vec())?;
    Ok(answer_explanation_view(explanation))
}

pub fn load_practice_score_strip(backend: &Backend) -> Result<GrePracticeScoreStripView, Vec<u8>> {
    Ok(practice_score_strip_from_scores(fetch_scores(backend)?))
}

pub fn load_study_page(backend: &Backend) -> Result<GreStudyView, Vec<u8>> {
    let status = fetch_gre_study_status(backend)?;
    Ok(GreStudyView::from_status(status))
}

fn fetch_scores(backend: &Backend) -> Result<GetScoresResponse, Vec<u8>> {
    invoke_proto(
        backend,
        GRE_ATLAS_SERVICE,
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
        GRE_ATLAS_SERVICE,
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
        GRE_ATLAS_SERVICE,
        "get_study_plan",
        &GetStudyPlanRequest { limit }.encode_to_vec(),
    )
}

fn fetch_gre_study_status(backend: &Backend) -> Result<GreStudyStatusResponse, Vec<u8>> {
    invoke_proto(
        backend,
        GRE_ATLAS_SERVICE,
        "get_gre_study_status",
        &Empty::default().encode_to_vec(),
    )
}

fn fetch_readiness_calibration(backend: &Backend) -> Result<ReadinessCalibrationResponse, Vec<u8>> {
    invoke_proto(
        backend,
        GRE_ATLAS_SERVICE,
        "get_readiness_calibration",
        &Empty::default().encode_to_vec(),
    )
}

fn fetch_topic_mastery(
    backend: &Backend,
    min_reviews: u32,
) -> Result<TopicMasteryResponse, Vec<u8>> {
    invoke_proto(
        backend,
        STATS_SERVICE,
        "topic_mastery",
        &TopicMasteryRequest {
            search: format!(r#"deck:"{GRE_DECK_NAME}" OR deck:"{LEGACY_GRE_DECK_NAME}""#),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            mastery_threshold: None,
            min_reviews,
        }
        .encode_to_vec(),
    )
}

fn coverage_view(coverage: DashboardCoverage) -> GreCoverageView {
    let weighted_pct = (coverage.weighted_ratio * 100.0).round() as u32;
    let readiness_available = coverage.readiness_available;
    GreCoverageView {
        weighted_ratio: coverage.weighted_ratio,
        unweighted_ratio: coverage.unweighted_ratio,
        catalog_leaf_count: coverage.catalog_leaf_count,
        covered_leaf_count: coverage.covered_leaf_count,
        sections: coverage
            .sections
            .into_iter()
            .map(|section| GreCoverageSectionView {
                label: section.label,
                percent: (section.covered_exam_weight * 100.0).round() as u32,
            })
            .collect(),
        uncovered_study_labels: coverage
            .uncovered_topics
            .into_iter()
            .map(|topic| topic.study_label)
            .filter(|label| !label.is_empty())
            .collect(),
        readiness_available,
        readiness_reason: if readiness_available {
            String::new()
        } else {
            format!("Only {weighted_pct}% of the GRE has evidence.")
        },
    }
}

fn topic_insight_view(topic: DashboardTopicInsight) -> GreTopicInsightView {
    GreTopicInsightView {
        topic_id: topic.topic_id,
        display_name: topic.display_name,
        section: topic.section,
        exam_weight: topic.exam_weight,
        memory_score: topic.memory_score,
        practice_accuracy: topic.practice_accuracy,
        studied_cards: topic.studied_cards,
        covered: topic.covered,
        reason: topic.reason,
        study_label: topic.study_label,
    }
}

fn attempt_view(attempt: PerformanceAttempt) -> GreAttemptView {
    GreAttemptView {
        topic: attempt.topic,
        correct: attempt.correct,
        response_time_ms: attempt.response_time_ms,
        answered_at_secs: attempt.answered_at_secs,
    }
}

fn daily_task_view(task: StudyPlanDailyTask) -> GreDailyTaskView {
    GreDailyTaskView {
        id: task.id,
        title: task.title,
        detail: task.detail,
        target_count: task.target_count,
        topic_id: task.topic_id,
        topic_display_name: task.topic_display_name,
    }
}

fn question_view(question: Question) -> GreQuestionView {
    GreQuestionView {
        id: question.id,
        topic: question.topic,
        section: question.section,
        format: question.format,
        stem: question.stem,
        choices: question.choices,
    }
}

fn answer_explanation_view(explanation: AnswerExplanation) -> GreAnswerExplanationView {
    GreAnswerExplanationView {
        summary: explanation.summary,
        choices: explanation
            .choices
            .into_iter()
            .map(|choice| GreAnswerChoiceExplanationView {
                choice: choice.choice,
                is_correct: choice.is_correct,
                reasoning: choice.reasoning,
            })
            .collect(),
        correct_answer: explanation.correct_answer,
        citation_source_name: explanation.citation_source_name,
        citation_source_section: explanation.citation_source_section,
        citation_excerpt: explanation.citation_excerpt,
        provenance: explanation.provenance,
        provenance_note: explanation.provenance_note,
        model_version: explanation.model_version,
    }
}

fn topic_mastery_view(entry: TopicMasteryEntry) -> GreTopicMasteryView {
    GreTopicMasteryView {
        topic_id: entry.topic_id,
        display_name: entry.display_name,
        avg_retrievability: entry.avg_retrievability,
    }
}

fn abstention_requirement_view(req: AbstentionRequirement) -> GreAbstentionRequirementView {
    GreAbstentionRequirementView {
        id: req.id,
        label: req.label,
        status: req.status,
        next_step: req.next_step,
        met: req.met,
    }
}

fn abstention_requirements_view(
    requirements: Vec<AbstentionRequirement>,
) -> Vec<GreAbstentionRequirementView> {
    requirements
        .into_iter()
        .map(abstention_requirement_view)
        .collect()
}

fn rolling_accuracy_series(attempts: &[PerformanceAttempt], window_size: usize) -> Vec<f32> {
    if attempts.is_empty() {
        return Vec::new();
    }
    let ordered: Vec<_> = attempts.iter().rev().collect();
    ordered
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let start = index.saturating_sub(window_size - 1);
            let slice = &ordered[start..=index];
            let correct = slice.iter().filter(|attempt| attempt.correct).count();
            ((correct as f32 / slice.len() as f32) * 100.0).clamp(0.0, 100.0)
        })
        .collect()
}

fn build_question_queue(questions: &[Question]) -> Vec<String> {
    let mut ids: Vec<_> = questions
        .iter()
        .map(|question| question.id.clone())
        .collect();
    ids.shuffle(&mut rand::rng());
    ids
}

fn build_section_queues(questions: &[Question]) -> GrePracticeQueuesView {
    GrePracticeQueuesView {
        all: build_question_queue(questions),
        quant: build_question_queue(
            &questions
                .iter()
                .filter(|question| question.section == "quant")
                .cloned()
                .collect::<Vec<_>>(),
        ),
        verbal: build_question_queue(
            &questions
                .iter()
                .filter(|question| question.section == "verbal")
                .cloned()
                .collect::<Vec<_>>(),
        ),
        awa: build_question_queue(
            &questions
                .iter()
                .filter(|question| question.section == "awa")
                .cloned()
                .collect::<Vec<_>>(),
        ),
    }
}

fn practice_score_strip_from_scores(scores: GetScoresResponse) -> GrePracticeScoreStripView {
    let memory = scores.memory.unwrap_or_default();
    let performance = scores.performance.unwrap_or_default();
    GrePracticeScoreStripView {
        memory_value: memory.value,
        memory_low: memory.value_low,
        memory_high: memory.value_high,
        memory_sufficient: memory.sufficient_data,
        memory_detail: memory.detail,
        performance_value: performance.value,
        performance_low: performance.value_low,
        performance_high: performance.value_high,
        performance_sufficient: performance.sufficient_data,
        performance_detail: performance.detail,
        performance_attempt_count: performance.attempt_count,
    }
}

fn section_counts(questions: &[Question]) -> (u32, u32, u32) {
    let mut quant = 0u32;
    let mut verbal = 0u32;
    let mut awa = 0u32;
    for question in questions {
        match question.section.as_str() {
            "quant" => quant += 1,
            "verbal" => verbal += 1,
            "awa" => awa += 1,
            _ => {}
        }
    }
    (quant, verbal, awa)
}

fn estimated_gre_confidence(
    estimated_preliminary: bool,
    readiness_confidence: &str,
    estimated_sufficient: bool,
) -> String {
    if estimated_preliminary {
        return "Preliminary".into();
    }
    if !readiness_confidence.is_empty() {
        let mut chars = readiness_confidence.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else if estimated_sufficient {
        "Medium".into()
    } else {
        String::new()
    }
}

impl GreDashboardView {
    fn from_responses(
        dashboard: DashboardState,
        plan: StudyPlanResponse,
        status: GreStudyStatusResponse,
    ) -> Self {
        let readiness = dashboard.readiness.unwrap_or_default();
        let estimated = dashboard.estimated_gre.unwrap_or_default();
        let memory = dashboard.memory.unwrap_or_default();
        let performance = dashboard.performance.unwrap_or_default();
        let coverage = dashboard.coverage.unwrap_or_default();
        let daily = plan.daily_plan.unwrap_or_default();
        let weak_topic = dashboard
            .weak_topics
            .first()
            .cloned()
            .map(topic_insight_view);
        let due_total = status.new_count + status.learn_count + status.review_count;
        let recent_accuracy_trend =
            rolling_accuracy_series(&dashboard.recent_activity, PRACTICE_TREND_WINDOW);
        Self {
            computed_at_millis: dashboard.computed_at_millis,
            readiness_projected: readiness.projected_score,
            readiness_low: readiness.projected_score_low,
            readiness_high: readiness.projected_score_high,
            readiness_sufficient: readiness.sufficient_data,
            readiness_summary: if readiness.sufficient_data {
                readiness.evidence_summary.clone()
            } else {
                readiness.abstain_reason.clone()
            },
            readiness_evidence_summary: readiness.evidence_summary,
            readiness_abstain_reason: readiness.abstain_reason,
            readiness_abstention_requirements: abstention_requirements_view(
                readiness.abstention_requirements,
            ),
            readiness_confidence_level: readiness.confidence_level,
            memory_value: memory.value,
            memory_low: memory.value_low,
            memory_high: memory.value_high,
            memory_sufficient: memory.sufficient_data,
            memory_detail: memory.detail,
            memory_abstain_reason: memory.abstain_reason,
            memory_abstention_requirements: abstention_requirements_view(
                memory.abstention_requirements,
            ),
            memory_studied_cards: memory.studied_cards,
            performance_value: performance.value,
            performance_low: performance.value_low,
            performance_high: performance.value_high,
            performance_sufficient: performance.sufficient_data,
            performance_detail: performance.detail,
            performance_abstain_reason: performance.abstain_reason,
            performance_abstention_requirements: abstention_requirements_view(
                performance.abstention_requirements,
            ),
            performance_attempt_count: performance.attempt_count,
            estimated_gre_combined: estimated.combined_score,
            estimated_gre_low: estimated.combined_score_low,
            estimated_gre_high: estimated.combined_score_high,
            estimated_gre_preliminary: estimated.preliminary,
            coverage: coverage_view(coverage),
            daily_plan_headline: daily.headline,
            daily_plan_task_count: daily.tasks.len() as u32,
            daily_plan_rationale: daily.rationale,
            daily_plan_tasks: daily.tasks.into_iter().map(daily_task_view).collect(),
            study_plan_summary: plan.summary,
            weak_topic_name: weak_topic.as_ref().map(|topic| topic.display_name.clone()),
            weak_topic,
            recommended_topics: dashboard
                .recommended_topics
                .into_iter()
                .map(topic_insight_view)
                .collect(),
            recent_activity: dashboard
                .recent_activity
                .into_iter()
                .map(attempt_view)
                .collect(),
            recent_accuracy_trend,
            deck_exists: status.deck_exists,
            deck_name: status.deck_name,
            due_new: status.new_count,
            due_learn: status.learn_count,
            due_review: status.review_count,
            due_total,
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
        let practice_trend =
            rolling_accuracy_series(&dashboard.recent_activity, PRACTICE_TREND_WINDOW);
        let recent_activity: Vec<_> = dashboard
            .recent_activity
            .into_iter()
            .map(attempt_view)
            .collect();
        let readiness_confidence_level = readiness.confidence_level.clone();
        let estimated_gre_confidence = estimated_gre_confidence(
            estimated.preliminary,
            &readiness_confidence_level,
            estimated.sufficient_data,
        );
        Self {
            computed_at_millis: dashboard.computed_at_millis,
            memory_value: memory.value,
            memory_low: memory.value_low,
            memory_high: memory.value_high,
            memory_sufficient: memory.sufficient_data,
            memory_detail: memory.detail,
            performance_value: performance.value,
            performance_low: performance.value_low,
            performance_high: performance.value_high,
            performance_sufficient: performance.sufficient_data,
            performance_detail: performance.detail,
            performance_attempt_count: performance.attempt_count,
            readiness_projected: readiness.projected_score,
            readiness_low: readiness.projected_score_low,
            readiness_high: readiness.projected_score_high,
            readiness_sufficient: readiness.sufficient_data,
            readiness_summary: if readiness.sufficient_data {
                readiness.evidence_summary
            } else {
                readiness.abstain_reason
            },
            readiness_confidence_level,
            estimated_gre_combined: estimated.combined_score,
            estimated_gre_low: estimated.combined_score_low,
            estimated_gre_high: estimated.combined_score_high,
            estimated_gre_preliminary: estimated.preliminary,
            estimated_gre_confidence,
            weighted_coverage: coverage.weighted_ratio,
            unweighted_coverage: coverage.unweighted_ratio,
            catalog_leaf_count: coverage.catalog_leaf_count,
            covered_leaf_count: coverage.covered_leaf_count,
            coverage: coverage_view(coverage),
            studied_cards: mastery_summary.studied_cards,
            topic_count: mastery_summary.topic_count,
            mastered_cards: mastery_summary.mastered_cards,
            calibration_assessment: calibration_stats.assessment,
            calibration_well_calibrated: calibration_stats.well_calibrated,
            practice_trend,
            recent_activity,
            weak_topics: dashboard
                .weak_topics
                .into_iter()
                .map(topic_insight_view)
                .collect(),
            topic_mastery: mastery.topics.into_iter().map(topic_mastery_view).collect(),
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
        let question_views: Vec<_> = questions
            .questions
            .iter()
            .map(|question| question_view(question.clone()))
            .collect();
        let (quant_count, verbal_count, awa_count) = section_counts(&questions.questions);
        let queues_by_section = build_section_queues(&questions.questions);
        Self {
            session_id: session.session_id,
            question_count: questions.questions.len() as u32,
            questions: question_views,
            queue: queues_by_section.all.clone(),
            queues_by_section,
            quant_count,
            verbal_count,
            awa_count,
            memory_value: memory.value,
            memory_low: memory.value_low,
            memory_high: memory.value_high,
            memory_sufficient: memory.sufficient_data,
            memory_detail: memory.detail,
            performance_value: performance.value,
            performance_low: performance.value_low,
            performance_high: performance.value_high,
            performance_sufficient: performance.sufficient_data,
            performance_detail: performance.detail,
            performance_attempt_count: performance.attempt_count,
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

#[cfg(test)]
pub fn normalize_practice_bootstrap(
    mut view: GrePracticeBootstrapView,
) -> GrePracticeBootstrapView {
    view.session_id.clear();
    view.queue.sort();
    view.queues_by_section.all.sort();
    view.queues_by_section.quant.sort();
    view.queues_by_section.verbal.sort();
    view.queues_by_section.awa.sort();
    view
}

#[cfg(test)]
pub fn normalize_dashboard(mut view: GreDashboardView) -> GreDashboardView {
    view.computed_at_millis = 0;
    view
}

#[cfg(test)]
pub fn normalize_progress(mut view: GreProgressView) -> GreProgressView {
    view.computed_at_millis = 0;
    view
}

#[cfg(test)]
mod test {
    use anki::backend::Backend;
    use anki::prelude::I18n;

    use super::*;
    use crate::demo_pages;

    #[test]
    fn rolling_accuracy_matches_recent_window() {
        let attempts = vec![
            PerformanceAttempt {
                topic: "a".into(),
                correct: true,
                ..Default::default()
            },
            PerformanceAttempt {
                topic: "b".into(),
                correct: false,
                ..Default::default()
            },
        ];
        let series = rolling_accuracy_series(&attempts, 5);
        assert_eq!(series.len(), 2);
        assert!((series[0] - 0.0).abs() < f32::EPSILON);
        assert!((series[1] - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dashboard_view_exposes_score_abstention_fields() {
        let backend = Backend::new(I18n::template_only(), false);
        let dir = std::env::temp_dir().join(format!(
            "anki-mobile-dashboard-fields-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let collection = dir.join("collection.anki2");
        open_collection(
            &backend,
            &collection.to_string_lossy(),
            &collection.with_extension("media").to_string_lossy(),
            &collection.with_extension("mdb").to_string_lossy(),
        )
        .expect("open collection");
        demo_pages::prepare_demo_collection(&backend).expect("seed demo");
        let view = load_dashboard_page(&backend).expect("dashboard page");

        assert!(!view.memory_abstention_requirements.is_empty());
        assert!(!view.performance_abstention_requirements.is_empty());
        assert!(!view.readiness_abstention_requirements.is_empty());
        assert!(
            !view.readiness_evidence_summary.is_empty()
                || !view.readiness_abstain_reason.is_empty()
        );
        assert!(!view.daily_plan_tasks.is_empty() || !view.recommended_topics.is_empty());

        let json = serde_json::to_string(&view).expect("serialize dashboard");
        assert!(json.contains("memoryAbstentionRequirements"));
        assert!(json.contains("readinessEvidenceSummary"));

        let _ = std::fs::remove_dir_all(dir);
    }
}
