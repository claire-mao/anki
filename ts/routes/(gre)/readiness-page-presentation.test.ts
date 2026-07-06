// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import type {
    DashboardCoverage,
    EstimatedGreScore,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import { presentReadinessPage } from "./readiness-page-presentation";

describe("presentReadinessPage coverage", () => {
    test("does not expose a broken structured coverage metric", () => {
        const model = presentReadinessPage(sampleInput());
        expect("coverage" in model).toBe(false);
    });
});

function sampleInput() {
    const coverage: DashboardCoverage = {
        weightedRatio: 0.4,
        unweightedRatio: 8 / 21,
        catalogLeafCount: 21,
        coveredLeafCount: 8,
        sections: [
            {
                section: "quant",
                label: "Quant",
                coveredExamWeight: 0.32,
                catalogLeafCount: 11,
                coveredLeafCount: 4,
            },
            {
                section: "verbal",
                label: "Verbal",
                coveredExamWeight: 0.38,
                catalogLeafCount: 8,
                coveredLeafCount: 3,
            },
            {
                section: "awa",
                label: "AWA",
                coveredExamWeight: 0.5,
                catalogLeafCount: 2,
                coveredLeafCount: 1,
            },
        ],
        uncoveredTopics: [],
        coverageThreshold: 0.5,
        readinessAvailable: false,
    };

    const readiness: ReadinessScore = {
        sufficientData: false,
        abstainReason: "Need more coverage",
        abstentionRequirements: [],
        lastUpdatedMillis: 1_700_000_000_000n,
    };

    const memory: MemoryScore = {
        studiedCards: 10,
        coverageRatio: 0.2,
        abstentionRequirements: [],
    };

    const performance: PerformanceScore = {
        attemptCount: 5,
        accuracy: 0.7,
        abstentionRequirements: [],
    };

    const estimatedGre: EstimatedGreScore = {};

    const calibration: ReadinessCalibrationStats = {};

    return {
        readiness,
        calibration,
        memory,
        performance,
        estimatedGre,
        coverage,
        weakTopics: [],
        computedAtMillis: 1_700_000_000_000n,
    };
}
