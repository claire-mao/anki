// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import type {
    DashboardCoverage,
    MemoryScore,
    PerformanceAttempt,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import {
    countAwaPracticeAttempts,
    presentHonestReporting,
} from "./honest-reporting-presentation";

function baseCoverage(overrides: Partial<DashboardCoverage> = {}): DashboardCoverage {
    return {
        weightedRatio: 0.4,
        unweightedRatio: 0.2,
        catalogLeafCount: 21,
        coveredLeafCount: 4,
        sections: [
            {
                section: "quant",
                label: "Quant",
                coveredExamWeight: 0.2,
                catalogLeafCount: 11,
                coveredLeafCount: 2,
            },
            {
                section: "verbal",
                label: "Verbal",
                coveredExamWeight: 0.2,
                catalogLeafCount: 8,
                coveredLeafCount: 2,
            },
            {
                section: "awa",
                label: "AWA",
                coveredExamWeight: 0,
                catalogLeafCount: 2,
                coveredLeafCount: 0,
            },
        ],
        uncoveredTopics: [],
        coverageThreshold: 0.5,
        readinessAvailable: false,
        ...overrides,
    };
}

function studiedCardsRequirement(studiedCards: number, met: boolean) {
    return {
        id: "studied_cards",
        label: "Studied GRE cards",
        status: `${studiedCards} of 50 studied cards`,
        nextStep: "Review more GRE flashcards.",
        met,
    };
}

function topicCoverageRequirement(coverageRatio: number, met: boolean) {
    const pct = Math.round(coverageRatio * 100);
    return {
        id: "topic_coverage",
        label: "Topic coverage",
        status: met
            ? `${pct}% exam-weighted catalog covered`
            : `Only ${pct}% of the GRE has evidence.`,
        nextStep: "Review cards across more GRE topics.",
        met,
    };
}

describe("presentHonestReporting", () => {
    it("reports abstention and coverage limitations without exposing readiness score", () => {
        const model = presentHonestReporting({
            memory: {
                studiedCards: 12,
                coverageRatio: 0.2,
                abstentionRequirements: [
                    studiedCardsRequirement(12, false),
                    topicCoverageRequirement(0.2, false),
                ],
            },
            performance: {
                attemptCount: 4,
                abstentionRequirements: [
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "4 of 50 practice attempts",
                        nextStep: "Answer more practice questions.",
                        met: false,
                    },
                ],
            },
            readiness: {
                sufficientData: false,
                abstainReason: "Topic coverage: Only 20% of the GRE has evidence.",
                abstentionRequirements: [
                    studiedCardsRequirement(12, false),
                    topicCoverageRequirement(0.2, false),
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "4 of 50 practice attempts",
                        nextStep: "Answer more practice questions.",
                        met: false,
                    },
                ],
            },
            coverage: baseCoverage(),
            calibration: { sufficientData: false, heldOutCount: 1 },
            aiEnabled: "Disabled",
        });

        expect(model.readinessScoreVisible).toBe(false);
        expect(model.limitations.map((item) => item.id)).toEqual([
            "readiness_abstained",
            "low_topic_coverage",
            "too_few_reviews",
            "insufficient_calibration",
            "sparse_awa_data",
            "ai_disabled",
        ]);
    });

    it("reports low confidence only when readiness is unlocked", () => {
        const model = presentHonestReporting({
            memory: {
                studiedCards: 120,
                coverageRatio: 0.72,
                sufficientData: true,
                abstentionRequirements: [
                    studiedCardsRequirement(120, true),
                    topicCoverageRequirement(0.72, true),
                ],
            },
            performance: {
                attemptCount: 80,
                sufficientData: true,
                abstentionRequirements: [
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "80 practice attempts",
                        met: true,
                    },
                ],
            },
            readiness: {
                sufficientData: true,
                projectedScore: 68,
                projectedScoreLow: 60,
                projectedScoreHigh: 76,
                confidenceLevel: "low",
                abstentionRequirements: [
                    studiedCardsRequirement(120, true),
                    topicCoverageRequirement(0.72, true),
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "80 practice attempts",
                        met: true,
                    },
                ],
            },
            coverage: baseCoverage({
                weightedRatio: 0.72,
                readinessAvailable: true,
                coveredLeafCount: 15,
                sections: [
                    {
                        section: "quant",
                        label: "Quant",
                        coveredExamWeight: 0.8,
                        catalogLeafCount: 11,
                        coveredLeafCount: 9,
                    },
                    {
                        section: "verbal",
                        label: "Verbal",
                        coveredExamWeight: 0.7,
                        catalogLeafCount: 8,
                        coveredLeafCount: 6,
                    },
                    {
                        section: "awa",
                        label: "AWA",
                        coveredExamWeight: 1,
                        catalogLeafCount: 2,
                        coveredLeafCount: 2,
                    },
                ],
            }),
            calibration: { sufficientData: true, wellCalibrated: true, heldOutCount: 6 },
            recentAttempts: [{ topic: "gre::awa::argument" }, { topic: "gre::awa::issue" }, { topic: "gre::awa::issue" }] as PerformanceAttempt[],
        });

        expect(model.readinessScoreVisible).toBe(true);
        expect(model.limitations.map((item) => item.id)).toEqual(["low_confidence"]);
        expect(model.limitations[0]?.title).toBe("Low confidence");
    });

    it("returns all clear when evidence gates are satisfied", () => {
        const model = presentHonestReporting({
            memory: {
                studiedCards: 120,
                coverageRatio: 0.72,
                sufficientData: true,
                abstentionRequirements: [
                    studiedCardsRequirement(120, true),
                    topicCoverageRequirement(0.72, true),
                ],
            },
            performance: {
                attemptCount: 80,
                sufficientData: true,
                abstentionRequirements: [
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "80 practice attempts",
                        met: true,
                    },
                ],
            },
            readiness: {
                sufficientData: true,
                projectedScore: 72,
                projectedScoreLow: 69,
                projectedScoreHigh: 75,
                confidenceLevel: "high",
                abstentionRequirements: [
                    studiedCardsRequirement(120, true),
                    topicCoverageRequirement(0.72, true),
                    {
                        id: "practice_attempts",
                        label: "GRE practice attempts",
                        status: "80 practice attempts",
                        met: true,
                    },
                ],
            },
            coverage: baseCoverage({
                weightedRatio: 0.72,
                readinessAvailable: true,
                coveredLeafCount: 15,
                sections: [
                    {
                        section: "quant",
                        label: "Quant",
                        coveredExamWeight: 0.8,
                        catalogLeafCount: 11,
                        coveredLeafCount: 9,
                    },
                    {
                        section: "verbal",
                        label: "Verbal",
                        coveredExamWeight: 0.7,
                        catalogLeafCount: 8,
                        coveredLeafCount: 6,
                    },
                    {
                        section: "awa",
                        label: "AWA",
                        coveredExamWeight: 1,
                        catalogLeafCount: 2,
                        coveredLeafCount: 2,
                    },
                ],
            }),
            calibration: { sufficientData: true, wellCalibrated: true, heldOutCount: 6 },
            aiEnabled: "Enabled",
            recentAttempts: [
                { topic: "gre::awa::argument" },
                { topic: "gre::awa::issue" },
                { topic: "gre::awa::issue" },
            ] as PerformanceAttempt[],
        });

        expect(model.allClear).toBe(true);
        expect(model.limitations).toEqual([]);
    });
});

describe("countAwaPracticeAttempts", () => {
    it("counts only AWA topic attempts", () => {
        expect(
            countAwaPracticeAttempts([
                { topic: "gre::awa::argument" },
                { topic: "gre::quant::algebra" },
                { topic: "gre::awa::issue" },
            ] as PerformanceAttempt[]),
        ).toBe(2);
    });
});
