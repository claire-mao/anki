# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from typing import TYPE_CHECKING

from anki import brainlift_pb2

if TYPE_CHECKING:
    from anki.collection import Collection

GRE_DECK_NAME = "BrainLift GRE"
TOPIC_TAG_PREFIX = "gre::"


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


def get_gre_study_status(col: Collection) -> brainlift_pb2.GreStudyStatusResponse:
    return col._backend.get_gre_study_status()


def get_study_plan(
    col: Collection,
    *,
    limit: int = 10,
) -> brainlift_pb2.StudyPlanResponse:
    return col._backend.get_study_plan(limit=limit)


def get_readiness_calibration(col: Collection) -> brainlift_pb2.ReadinessCalibrationResponse:
    return col._backend.get_readiness_calibration()


def get_brainlift_sync_status(col: Collection) -> brainlift_pb2.BrainLiftSyncStatus:
    return col._backend.get_brain_lift_sync_status()


def pull_brainlift_changes(
    col: Collection, *, after_usn: int = 0, limit: int = 100
) -> brainlift_pb2.BrainLiftSyncPullResponse:
    return col._backend.pull_brain_lift_changes(after_usn=after_usn, limit=limit)


def push_brainlift_changes(
    col: Collection, attempts: list[brainlift_pb2.BrainLiftSyncAttempt]
) -> brainlift_pb2.BrainLiftSyncPushResponse:
    return col._backend.push_brain_lift_changes(attempts=attempts)
