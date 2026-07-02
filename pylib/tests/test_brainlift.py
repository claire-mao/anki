# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import os
import tempfile

from anki import brainlift_pb2
from anki.brainlift import (
    GRE_DECK_NAME,
    create_session,
    get_dashboard,
    get_study_plan,
    get_readiness_calibration,
    get_question,
    get_recent_attempts,
    list_questions,
    record_attempt,
)
from anki.collection import Collection
from tests.shared import getEmptyCol


def isolated_col() -> Collection:
    """Use a dedicated folder so each test gets its own brainlift.db."""
    directory = tempfile.mkdtemp()
    return Collection(os.path.join(directory, "collection.anki2"))


def test_list_questions_returns_protobuf_response() -> None:
    col = isolated_col()
    resp = list_questions(col, limit=100)
    assert isinstance(resp, brainlift_pb2.ListQuestionsResponse)
    assert len(resp.questions) >= 9
    assert all(q.id and q.stem and q.topic for q in resp.questions)


def test_list_questions_filters_by_topic() -> None:
    col = isolated_col()
    quant = list_questions(col, limit=100, topic_prefix="gre::quant")
    assert quant.questions
    assert all(q.topic.startswith("gre::quant") for q in quant.questions)
    linear = list_questions(
        col, limit=10, topic_prefix="gre::quant::algebra::linear"
    )
    assert len(linear.questions) == 1
    assert linear.questions[0].id == "gre-quant-alg-001"


def test_get_question_returns_protobuf() -> None:
    col = isolated_col()
    listed = list_questions(col, limit=1)
    qid = listed.questions[0].id
    question = get_question(col, qid)
    assert isinstance(question, brainlift_pb2.Question)
    assert question.id == qid
    assert question.stem
    assert question.choices


def test_brainlift_record_attempt() -> None:
    col = isolated_col()
    listed = list_questions(col, limit=1)
    q = listed.questions[0]
    session = create_session(col, source="practice")
    resp = record_attempt(
        col,
        question_id=q.id,
        answer="definitely wrong",
        response_time_ms=1500,
        confidence=2,
        session_id=session.session_id,
    )
    assert isinstance(resp, brainlift_pb2.RecordAttemptResponse)
    assert resp.correct is False
    assert resp.explanation

    scores = col._backend.get_scores()
    perf = scores.performance
    assert perf is not None
    assert not perf.sufficient_data
    assert perf.attempt_count == 1
    assert not perf.HasField("value")

    readiness = scores.readiness
    assert readiness is not None
    assert not readiness.sufficient_data
    assert not readiness.HasField("projected_score")
    assert readiness.evidence_summary

    attempts = get_recent_attempts(col, limit=1)
    assert isinstance(attempts, brainlift_pb2.GetRecentAttemptsResponse)
    assert len(attempts.attempts) == 1
    attempt = attempts.attempts[0]
    assert attempt.question_id == q.id
    assert attempt.topic == resp.topic
    assert attempt.answer == "definitely wrong"
    assert attempt.correct is False
    assert attempt.response_time_ms == 1500
    assert attempt.confidence == 2
    assert attempt.session_id == session.session_id
    assert attempt.answered_at_secs > 0


def test_get_recent_attempts_filters_by_topic() -> None:
    col = isolated_col()
    session = create_session(col)
    quant_q = list_questions(col, limit=1, topic_prefix="gre::quant").questions[0]
    verbal_q = list_questions(col, limit=1, topic_prefix="gre::verbal").questions[0]
    record_attempt(
        col,
        question_id=quant_q.id,
        answer="wrong",
        response_time_ms=1000,
        session_id=session.session_id,
    )
    record_attempt(
        col,
        question_id=verbal_q.id,
        answer="wrong",
        response_time_ms=1000,
        session_id=session.session_id,
    )
    quant_attempts = get_recent_attempts(col, limit=10, topic_prefix="gre::quant")
    assert all(a.topic.startswith("gre::quant") for a in quant_attempts.attempts)
    assert len(quant_attempts.attempts) == 1


def test_brainlift_record_attempt_does_not_modify_revlog() -> None:
    col = isolated_col()
    revlog_before = col.db.scalar("select count() from revlog")
    q = list_questions(col, limit=1).questions[0]
    session = create_session(col)
    record_attempt(
        col,
        question_id=q.id,
        answer="wrong",
        response_time_ms=800,
        session_id=session.session_id,
    )
    revlog_after = col.db.scalar("select count() from revlog")
    assert revlog_before == revlog_after


def test_brainlift_create_session() -> None:
    col = isolated_col()
    session = create_session(col, source="post_review")
    assert isinstance(session, brainlift_pb2.CreateSessionResponse)
    assert session.session_id
    assert session.started_at_secs > 0
    assert session.source == "post_review"


def test_gre_study_status_missing_deck() -> None:
    col = getEmptyCol()
    status = col._backend.get_gre_study_status()
    assert status.deck_name == GRE_DECK_NAME
    assert status.deck_exists is False


def test_dashboard_memory_reflects_gre_deck_reviews() -> None:
    col = isolated_col()
    gre_deck = col.decks.id(GRE_DECK_NAME)
    col.decks.select(gre_deck)

    note = col.newNote()
    note["Front"] = "hypothesis"
    note.tags = ["gre::quant::algebra::linear"]
    col.addNote(note)
    col.set_deck([note.cards()[0].id], gre_deck)

    before = get_dashboard(col)
    assert before.memory is not None
    assert before.memory.studied_cards == 0

    card = col.sched.getCard()
    assert card is not None
    col.sched.answerCard(card, 3)

    after = get_dashboard(col)
    assert after.memory is not None
    assert after.memory.studied_cards >= 1


def test_readiness_abstains_without_minimum_evidence() -> None:
    col = isolated_col()
    scores = col._backend.get_scores()
    readiness = scores.readiness
    assert readiness is not None
    assert not readiness.sufficient_data
    assert not readiness.HasField("projected_score")
    assert readiness.abstain_reason
    assert readiness.evidence_summary
    assert readiness.last_updated_millis > 0


def test_get_dashboard_returns_full_state() -> None:
    col = isolated_col()
    session = create_session(col)
    q = list_questions(col, limit=1).questions[0]
    record_attempt(
        col,
        question_id=q.id,
        answer="wrong",
        response_time_ms=500,
        session_id=session.session_id,
    )

    state = get_dashboard(col, recent_activity_limit=5, topic_insight_limit=3)
    assert isinstance(state, brainlift_pb2.DashboardState)
    assert state.memory is not None
    assert state.performance is not None
    assert state.readiness is not None
    assert state.coverage is not None
    assert state.coverage.catalog_leaf_count > 0
    assert len(state.recommended_topics) > 0
    assert len(state.weak_topics) > 0
    assert len(state.recent_activity) == 1
    assert state.computed_at_millis > 0
    assert state.recent_activity[0].question_id == q.id


def test_get_dashboard_matches_scores_subset() -> None:
    col = isolated_col()
    dashboard = get_dashboard(col)
    scores = col._backend.get_scores()
    assert dashboard.memory.sufficient_data == scores.memory.sufficient_data
    assert dashboard.performance.attempt_count == scores.performance.attempt_count
    assert dashboard.readiness.abstain_reason == scores.readiness.abstain_reason


def test_get_study_plan_returns_ranked_recommendations() -> None:
    import math

    col = isolated_col()
    plan = get_study_plan(col, limit=5)
    assert isinstance(plan, brainlift_pb2.StudyPlanResponse)
    assert plan.coverage is not None
    assert plan.coverage.catalog_leaf_count > 0
    assert len(plan.recommendations) > 0
    assert plan.summary
    assert plan.computed_at_millis > 0

    previous = math.inf
    for topic in plan.recommendations:
        assert topic.explanation
        assert topic.factors
        assert topic.priority_score > 0
        assert topic.priority_score <= previous
        previous = topic.priority_score

    daily = plan.daily_plan
    assert daily.headline
    assert daily.rationale
    assert len(daily.tasks) >= 2
    assert any(task.id == "review_cards" for task in daily.tasks)
    assert any(task.id == "practice_questions" for task in daily.tasks)


def test_readiness_abstention_lists_missing_requirements() -> None:
    col = isolated_col()
    scores = col._backend.get_scores()
    readiness = scores.readiness
    assert not readiness.sufficient_data
    assert not readiness.HasField("projected_score")
    assert readiness.abstain_reason
    assert len(readiness.abstention_requirements) == 4
    unmet = [req for req in readiness.abstention_requirements if not req.met]
    assert len(unmet) >= 3
    for req in unmet:
        assert req.id
        assert req.label
        assert req.status
        assert req.next_step

    memory = scores.memory
    assert len(memory.abstention_requirements) == 3
    performance = scores.performance
    assert len(performance.abstention_requirements) == 1
    assert performance.abstention_requirements[0].id == "practice_attempts"
    assert not performance.abstention_requirements[0].met


def test_get_readiness_calibration_reports_honest_stats() -> None:
    col = isolated_col()
    response = get_readiness_calibration(col)
    assert isinstance(response, brainlift_pb2.ReadinessCalibrationResponse)
    assert response.readiness is not None
    assert response.calibration is not None
    assert response.calibration.assessment
    assert response.computed_at_millis > 0
    assert not response.calibration.well_calibrated or response.calibration.brier_score is not None


def test_brainlift_sync_pull_push_roundtrip() -> None:
    col = isolated_col()
    status = col._backend.get_brain_lift_sync_status()
    assert status.current_usn >= 0

    session = create_session(col)
    q = list_questions(col, limit=1).questions[0]
    record_attempt(
        col,
        question_id=q.id,
        answer=q.choices[0] if q.choices else "A",
        response_time_ms=800,
        session_id=session.session_id,
    )

    pulled = col._backend.pull_brain_lift_changes(after_usn=0, limit=10)
    assert len(pulled.attempts) == 1
    push = col._backend.push_brain_lift_changes(attempts=[])
    assert push.applied_count == 0
