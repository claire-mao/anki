# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from typing import TYPE_CHECKING

from anki import brainlift_pb2

if TYPE_CHECKING:
    from anki.collection import Collection

GRE_DECK_NAME = "BrainLift GRE"
TOPIC_TAG_PREFIX = "gre::"


def list_questions(col: Collection, *, limit: int = 1) -> list[brainlift_pb2.Question]:
    return list(col._backend.list_questions(limit))


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


def get_recent_attempts(
    col: Collection, *, limit: int = 10
) -> list[brainlift_pb2.PerformanceAttempt]:
    return list(col._backend.get_recent_attempts(limit))


def get_gre_study_status(col: Collection) -> brainlift_pb2.GreStudyStatusResponse:
    return col._backend.get_gre_study_status()
