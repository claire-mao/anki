# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import aqt
from anki.brainlift import GRE_DECK_NAME
from anki.decks import DeckId
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


def open_gre_page(mw: aqt.main.AnkiQt, path: str) -> None:
    if mw.state == "greDashboard":
        mw.greDashboard.load_page(path)
    else:
        mw.greDashboard._page = path
        mw.moveToState("greDashboard")


def handle_brainlift_bridge_cmd(mw: aqt.main.AnkiQt, cmd: str) -> bool:
    if cmd == "greStartReview":
        start_gre_review(mw)
        return True
    if cmd == "greOpenDashboard":
        open_gre_page(mw, "home")
        return True
    if cmd == "greOpenFullStudy":
        open_brainlift(mw, path="dashboard")
        return True
    if cmd == "greOpenPractice":
        open_gre_page(mw, "practice")
        return True
    if cmd == "greOpenStudyPlan":
        open_brainlift(mw, path="study-plan")
        return True
    if cmd == "greOpenReadiness":
        open_brainlift(mw, path="readiness")
        return True
    if cmd == "greOpenProgress":
        open_gre_page(mw, "progress")
        return True
    if cmd == "greOpenSettings":
        open_gre_page(mw, "settings")
        return True
    if cmd == "greOpenDeckOptions":
        _gre_open_deck_options(mw)
        return True
    if cmd == "greOpenAnkiPreferences":
        mw.onPrefs()
        return True
    if cmd == "greSyncLogin":
        _gre_sync_login(mw)
        return True
    if cmd == "greSyncLogout":
        _gre_sync_logout(mw)
        return True
    return False


def _gre_open_deck_options(mw: aqt.main.AnkiQt) -> None:
    from aqt.deckoptions import display_options_for_deck_id

    deck_id = ensure_gre_study_deck(mw)
    if deck_id is None:
        showWarning(
            f'Create a deck named "{GRE_DECK_NAME}" with your GRE flashcards first.',
            parent=mw,
        )
        return
    display_options_for_deck_id(deck_id)


def _gre_sync_login(mw: aqt.main.AnkiQt) -> None:
    from aqt.sync import sync_login

    def on_success() -> None:
        assert mw.pm.profile is not None
        if mw.pm.profile.get("syncKey"):
            tooltip(tr.preferences_login_successful())

    sync_login(mw, on_success)


def _gre_sync_logout(mw: aqt.main.AnkiQt) -> None:
    from aqt.utils import showWarning

    if mw.media_syncer.is_syncing():
        showWarning("Can't log out while sync in progress.")
        return
    assert mw.pm.profile is not None
    mw.pm.profile["syncKey"] = None
    mw.col.media.force_resync()


def ensure_gre_study_deck(mw: aqt.main.AnkiQt) -> DeckId | None:
    deck_id = mw.col.decks.id(GRE_DECK_NAME)
    if deck_id is None:
        return None
    deck_id = DeckId(deck_id)
    if mw.col.decks.get_current_id() != deck_id:
        mw.col.decks.select(deck_id)
    return deck_id


def start_gre_review(mw: aqt.main.AnkiQt) -> None:
    deck_id = ensure_gre_study_deck(mw)
    if deck_id is None:
        showWarning(
            f'Create a deck named "{GRE_DECK_NAME}" with your GRE flashcards first.',
            parent=mw,
        )
        return

    mw.gre_review_pending_dashboard_refresh = True
    mw.col.startTimebox()
    mw.moveToState("review")
    if mw.state != "review":
        tooltip(tr.studying_no_cards_are_due_yet())
        mw.gre_review_pending_dashboard_refresh = False


def refresh_gre_dashboard(mw: aqt.main.AnkiQt) -> None:
    if mw.state == "greDashboard" and mw.greDashboard._page == "home":
        mw.greDashboard.refresh()
    dialog = aqt.dialogs._dialogs.get("GRE", [None, None])[1]
    if isinstance(dialog, BrainLiftDialog):
        dialog.refresh_dashboard()


def _on_state_did_change(new_state: str, old_state: str) -> None:
    if old_state != "review" or new_state == "review":
        return
    mw = aqt.mw
    if mw is None or not getattr(mw, "gre_review_pending_dashboard_refresh", False):
        return
    mw.gre_review_pending_dashboard_refresh = False
    refresh_gre_dashboard(mw)


def _register_hooks() -> None:
    from aqt import gui_hooks

    gui_hooks.state_did_change.append(_on_state_did_change)


def open_brainlift(mw: aqt.main.AnkiQt, *, path: str = "dashboard") -> None:
    aqt.dialogs.open("GRE", mw, path=path)


class BrainLiftDialog(QDialog):
    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt, path: str = "dashboard") -> None:
        super().__init__(parent=mw, flags=Qt.WindowType.Window)
        mw.garbage_collect_on_dialog_finish(self)
        self.mw = mw
        self.name = "gre"
        self._path = path
        self.setMinimumSize(900, 700)
        disable_help_button(self)
        self.setWindowTitle("GRE")
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.web = AnkiWebView(parent=self, kind=AnkiWebViewKind.BRAINLIFT)
        layout.addWidget(self.web)
        restoreGeom(self, self.name, default_size=(1000, 800))
        self.web.set_bridge_command(
            lambda cmd: handle_brainlift_bridge_cmd(mw, cmd), self
        )
        self._load(path)

    def reopen(self, path: str = "dashboard") -> None:
        self._path = path
        self._load(path)

    def refresh_dashboard(self) -> None:
        self._path = "dashboard"
        self._load("dashboard")

    def _load(self, path: str) -> None:
        self.web.load_sveltekit_page(path)

    def closeWithCallback(self, callback) -> None:
        self.reject()
        callback()

    def reject(self) -> None:
        self.web.cleanup()
        saveGeom(self, self.name)
        aqt.dialogs.markClosed("GRE")
        super().reject()


_register_hooks()
