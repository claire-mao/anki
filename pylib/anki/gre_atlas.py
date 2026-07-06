# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from typing import TYPE_CHECKING

from anki import brainlift_pb2, sync_pb2

if TYPE_CHECKING:
    from anki.collection import Collection

GRE_ATLAS_DB_NAME = "greatlas.db"
LEGACY_BRAINLIFT_DB_NAME = "brainlift.db"
GRE_DECK_NAME = "GRE Atlas"
LEGACY_GRE_DECK_NAME = "BrainLift GRE"
TOPIC_TAG_PREFIX = "gre::"


def resolve_gre_deck_id(col: Collection) -> int | None:
    deck_id = col.decks.id_for_name(GRE_DECK_NAME)
    if deck_id is not None:
        return deck_id
    return col.decks.id_for_name(LEGACY_GRE_DECK_NAME)


def list_questions(
    col: Collection,
    *,
    limit: int = 10,
    topic_prefix: str = "",
) -> brainlift_pb2.ListQuestionsResponse:
    resp = brainlift_pb2.ListQuestionsResponse()
    resp.questions.extend(
        col._backend.list_questions(limit=limit, topic_prefix=topic_prefix)
    )
    return resp


def get_question(col: Collection, question_id: str) -> brainlift_pb2.Question:
    return col._backend.get_question(question_id)


def create_session(
    col: Collection, *, source: str = "practice"
) -> brainlift_pb2.CreateSessionResponse:
    return col._backend.create_session(source)


def record_attempt(
    col: Collection,
    *,
    question_id: str,
    answer: str,
    response_time_ms: int,
    confidence: int | None = None,
    session_id: str | None = None,
) -> brainlift_pb2.RecordAttemptResponse:
    req = brainlift_pb2.RecordAttemptRequest(
        question_id=question_id,
        answer=answer,
        response_time_ms=response_time_ms,
    )
    if confidence is not None:
        req.confidence = confidence
    if session_id is not None:
        req.session_id = session_id
    return col._backend.record_attempt(req)


def get_scores(col: Collection) -> brainlift_pb2.GetScoresResponse:
    return col._backend.get_scores()


def get_dashboard(
    col: Collection,
    *,
    recent_activity_limit: int = 10,
    topic_insight_limit: int = 5,
) -> brainlift_pb2.DashboardState:
    return col._backend.get_dashboard(
        recent_activity_limit=recent_activity_limit,
        topic_insight_limit=topic_insight_limit,
    )


def get_recent_attempts(
    col: Collection,
    *,
    limit: int = 10,
    topic_prefix: str = "",
) -> brainlift_pb2.GetRecentAttemptsResponse:
    resp = brainlift_pb2.GetRecentAttemptsResponse()
    resp.attempts.extend(
        col._backend.get_recent_attempts(limit=limit, topic_prefix=topic_prefix)
    )
    return resp


def get_performance_chart(
    col: Collection,
    *,
    horizon: brainlift_pb2.PerformanceChartHorizon.ValueType,
    topic_prefix: str = "",
) -> brainlift_pb2.GetPerformanceChartResponse:
    return col._backend.get_performance_chart(
        horizon=horizon,
        topic_prefix=topic_prefix,
    )


def get_gre_study_status(col: Collection) -> brainlift_pb2.GreStudyStatusResponse:
    return col._backend.get_gre_study_status()


def prepare_demo_collection(
    col: Collection,
) -> brainlift_pb2.PrepareDemoCollectionResponse:
    return col._backend.prepare_demo_collection()


def get_study_plan(
    col: Collection,
    *,
    limit: int = 10,
) -> brainlift_pb2.StudyPlanResponse:
    return col._backend.get_study_plan(limit=limit)


def get_topic_details(
    col: Collection,
    topic_id: str,
    *,
    practice_question_limit: int = 12,
    recent_attempt_limit: int = 10,
) -> brainlift_pb2.TopicDetailsResponse:
    return col._backend.get_topic_details(
        topic_id=topic_id,
        practice_question_limit=practice_question_limit,
        recent_attempt_limit=recent_attempt_limit,
    )


def get_readiness_calibration(
    col: Collection,
) -> brainlift_pb2.ReadinessCalibrationResponse:
    return col._backend.get_readiness_calibration()


def get_performance_eval(
    col: Collection,
) -> brainlift_pb2.PerformanceEvalResponse:
    return col._backend.get_performance_eval()


def get_memory_eval(
    col: Collection,
) -> brainlift_pb2.MemoryEvalResponse:
    return col._backend.get_memory_eval()


def generate_gre_atlas_eval_report(
    col: Collection,
) -> brainlift_pb2.BrainLiftEvalReportResponse:
    return col._backend.generate_brain_lift_eval_report()


def generate_gre_atlas_ai_eval_report(
    col: Collection,
) -> brainlift_pb2.BrainLiftAiEvalReportResponse:
    return col._backend.generate_brain_lift_ai_eval_report()


def generate_question(
    col: Collection,
    *,
    topic_id: str,
    persist: bool = False,
) -> brainlift_pb2.GenerateQuestionResponse:
    return col._backend.generate_question(topic_id=topic_id, persist=persist)


def explain_answer(
    col: Collection,
    *,
    question_id: str,
    selected_answer: str = "",
) -> brainlift_pb2.AnswerExplanation:
    """Post-answer explanation for a stored question.

    Uses the optional LLM path when ``GRE_ATLAS_OPENAI_API_KEY`` is set and the
    provider is reachable; otherwise falls back to a deterministic templated
    explanation. Never raises for AI-unavailability.
    """
    return col._backend.explain_answer(
        question_id=question_id, selected_answer=selected_answer
    )


def get_gre_atlas_sync_status(col: Collection) -> brainlift_pb2.BrainLiftSyncStatus:
    return col._backend.get_brain_lift_sync_status()


def pull_gre_atlas_changes(
    col: Collection, *, after_usn: int = 0, limit: int = 100
) -> brainlift_pb2.BrainLiftSyncPullResponse:
    return col._backend.pull_brain_lift_changes(after_usn=after_usn, limit=limit)


def push_gre_atlas_changes(
    col: Collection, attempts: list[brainlift_pb2.BrainLiftSyncAttempt]
) -> brainlift_pb2.BrainLiftSyncPushResponse:
    return col._backend.push_brain_lift_changes(attempts=attempts)


def pull_gre_atlas_sync_bundle(
    col: Collection, *, after_usn: int = 0, limit: int = 5000
) -> brainlift_pb2.BrainLiftSyncBundle:
    return col._backend.pull_brain_lift_sync_bundle(after_usn=after_usn, limit=limit)


def push_gre_atlas_sync_bundle(
    col: Collection, bundle: brainlift_pb2.BrainLiftSyncBundle
) -> brainlift_pb2.BrainLiftSyncBundlePushResponse:
    return col._backend.push_brain_lift_sync_bundle(bundle=bundle)


def perform_gre_atlas_sync(
    col: Collection,
    *,
    hkey: str | None = None,
    endpoint: str | None = None,
    io_timeout_secs: int | None = None,
) -> brainlift_pb2.PerformGreAtlasSyncResponse:
    """Download remote GRE Atlas changes, merge locally, and upload pending rows.

    When ``hkey`` is omitted, performs a local status refresh only.
    """
    auth = None
    if hkey:
        auth = sync_pb2.SyncAuth(
            hkey=hkey, endpoint=endpoint, io_timeout_secs=io_timeout_secs
        )
    request = brainlift_pb2.PerformGreAtlasSyncRequest(auth=auth)
    return col._backend.perform_gre_atlas_sync(request)
