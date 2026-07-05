// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { ReadinessCalibrationStats, ReadinessScore } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    calibrationQualityVisible,
    formatCalibrationChecksProgress,
    presentCalibration,
} from "./calibration-presentation";

const emptyStats = {
    sufficientData: false,
    totalPredictions: 0,
    heldOutCount: 0,
    resolvedOutcomes: 0,
    calibrationCurve: [],
} as unknown as ReadinessCalibrationStats;

const lockedReadiness = {
    sufficientData: false,
    abstainReason: "8/50 cards studied · 40% coverage",
    abstentionRequirements: [
        {
            id: "studied_cards",
            label: "Studied cards",
            status: "8 / 50 cards",
            nextStep: "Review 42 more cards",
            met: false,
        },
    ],
} as unknown as ReadinessScore;

describe("presentCalibration", () => {
    test("early state uses a single summary instead of the full panel", () => {
        const model = presentCalibration(lockedReadiness, emptyStats);

        expect(model.showFullPanel).toBe(false);
        expect(model.earlyStateSummary).toMatch(/0 of 5 checks completed/);
        expect(model.confidenceCaption).toBe("Provisional until readiness gates clear");
        expect(model.confidenceCaption).not.toContain("8/50");
        expect(model.historicalAccuracy).toBe("None yet");
        expect(model.historicalAccuracyDetail).toMatch(/predictions start/);
        expect(model.predictionQuality).toBe("0 of 5 checks completed");
        expect(model.predictionQualityDetail).toBe("");
        expect(model.trendAvailable).toBe(false);
        expect(model.trendCaption).toMatch(/score levels/);
    });

    test("surfaces gate and check guidance once in confidence notes", () => {
        const model = presentCalibration(lockedReadiness, emptyStats);

        expect(model.confidenceChangeNotes).toContain("Review 42 more cards");
        expect(model.confidenceChangeNotes.some((note) => note.includes("check"))).toBe(
            true,
        );
        expect(
            model.confidenceChangeNotes.filter((note) => note.includes("check")),
        ).toHaveLength(1);
    });

    test("shows measured accuracy when history exists", () => {
        const model = presentCalibration(
            {
                sufficientData: true,
                confidenceLevel: "medium",
                abstentionRequirements: [],
            } as unknown as ReadinessScore,
            {
                sufficientData: true,
                wellCalibrated: true,
                meanAbsoluteError: 4.2,
                totalPredictions: 12,
                resolvedOutcomes: 10,
                heldOutCount: 6,
                brierScore: 0.08,
                calibrationCurve: [],
            } as unknown as ReadinessCalibrationStats,
        );

        expect(model.showFullPanel).toBe(true);
        expect(model.historicalAccuracy).toBe("4.2 pt error");
        expect(model.predictionQuality).toBe("Good fit");
        expect(model.predictionQualityDetail).toContain("4.2 pt average error");
        expect(model.predictionQualityDetail).not.toContain("Brier");
    });
});

describe("formatCalibrationChecksProgress", () => {
    test("formats check progress against the verification threshold", () => {
        expect(formatCalibrationChecksProgress({ heldOutCount: 2 } as unknown as ReadinessCalibrationStats)).toBe(
            "2 of 5 checks completed",
        );
    });
});

describe("calibrationQualityVisible", () => {
    test("hides track record until sufficient checks exist", () => {
        expect(calibrationQualityVisible(emptyStats)).toBe(false);
        expect(
            calibrationQualityVisible({ sufficientData: true } as unknown as ReadinessCalibrationStats),
        ).toBe(true);
    });
});
