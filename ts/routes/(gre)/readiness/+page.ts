// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getDashboard, getReadinessCalibration, getScores } from "@generated/backend";

import type { PageLoad } from "./$types";

export const load = (async () => {
    const [response, scores, dashboard] = await Promise.all([
        getReadinessCalibration({}),
        getScores({}),
        getDashboard({
            recentActivityLimit: 1,
            topicInsightLimit: 5,
        }),
    ]);
    return { response, scores, dashboard };
}) satisfies PageLoad;
