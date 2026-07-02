// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Preferences } from "@generated/anki/config_pb";
import { setPreferences } from "@generated/backend";

let saveChain: Promise<unknown> = Promise.resolve();

export function queuePreferencesSave(prefs: Preferences): void {
    saveChain = saveChain
        .then(() => setPreferences(prefs))
        .catch((error) => {
            console.error("Failed to save preferences", error);
        });
}
