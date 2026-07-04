// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DeckConfigsForUpdate } from "@generated/anki/deck_config_pb";
import {
    getDeckConfigsForUpdate,
    getDeckIdByName,
    getGreStudyStatus,
    getPreferences,
    getScores,
} from "@generated/backend";

import { ensureGreAtlasStudyDeck } from "../study-bootstrap";

import type { PageLoad } from "./$types";

const GRE_DECK_NAME = "GRE Atlas";

export const load = (async () => {
    await ensureGreAtlasStudyDeck();

    const [preferences, studyStatus, scores] = await Promise.all([
        getPreferences({}),
        getGreStudyStatus({}),
        getScores({}),
    ]);

    let deckId: bigint | null = null;
    let deckConfigs: DeckConfigsForUpdate | null = null;

    if (studyStatus.deckExists) {
        try {
            const deckIdResponse = await getDeckIdByName({ val: GRE_DECK_NAME });
            deckId = deckIdResponse.did;
            deckConfigs = await getDeckConfigsForUpdate(
                { did: deckId },
                { alertOnError: false },
            );
        } catch {
            deckId = null;
            deckConfigs = null;
        }
    }

    return {
        preferences,
        studyStatus,
        scores,
        deckId,
        deckConfigs,
    };
}) satisfies PageLoad;
