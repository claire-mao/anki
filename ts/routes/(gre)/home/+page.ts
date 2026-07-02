// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getDashboard, getGreStudyStatus, getReadinessCalibration, getStudyPlan } from "@generated/backend";

import type { PageLoad } from "./$types";

export const load = (async () => {
    const [dashboard, plan, status, readinessCalibration] = await Promise.all([
        getDashboard({
            recentActivityLimit: 5,
            topicInsightLimit: 3,
        }),
        getStudyPlan({ limit: 3 }),
        getGreStudyStatus({}),
        getReadinessCalibration({}),
    ]);
    return { dashboard, plan, status, readinessCalibration };
}) satisfies PageLoad;
