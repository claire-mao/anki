// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import type {
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import {
    compositeReadinessScore,
    presentReadinessModelExplanation,
    READINESS_COVERAGE_WEIGHT,
    READINESS_MEMORY_WEIGHT,
    READINESS_PERFORMANCE_WEIGHT,
    readinessProjectionInterval,
} from "./readiness-model-presentation";

describe("compositeReadinessScore", () => {
    test("matches documented weights from calibration.rs", () => {
        const score = compositeReadinessScore(80, 60, 0.5);
        const expected =
            (READINESS_MEMORY_WEIGHT * 0.8
                + READINESS_PERFORMANCE_WEIGHT * 0.6
                + READINESS_COVERAGE_WEIGHT * 0.5)
            * 100;
        expect(score).toBeCloseTo(expected, 5);
    });
});

describe("readinessProjectionInterval", () => {
    test("combines memory and performance interval widths in quadrature", () => {
        const memory: MemoryScore = {
            value: 80,
            valueLow: 75,
            valueHigh: 85,
            sufficientData: true,
            abstentionRequirements: [],
        };
        const performance: PerformanceScore = {
            value: 60,
            valueLow: 50,
            valueHigh: 70,
            sufficientData: true,
            abstentionRequirements: [],
        };
        const projected = compositeReadinessScore(80, 60, 0.6);
        const interval = readinessProjectionInterval(memory, performance, projected);
        expect(interval.low).toBeLessThanOrEqual(projected);
        expect(interval.high).toBeGreaterThanOrEqual(projected);
        expect(interval.low).toBeGreaterThanOrEqual(0);
        expect(interval.high).toBeLessThanOrEqual(100);
    });
});

describe("presentReadinessModelExplanation", () => {
    test("derives step details from backend fields, not hardcoded prose", () => {
        const memory: MemoryScore = {
            value: 80,
            valueLow: 75,
            valueHigh: 85,
            sufficientData: true,
            studiedCards: 250,
            coverageRatio: 0.6,
            detail: "250 studied cards · 60% catalog coverage (12 leaf topics with data)",
            abstentionRequirements: [
                {
                    id: "topic_coverage",
                    label: "Topic coverage",
                    status: "60% exam-weighted catalog covered (minimum 50%)",
                    met: true,
                },
            ],
        };
        const performance: PerformanceScore = {
            value: 80,
            valueLow: 70,
            valueHigh: 88,
            sufficientData: true,
            attemptCount: 50,
            detail: "40/50 practice questions correct (80%)",
            abstentionRequirements: [],
        };
        const readiness: ReadinessScore = {
            projectedScore: 78,
            projectedScoreLow: 72,
            projectedScoreHigh: 84,
            confidenceLevel: "medium",
            coverageRatio: 0.6,
            sufficientData: true,
            evidenceSummary:
                "Memory 80% (250 studied cards, 60% catalog coverage) · Performance 80% (50 attempts)",
            calibrationNote:
                "Held-out calibration on 6 predictions: Brier score 0.05. Predictions track later outcomes reasonably well.",
            calibrationBrierScore: 0.05,
            calibrationSufficientData: true,
            calibrationWellCalibrated: true,
            abstentionRequirements: [],
        };

        const model = presentReadinessModelExplanation({
            memory,
            performance,
            readiness,
        });

        expect(model.score).toBe("78%");
        expect(model.predictionInterval).toBe("72%–84%");
        expect(model.confidence).toBe("Medium");
        expect(model.evidenceUsed[0]?.detail).toBe(readiness.evidenceSummary);

        const memoryStep = model.steps.find((step) => step.id === "memory");
        const performanceStep = model.steps.find((step) => step.id === "performance");
        const coverageStep = model.steps.find((step) => step.id === "coverage");
        const calibrationStep = model.steps.find((step) => step.id === "calibration");
        const readinessStep = model.steps.find((step) => step.id === "readiness");

        expect(memoryStep?.detail).toBe(memory.detail);
        expect(performanceStep?.detail).toBe(performance.detail);
        expect(coverageStep?.detail).toBe("60% exam-weighted catalog covered (minimum 50%)");
        expect(calibrationStep?.detail).toBe(readiness.calibrationNote);
        expect(readinessStep?.detail).toContain(readiness.evidenceSummary);
        expect(readinessStep?.detail).toContain("36% memory");
        expect(model.steps.map((step) => step.label)).toEqual([
            "Memory",
            "Performance",
            "Coverage",
            "Calibration",
            "Readiness",
        ]);
    });

    test("uses calibration assessment when readiness note is empty", () => {
        const calibration: ReadinessCalibrationStats = {
            sufficientData: false,
            wellCalibrated: false,
            assessment:
                "Calibration requires at least 5 held-out predictions with observed outcomes (current: 1). Model confidence is unverified.",
        };
        const model = presentReadinessModelExplanation({
            memory: { abstentionRequirements: [] },
            performance: { abstentionRequirements: [] },
            readiness: {
                sufficientData: false,
                coverageRatio: 0.2,
                abstentionRequirements: [],
            },
            calibration,
        });

        expect(model.score).toBeNull();
        expect(model.predictionInterval).toBe("—");
        expect(model.steps.find((step) => step.id === "calibration")?.detail).toBe(
            calibration.assessment,
        );
    });
});
