// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreStudyStatusResponse } from "@generated/anki/brainlift_pb";

export const GRE_EXTRA_STUDY_BATCH = 8;
export const GRE_EXTRA_STUDY_DAILY_CAP = 20;

export function nextReviewScheduleLabel(days: number | undefined): string | undefined {
    if (days === undefined) {
        return undefined;
    }
    if (days === 0) {
        return "Flashcards due today";
    }
    if (days === 1) {
        return "Flashcards due tomorrow";
    }
    return `Flashcards due in ${days} days`;
}

export function extraStudyActionLabel(count: number): string {
    if (count <= 1) {
        return "Study 1 more card";
    }
    return `Study ${count} more cards`;
}

export function extraStudyDetail(count: number): string {
    return `Unlock up to ${count} new flashcard${count === 1 ? "" : "s"} today (${GRE_EXTRA_STUDY_DAILY_CAP} max per day) to build memory evidence without cramming.`;
}

export function canStartExtraStudy(status: GreStudyStatusResponse): boolean {
    return Number(status.extraStudyAvailable) > 0;
}
