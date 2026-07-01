# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from anki.brainlift import GRE_DECK_NAME, record_attempt
from tests.shared import getEmptyCol


def test_brainlift_record_attempt() -> None:
    col = getEmptyCol()
    questions = col._backend.list_questions(1)
    assert questions
    q = questions[0]
    resp = record_attempt(
        col,
        question_id=q.id,
        answer="definitely wrong",
        response_time_ms=1500,
    )
    assert resp.correct is False
    assert resp.explanation

    scores = col._backend.get_scores()
    perf = scores.performance
    assert perf is not None
    assert perf.sufficient_data
    assert perf.value == 0.0


def test_gre_study_status_missing_deck() -> None:
    col = getEmptyCol()
    status = col._backend.get_gre_study_status()
    assert status.deck_name == GRE_DECK_NAME
    assert status.deck_exists is False
