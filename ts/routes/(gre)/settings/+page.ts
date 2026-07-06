// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DeckConfigsForUpdate } from "@generated/anki/deck_config_pb";
import type { Preferences } from "@generated/anki/config_pb";
import {
    getDeckConfigsForUpdate,
    getDeckIdByName,
    getGreAtlasAiSettings,
    getGreStudyStatus,
    getPreferences,
    getScores,
} from "@generated/backend";
import {
    GetScoresResponse,
    GreStudyStatusResponse,
} from "@generated/anki/brainlift_pb";

import { ensureGreAtlasStudyDeck } from "../study-bootstrap";

import type { PageLoad } from "./$types";

const GRE_DECK_NAME = "GRE Atlas";
const silent = { alertOnError: false } as const;

export const load = (async () => {
    await ensureGreAtlasStudyDeck();

    const [preferences, studyStatus, scores, aiSettings] = await Promise.all([
        getPreferences({}, silent).catch(() => null),
        getGreStudyStatus({}, silent).catch(() => new GreStudyStatusResponse()),
        getScores({}, silent).catch(() => new GetScoresResponse()),
        getGreAtlasAiSettings({}, silent).catch(() => null),
    ]);

    let deckId: bigint | null = null;
    let deckConfigs: DeckConfigsForUpdate | null = null;

    if (studyStatus.deckExists) {
        try {
            const deckIdResponse = await getDeckIdByName({ val: GRE_DECK_NAME }, silent);
            deckId = deckIdResponse.did;
            deckConfigs = await getDeckConfigsForUpdate({ did: deckId }, silent);
        } catch {
            deckId = null;
            deckConfigs = null;
        }
    }

    return {
        preferences: preferences as Preferences,
        studyStatus,
        scores,
        aiSettings,
        deckId,
        deckConfigs,
        loadError: preferences === null,
    };
}) satisfies PageLoad;
