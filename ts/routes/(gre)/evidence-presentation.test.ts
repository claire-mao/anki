// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    DashboardCoverage,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
} from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import { buildEvidenceItems } from "./evidence-presentation";

const memory = (studiedCards: number) => ({ studiedCards }) as MemoryScore;
const performance = (attemptCount: number) => ({ attemptCount }) as PerformanceScore;
const coverage = (weightedRatio: number) => ({ weightedRatio }) as DashboardCoverage;

describe("evidence items", () => {
    test("summarizes the concrete evidence behind the estimate", () => {
        const items = buildEvidenceItems(memory(842), performance(124), coverage(0.61));
        expect(items[0].label).toBe("842 flashcards reviewed");
        expect(items[1].label).toBe("124 practice questions");
        expect(items[2].label).toMatch(/GRE topic coverage$/);
        expect(items.every((item) => item.met)).toBe(true);
    });

    test("marks not-yet-collected evidence as not met", () => {
        const items = buildEvidenceItems(memory(0), performance(0), coverage(0));
        expect(items.every((item) => !item.met)).toBe(true);
    });

    test("only shows calibration error with sufficient held-out data (honest)", () => {
        const withCalibration = buildEvidenceItems(
            memory(842),
            performance(124),
            coverage(0.61),
            {
                sufficientData: true,
                meanAbsoluteError: 4.8,
            } as ReadinessCalibrationStats,
        );
        expect(withCalibration.at(-1)?.label).toBe("Past estimate error: ±4.8 points");

        const insufficient = buildEvidenceItems(
            memory(842),
            performance(124),
            coverage(0.61),
            {
                sufficientData: false,
                meanAbsoluteError: 4.8,
            } as ReadinessCalibrationStats,
        );
        expect(insufficient).toHaveLength(3);
    });
});
