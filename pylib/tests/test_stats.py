# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import os
import tempfile

from anki.collection import CardStats
from anki.consts import CARD_TYPE_REV
from tests.shared import getEmptyCol


def _graduate_card(col, card):
    while card.type != CARD_TYPE_REV:
        card = col.get_card(card.id)
        card.start_timer()
        col.sched.answerCard(card, 4)
        card = col.get_card(card.id)
    return card


def test_stats():
    col = getEmptyCol()
    note = col.newNote()
    note["Front"] = "foo"
    col.addNote(note)
    c = note.cards()[0]
    # card stats
    card_stats = col.card_stats_data(c.id)
    assert card_stats.note_id == note.id
    c = col.sched.getCard()
    col.sched.answerCard(c, 3)
    col.sched.answerCard(c, 2)
    card_stats = col.card_stats_data(c.id)
    assert len(card_stats.revlog) == 2


def test_graphs_empty():
    col = getEmptyCol()
    assert col.stats().report()


def test_graphs():
    dir = tempfile.gettempdir()
    col = getEmptyCol()
    g = col.stats()
    rep = g.report()
    with open(os.path.join(dir, "test.html"), "w", encoding="UTF-8") as note:
        note.write(rep)
    return


def test_topic_mastery_returns_catalog_and_observed_tags():
    col = getEmptyCol()
    note = col.newNote()
    note["Front"] = "hypothesis"
    note.tags = ["gre::quant::algebra"]
    col.addNote(note)
    c = col.sched.getCard()
    assert c is not None
    col.sched.answerCard(c, 3)

    resp = col.topic_mastery(search="deck:current")
    assert resp.summary.topic_count > 0
    assert 0.0 <= resp.summary.coverage_ratio <= 1.0

    algebra = next(t for t in resp.topics if t.topic_id == "gre::quant::algebra")
    assert algebra.studied_cards >= 1
    assert algebra.total_cards >= 1
    assert algebra.display_name == "Algebra"
    assert algebra.avg_retrievability_low <= algebra.avg_retrievability
    assert algebra.avg_retrievability_high >= algebra.avg_retrievability


def test_topic_mastery_fsrs_retrievability_and_mastered():
    col = getEmptyCol()
    col.set_config("fsrs", True)

    for front in ("a", "b", "c"):
        note = col.newNote()
        note["Front"] = front
        note.tags = ["gre::quant::algebra::linear"]
        col.addNote(note)
        _graduate_card(col, note.cards()[0])

    resp = col.topic_mastery(
        search="deck:current",
        mastery_threshold=0.5,
    )
    topic = next(t for t in resp.topics if t.topic_id == "gre::quant::algebra::linear")
    assert topic.studied_cards == 3
    assert topic.total_cards == 3
    assert topic.avg_retrievability > 0.0
    assert topic.mastered_cards >= 1
    assert resp.summary.studied_cards == 3
    assert resp.summary.mastered_cards >= 1
    assert resp.fsrs_enabled is True


def test_topic_mastery_catalog_coverage_partial():
    col = getEmptyCol()
    col.set_config("fsrs", True)

    note = col.newNote()
    note["Front"] = "single topic"
    note.tags = ["gre::quant::data_interpretation"]
    col.addNote(note)
    card = col.sched.getCard()
    col.sched.answerCard(card, 3)

    resp = col.topic_mastery(search="deck:current")
    assert resp.summary.coverage_ratio > 0.0
    assert resp.summary.coverage_ratio < 1.0
    assert not resp.summary.sufficient_data

