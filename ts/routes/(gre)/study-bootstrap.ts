// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { prepareDemoCollection } from "@generated/backend";

/**
 * Idempotent: creates the GRE Atlas deck, seeds built-in flashcards when the
 * deck is empty, and ensures starter practice history exists.
 */
export async function ensureGreAtlasStudyDeck(): Promise<void> {
    await prepareDemoCollection({}, { alertOnError: false }).catch(() => undefined);
}
