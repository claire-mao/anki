// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getGreStudyStatus, getRecentAttempts, getStudyPlan } from "@generated/backend";

import type { PageLoad } from "./$types";

export const load = (async () => {
    const [plan, status, recentAttemptsResponse] = await Promise.all([
        getStudyPlan({ limit: 10 }),
        getGreStudyStatus({}),
        getRecentAttempts({ limit: 200, topicPrefix: "" }),
    ]);
    return {
        plan,
        status,
        recentAttempts: recentAttemptsResponse.attempts,
    };
}) satisfies PageLoad;
