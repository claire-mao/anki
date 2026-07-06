// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import {
    COVERAGE_EXPLANATION,
    COVERAGE_WEIGHTED_FORMULA_INTRO,
    computeWeightedCoveragePercent,
    coverageShowsBreakdown,
    formatBreakdownContributionFormula,
    presentCoverageSummary,
    weightedContributionPoints,
} from "./coverage-presentation";
import type { DashboardCoverage } from "@generated/anki/brainlift_pb";

describe("coverage explanation", () => {
    test("describes what counts as covered", () => {
        expect(COVERAGE_EXPLANATION).toMatch(/review its flashcards/i);
        expect(COVERAGE_EXPLANATION).toMatch(/Practice alone does not count/i);
        expect(COVERAGE_EXPLANATION).not.toMatch(/—/);
    });
});

describe("weighted coverage", () => {
    test("multiplies official section share by exact topic fraction", () => {
        expect(weightedContributionPoints("quant", 4 / 11)).toBeCloseTo(17.0909, 3);
        expect(weightedContributionPoints("verbal", 3 / 8)).toBeCloseTo(17.625, 3);
        expect(weightedContributionPoints("awa", 1 / 2)).toBe(3);
    });

    test("frontend weighted sum matches backend ratio for sample dashboard", () => {
        const coverage = sampleCoverage();
        const computed = computeWeightedCoveragePercent(coverage.sections.map((section) => ({
            sectionSlug: section.section,
            coveredLeafCount: section.coveredLeafCount,
            catalogLeafCount: section.catalogLeafCount,
        })));
        expect(computed).toBe(Math.round(coverage.weightedRatio * 100));
    });

    test("presents breakdown rows that sum to readiness coverage", () => {
        const summary = presentCoverageSummary(sampleCoverage());
        expect(summary.totalPercent).toBe(38);
        expect(summary.topicPercent).toBe(38);
        expect(summary.breakdown).toEqual([
            {
                sectionSlug: "quant",
                label: "Quant",
                coveragePercent: 36,
                coverageRatio: 4 / 11,
                weightPercent: 47,
                contributionPoints: weightedContributionPoints("quant", 4 / 11),
            },
            {
                sectionSlug: "verbal",
                label: "Verbal",
                coveragePercent: 38,
                coverageRatio: 3 / 8,
                weightPercent: 47,
                contributionPoints: weightedContributionPoints("verbal", 3 / 8),
            },
            {
                sectionSlug: "awa",
                label: "AWA",
                coveragePercent: 50,
                coverageRatio: 0.5,
                weightPercent: 6,
                contributionPoints: 3,
            },
        ]);
        const contributionSum = summary.breakdown.reduce(
            (sum, row) => sum + row.contributionPoints,
            0,
        );
        expect(Math.round(contributionSum)).toBe(summary.totalPercent);
    });

    test("keeps section topic percent on presentation model", () => {
        const summary = presentCoverageSummary(sampleCoverage());
        expect(summary.sections[0]).toMatchObject({
            label: "Quant",
            topicPercent: 36,
            contributionPoints: weightedContributionPoints("quant", 4 / 11),
        });
    });

    test("dashboard variant shows breakdown helpers", () => {
        const summary = presentCoverageSummary(sampleCoverage());
        expect(coverageShowsBreakdown("dashboard", false)).toBe(true);
        expect(coverageShowsBreakdown("info", false)).toBe(false);
        expect(coverageShowsBreakdown("dashboard", true)).toBe(false);
        expect(COVERAGE_WEIGHTED_FORMULA_INTRO).toMatch(/47%/);
        expect(formatBreakdownContributionFormula(summary.breakdown[0])).toBe(
            `47% × 36% = ${weightedContributionPoints("quant", 4 / 11).toFixed(1)}%`,
        );
    });

    test("info variant stays percentage-only in presentation model", () => {
        const summary = presentCoverageSummary(sampleCoverage());
        expect(summary.sections.map((section) => section.topicPercent)).toEqual([36, 38, 50]);
        expect(summary.totalPercent).toBe(38);
        expect(coverageShowsBreakdown("info", false)).toBe(false);
    });

    test("matches several known combinations exactly", () => {
        expect(computeWeightedCoveragePercent([
            { sectionSlug: "quant", coveredLeafCount: 0, catalogLeafCount: 11 },
            { sectionSlug: "verbal", coveredLeafCount: 0, catalogLeafCount: 8 },
            { sectionSlug: "awa", coveredLeafCount: 0, catalogLeafCount: 2 },
        ])).toBe(0);

        expect(computeWeightedCoveragePercent([
            { sectionSlug: "quant", coveredLeafCount: 11, catalogLeafCount: 11 },
            { sectionSlug: "verbal", coveredLeafCount: 8, catalogLeafCount: 8 },
            { sectionSlug: "awa", coveredLeafCount: 2, catalogLeafCount: 2 },
        ])).toBe(100);

        expect(computeWeightedCoveragePercent([
            { sectionSlug: "quant", coveredLeafCount: 1, catalogLeafCount: 11 },
            { sectionSlug: "verbal", coveredLeafCount: 1, catalogLeafCount: 8 },
            { sectionSlug: "awa", coveredLeafCount: 1, catalogLeafCount: 2 },
        ])).toBe(Math.round((0.47 / 11 + 0.47 / 8 + 0.06 / 2) * 100));
    });
});

function sampleCoverage(): DashboardCoverage {
    const weightedRatio =
        0.47 * (4 / 11) + 0.47 * (3 / 8) + 0.06 * (1 / 2);
    return {
        weightedRatio,
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
}
