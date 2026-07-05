// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    getDashboard,
    getGreStudyStatus,
    getReadinessCalibration,
    getRecentAttempts,
    getScores,
    topicMastery,
} from "@generated/backend";

import type { PageLoad } from "./$types";

const GRE_DECK_NAME = "GRE Atlas";
const PERFORMANCE_ATTEMPTS_LIMIT = 500;

export const load = (async () => {
    const [scores, dashboard, recentAttempts, mastery, calibration, status] = await Promise.all([
        getScores({}),
        getDashboard({
            recentActivityLimit: 5,
            topicInsightLimit: 8,
        }),
        getRecentAttempts({
            limit: PERFORMANCE_ATTEMPTS_LIMIT,
            topicPrefix: "",
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
        recentAttempts: recentAttempts.attempts,
        mastery,
        readinessCalibration: calibration,
        status,
    };
}) satisfies PageLoad;
