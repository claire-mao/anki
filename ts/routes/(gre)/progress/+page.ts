// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getDashboard, getGreStudyStatus, getReadinessCalibration, getScores, topicMastery } from "@generated/backend";

import type { PageLoad } from "./$types";

const GRE_DECK_NAME = "BrainLift GRE";

export const load = (async () => {
    const [scores, dashboard, mastery, calibration, status] = await Promise.all([
        getScores({}),
        getDashboard({
            recentActivityLimit: 5,
            topicInsightLimit: 8,
        }),
        topicMastery({
            search: `deck:"${GRE_DECK_NAME}"`,
            topicTagPrefix: "gre::",
            minReviews: 1,
        }),
        getReadinessCalibration({}),
        getGreStudyStatus({}),
    ]);

    return {
        scores,
        dashboard,
        mastery,
        readinessCalibration: calibration,
        status,
    };
}) satisfies PageLoad;
