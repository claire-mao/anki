// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getDashboard, getGreStudyStatus, getStudyPlan } from "@generated/backend";

import { ensureGreAtlasStudyDeck } from "../study-bootstrap";

import type { PageLoad } from "./$types";

export const load = (async () => {
    await ensureGreAtlasStudyDeck();

    const [status, dashboard, plan] = await Promise.all([
        getGreStudyStatus({}),
        getDashboard({
            recentActivityLimit: 3,
            topicInsightLimit: 3,
        }),
        getStudyPlan({ limit: 3 }),
    ]);
    return { status, dashboard, plan };
}) satisfies PageLoad;
