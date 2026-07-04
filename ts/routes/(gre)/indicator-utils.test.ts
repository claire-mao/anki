// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceAttempt } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    accuracyHorizonLabel,
    filterAttemptsByHorizon,
    rollingAccuracySeries,
    rollingAccuracyTrendPoints,
} from "./indicator-utils";

const NOW_SECS = 1_700_000_000;

function attempt(answeredAtSecs: number, correct: boolean): PerformanceAttempt {
    return {
        questionId: `q-${answeredAtSecs}`,
        topic: "gre::quant::arithmetic",
        answeredAtSecs: BigInt(answeredAtSecs),
        answer: "A",
        correct,
        responseTimeMs: 1200,
    } as PerformanceAttempt;
}

describe("accuracy trend horizons", () => {
    test("filters attempts to the selected day window", () => {
        const attempts = [
            attempt(NOW_SECS - 12 * 3_600, true),
            attempt(NOW_SECS - 2 * 86_400, true),
            attempt(NOW_SECS - 10 * 86_400, false),
            attempt(NOW_SECS - 40 * 86_400, true),
        ];

        expect(filterAttemptsByHorizon(attempts, "1d", NOW_SECS)).toHaveLength(1);
        expect(filterAttemptsByHorizon(attempts, "3d", NOW_SECS)).toHaveLength(2);
        expect(filterAttemptsByHorizon(attempts, "7d", NOW_SECS)).toHaveLength(2);
        expect(filterAttemptsByHorizon(attempts, "30d", NOW_SECS)).toHaveLength(3);
        expect(filterAttemptsByHorizon(attempts, "all", NOW_SECS)).toHaveLength(4);
    });

    test("builds rolling accuracy points with timestamps", () => {
        const attempts = [
            attempt(NOW_SECS - 86_400, true),
            attempt(NOW_SECS - 2 * 86_400, true),
            attempt(NOW_SECS - 3 * 86_400, false),
        ];
        const points = rollingAccuracyTrendPoints(attempts, 2);

        expect(points).toEqual([
            { answeredAtSecs: NOW_SECS - 3 * 86_400, accuracy: 0 },
            { answeredAtSecs: NOW_SECS - 2 * 86_400, accuracy: 50 },
            { answeredAtSecs: NOW_SECS - 86_400, accuracy: 100 },
        ]);
        expect(rollingAccuracySeries(attempts, 2)).toEqual([0, 50, 100]);
    });

    test("labels horizons for the performance footer", () => {
        expect(accuracyHorizonLabel("1d")).toBe("Last 1 day accuracy");
        expect(accuracyHorizonLabel("3d")).toBe("Last 3 days accuracy");
        expect(accuracyHorizonLabel("7d")).toBe("Last 7 days accuracy");
        expect(accuracyHorizonLabel("30d")).toBe("Last 30 days accuracy");
        expect(accuracyHorizonLabel("all")).toBe("All-time accuracy");
    });
});
