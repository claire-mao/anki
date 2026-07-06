// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import {
    extraStudyActionLabel,
    extraStudyDetail,
    GRE_EXTRA_STUDY_BATCH,
    GRE_EXTRA_STUDY_DAILY_CAP,
    nextReviewScheduleLabel,
} from "./extra-study";

describe("extra-study helpers", () => {
    it("labels study ahead batches", () => {
        expect(extraStudyActionLabel(1)).toBe("Study 1 more card");
        expect(extraStudyActionLabel(GRE_EXTRA_STUDY_BATCH)).toBe(
            `Study ${GRE_EXTRA_STUDY_BATCH} more cards`,
        );
    });

    it("explains daily pacing cap", () => {
        expect(extraStudyDetail(GRE_EXTRA_STUDY_BATCH)).toContain(
            String(GRE_EXTRA_STUDY_DAILY_CAP),
        );
    });

    it("formats next review schedule", () => {
        expect(nextReviewScheduleLabel(1)).toBe("Flashcards due tomorrow");
        expect(nextReviewScheduleLabel(4)).toBe("Flashcards due in 4 days");
    });
});
