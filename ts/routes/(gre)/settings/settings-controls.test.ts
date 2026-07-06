// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { Preferences_Scheduling_NewReviewMix } from "@generated/anki/config_pb";
import { describe, expect, test } from "vitest";

import { newReviewMixLabel, newReviewMixOptions } from "./settings-controls";

describe("newReviewMixOptions", () => {
    test("exposes three selectable labels for scheduling mix", () => {
        expect(newReviewMixOptions).toHaveLength(3);
        expect(newReviewMixOptions.map((option) => option.label)).toEqual([
            "Mix new and review cards",
            "Show review cards first",
            "Show new cards first",
        ]);
    });

    test("uses unique backend enum values", () => {
        const values = newReviewMixOptions.map((option) => option.value);
        expect(new Set(values).size).toBe(3);
        expect(values).toContain(String(Preferences_Scheduling_NewReviewMix.DISTRIBUTE));
        expect(values).toContain(String(Preferences_Scheduling_NewReviewMix.REVIEWS_FIRST));
        expect(values).toContain(String(Preferences_Scheduling_NewReviewMix.NEW_FIRST));
    });

    test("maps stored value to visible label", () => {
        expect(
            newReviewMixLabel(String(Preferences_Scheduling_NewReviewMix.NEW_FIRST)),
        ).toBe("Show new cards first");
    });
});
