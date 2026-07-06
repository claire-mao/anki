// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    getDashboard,
    getGreStudyStatus,
    getPerformanceChart,
    getReadinessCalibration,
    getScores,
    topicMastery,
} from "@generated/backend";
import { PerformanceChartHorizon } from "@generated/anki/brainlift_pb";

import type { PageLoad } from "./$types";

const GRE_DECK_NAME = "GRE Atlas";

export const load = (async () => {
    const [scores, dashboard, performanceChart, mastery, calibration, status] = await Promise.all([
        getScores({}),
        getDashboard({
            recentActivityLimit: 5,
            topicInsightLimit: 8,
        }),
        getPerformanceChart({
            horizon: PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_30D,
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
        performanceChart,
        mastery,
        readinessCalibration: calibration,
        status,
    };
}) satisfies PageLoad;
