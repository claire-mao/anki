// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreStudyStatusResponse } from "@generated/anki/brainlift_pb";

/** Cards due across the new / learning / review queues for the GRE deck. */
export function dueCardCount(status: GreStudyStatusResponse): number {
    return (
        Number(status.newCount) + Number(status.learnCount) + Number(status.reviewCount)
    );
}

/**
 * Only jump straight into the Anki reviewer when a GRE deck exists AND cards are
 * due. Otherwise the Study page must render its deck-setup empty state or its
 * due-count summary — auto-launching in those cases strands the user on a
 * spinner that never resolves, because `moveToState("review")` bounces back.
 */
export function shouldAutoLaunchReview(status: GreStudyStatusResponse): boolean {
    return status.deckExists && dueCardCount(status) > 0;
}
