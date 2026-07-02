// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getTopicDetails } from "@generated/backend";

import { decodeTopicIdParam } from "../../topic-link";
import type { PageLoad } from "./$types";

export const load = (async ({ params }) => {
    const topicId = decodeTopicIdParam(params.topicId);
    const details = await getTopicDetails({
        topicId,
        practiceQuestionLimit: 12,
        recentAttemptLimit: 10,
    });

    return { details };
}) satisfies PageLoad;
