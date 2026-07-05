// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardCoverage, EstimatedGreScore, ReadinessScore } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    dashboardHeroEstimatedGre,
    dashboardHeroEstimatedGreAvailable,
    dashboardHeroEstimatedGreHint,
    dashboardHeroMetricsUnlocked,
    estimatedGreHero,
} from "./summary-metrics";

const coverage = (readinessAvailable: boolean): DashboardCoverage => ({ readinessAvailable }) as DashboardCoverage;

describe("dashboard hero metrics", () => {
    test("shows estimated GRE only when readiness is unlocked", () => {
        const estimate = { combinedScore: 340 } as EstimatedGreScore;
        const readiness = {
            sufficientData: false,
            projectedScore: undefined,
        } as ReadinessScore;

        expect(estimatedGreHero(estimate)).toBe("340");
        expect(dashboardHeroMetricsUnlocked(readiness, coverage(true))).toBe(false);
        expect(dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage(true))).toBe(false);
        expect(dashboardHeroEstimatedGre(estimate, readiness, coverage(true))).toBe("—");
        expect(dashboardHeroEstimatedGreHint(estimate, readiness, coverage(true))).not.toContain("260–340");
    });

    test("shows both metrics when readiness unlocks", () => {
        const estimate = { combinedScore: 318 } as EstimatedGreScore;
        const readiness = {
            sufficientData: true,
            projectedScore: 72,
        } as ReadinessScore;

        expect(dashboardHeroMetricsUnlocked(readiness, coverage(true))).toBe(true);
        expect(dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage(true))).toBe(true);
        expect(dashboardHeroEstimatedGre(estimate, readiness, coverage(true))).toBe("318");
        expect(dashboardHeroEstimatedGreHint(estimate, readiness, coverage(true))).toContain("260–340");
    });

    test("blocks metrics when coverage is below threshold", () => {
        const estimate = { combinedScore: 330 } as EstimatedGreScore;
        const readiness = {
            sufficientData: true,
            projectedScore: 85,
        } as ReadinessScore;

        expect(dashboardHeroMetricsUnlocked(readiness, coverage(false))).toBe(false);
        expect(dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage(false))).toBe(false);
        expect(dashboardHeroEstimatedGre(estimate, readiness, coverage(false))).toBe("—");
        expect(dashboardHeroEstimatedGreHint(estimate, readiness, coverage(false))).not.toContain("260–340");
    });

    test("blocks projected score copy when estimate abstains despite unlocked readiness", () => {
        const estimate = {
            combinedScore: undefined,
            abstainReason: "Need more practice attempts.",
        } as EstimatedGreScore;
        const readiness = {
            sufficientData: true,
            projectedScore: 72,
        } as ReadinessScore;

        expect(dashboardHeroMetricsUnlocked(readiness, coverage(true))).toBe(true);
        expect(dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage(true))).toBe(false);
        expect(dashboardHeroEstimatedGre(estimate, readiness, coverage(true))).toBe("—");
        expect(dashboardHeroEstimatedGreHint(estimate, readiness, coverage(true))).toBe(
            "Need more practice attempts.",
        );
    });
});
