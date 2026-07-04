// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreStudyStatusResponse } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import { dueCardCount, shouldAutoLaunchReview } from "./study-launch";

function status(overrides: Partial<GreStudyStatusResponse>): GreStudyStatusResponse {
    return {
        deckExists: true,
        deckName: "GRE Atlas",
        newCount: 0,
        learnCount: 0,
        reviewCount: 0,
        ...overrides,
    } as GreStudyStatusResponse;
}

describe("study auto-launch gating", () => {
    test("does not auto-launch when the GRE deck is missing", () => {
        // Regression: Study must show its "create a deck" empty state, not a
        // perpetual "Opening…" spinner.
        expect(shouldAutoLaunchReview(status({ deckExists: false, newCount: 5 }))).toBe(
            false,
        );
    });

    test("does not auto-launch when no cards are due", () => {
        // Regression: Study must show the due-count summary, not a spinner.
        expect(shouldAutoLaunchReview(status({ deckExists: true }))).toBe(false);
    });

    test("auto-launches only when a deck exists and cards are due", () => {
        expect(shouldAutoLaunchReview(status({ reviewCount: 3 }))).toBe(true);
        expect(shouldAutoLaunchReview(status({ newCount: 1 }))).toBe(true);
        expect(shouldAutoLaunchReview(status({ learnCount: 2 }))).toBe(true);
    });

    test("dueCardCount sums the new/learning/review queues", () => {
        expect(dueCardCount(status({ newCount: 1, learnCount: 2, reviewCount: 4 }))).toBe(
            7,
        );
    });
});
