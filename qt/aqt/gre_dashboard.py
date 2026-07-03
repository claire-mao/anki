# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from collections.abc import Callable

import aqt
from anki.collection import OpChanges
from aqt.gre_atlas import handle_gre_atlas_bridge_cmd
from aqt.sound import av_player


class GreDashboard:
    """Primary GRE shell in the main window (dashboard, practice, etc.)."""

    def __init__(self, mw: aqt.AnkiQt) -> None:
        self.mw = mw
        self.web = mw.web
        self._refresh_needed = False
        self._page = "home"

    def show(self, page: str | None = None) -> None:
        if page is not None:
            self._page = page
        av_player.stop_and_clear_queue()
        self.web.set_bridge_command(self._linkHandler, self)
        self.mw.setStateShortcuts(self._shortcutKeys())
        self._hide_anki_chrome()
        self.refresh()

    def load_page(self, page: str) -> None:
        self._page = page
        self._refresh_needed = False
        self.web.load_sveltekit_page(page)
        self._hide_anki_chrome()
        self.mw.web.setFocus()

    def refresh(self) -> None:
        self._refresh_needed = False
        self.web.load_sveltekit_page(self._page)
        self._hide_anki_chrome()
        self.mw.web.setFocus()

    def refresh_if_needed(self) -> None:
        if self._refresh_needed:
            self.refresh()

    def op_executed(
        self, changes: OpChanges, handler: object | None, focused: bool
    ) -> bool:
        if changes.study_queues or changes.note:
            self._refresh_needed = True

        if focused:
            self.refresh_if_needed()

        return self._refresh_needed

    def _linkHandler(self, url: str) -> bool:
        if handle_gre_atlas_bridge_cmd(self.mw, url):
            return False
        if url == "decks":
            self.mw.onOpenDebugDeckBrowser()
        return False

    def _shortcutKeys(self) -> list[tuple[str, Callable]]:
        return [
            ("s", self.mw.onStudyKey),
        ]

    def _hide_anki_chrome(self) -> None:
        "GRE pages provide their own header navigation; hide duplicate Anki toolbars."
        self.mw.toolbarWeb.hide()
        self.mw.bottomWeb.hide()
