// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getDashboard, getGreStudyStatus, getReadinessCalibration } from "@generated/backend";

import type { PageLoad } from "./$types";

export const load = (async () => {
    const [dashboard, status, readinessCalibration] = await Promise.all([
        getDashboard({
            recentActivityLimit: 10,
            topicInsightLimit: 5,
        }),
        getGreStudyStatus({}),
        getReadinessCalibration({}),
    ]);
    return { dashboard, status, readinessCalibration };
}) satisfies PageLoad;
