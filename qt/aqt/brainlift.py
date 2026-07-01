# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import aqt
from anki.brainlift import GRE_DECK_NAME
from anki.decks import DeckId
from aqt.operations.deck import set_current_deck
from aqt.qt import *
from aqt.utils import (
    disable_help_button,
    restoreGeom,
    saveGeom,
    showWarning,
    tooltip,
    tr,
)
from aqt.webview import AnkiWebView, AnkiWebViewKind


def handle_brainlift_bridge_cmd(mw: aqt.main.AnkiQt, cmd: str) -> bool:
    if cmd == "brainliftStartReview":
        start_gre_review(mw)
        return True
    if cmd == "brainliftOpenPractice":
        open_brainlift(mw, path="brainlift/practice")
        return True
    return False


def start_gre_review(mw: aqt.main.AnkiQt) -> None:
    deck_id = mw.col.decks.id(GRE_DECK_NAME)
    if deck_id is None:
        showWarning(
            f'Create a deck named "{GRE_DECK_NAME}" with your GRE flashcards first.',
            parent=mw,
        )
        return

    def after_deck_set(_changes: object) -> None:
        mw.col.startTimebox()
        mw.moveToState("review")
        if mw.state == "overview":
            tooltip(tr.studying_no_cards_are_due_yet())

    set_current_deck(parent=mw, deck_id=DeckId(deck_id)).success(
        after_deck_set
    ).run_in_background()


def open_brainlift(mw: aqt.main.AnkiQt, *, path: str = "brainlift") -> None:
    aqt.dialogs.open("BrainLift", mw, path=path)


class BrainLiftDialog(QDialog):
    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt, path: str = "brainlift") -> None:
        super().__init__(parent=mw, flags=Qt.WindowType.Window)
        mw.garbage_collect_on_dialog_finish(self)
        self.mw = mw
        self.name = "brainlift"
        self._path = path
        self.setMinimumSize(900, 700)
        disable_help_button(self)
        self.setWindowTitle("BrainLift")
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.web = AnkiWebView(parent=self, kind=AnkiWebViewKind.BRAINLIFT)
        layout.addWidget(self.web)
        restoreGeom(self, self.name, default_size=(1000, 800))
        self.web.set_bridge_command(
            lambda cmd: handle_brainlift_bridge_cmd(mw, cmd), self
        )
        self._load(path)

    def reopen(self, path: str = "brainlift") -> None:
        self._path = path
        self._load(path)

    def _load(self, path: str) -> None:
        self.web.load_sveltekit_page(path)

    def closeWithCallback(self, callback) -> None:
        self.reject()
        callback()

    def reject(self) -> None:
        self.web.cleanup()
        saveGeom(self, self.name)
        aqt.dialogs.markClosed("BrainLift")
        super().reject()
