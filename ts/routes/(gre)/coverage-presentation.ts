// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardCoverage } from "@generated/anki/brainlift_pb";

/** Plain-language explanation of weighted GRE topic coverage (shared across UI). */
export const COVERAGE_EXPLANATION =
    "A topic counts as covered once you've reviewed its flashcards at least once—practice alone doesn't count. "
    + "The weighted % reflects exam section weights (Quant and Verbal 47% each, AWA 6%)—"
    + "not a simple average of the three section numbers.";

export type CoverageSectionPresentation = {
    label: string;
    percent: number;
    coveredLeafCount: number;
    catalogLeafCount: number;
};

export type CoverageSummaryPresentation = {
    totalPercent: number;
    unweightedPercent: number;
    sections: CoverageSectionPresentation[];
    readinessAvailable: boolean;
    readinessReason: string | null;
    recommendations: string[];
    thresholdPercent: number;
};

export function coverageBlocksReadiness(coverage: DashboardCoverage): boolean {
    return !coverage.readinessAvailable;
}

export function coverageReadinessReason(coverage: DashboardCoverage): string {
    const pct = Math.round(coverage.weightedRatio * 100);
    return `Only ${pct}% of the GRE has evidence.`;
}

export function presentCoverageSummary(coverage: DashboardCoverage): CoverageSummaryPresentation {
    const recommendations = coverage.uncoveredTopics
        .map((topic) => topic.studyLabel)
        .filter((label) => label.length > 0);

    return {
        totalPercent: Math.round(coverage.weightedRatio * 100),
        unweightedPercent: Math.round(coverage.unweightedRatio * 100),
        sections: coverage.sections.map((section) => ({
            label: section.label,
            percent: Math.round(section.coveredExamWeight * 100),
            coveredLeafCount: section.coveredLeafCount,
            catalogLeafCount: section.catalogLeafCount,
        })),
        readinessAvailable: coverage.readinessAvailable,
        readinessReason: coverage.readinessAvailable ? null : coverageReadinessReason(coverage),
        recommendations,
        thresholdPercent: Math.round(coverage.coverageThreshold * 100),
    };
}

export function coverageAwareReadinessUnlocked(
    readiness: { sufficientData: boolean; projectedScore?: number },
    coverage?: DashboardCoverage | null,
): boolean {
    if (coverage && coverageBlocksReadiness(coverage)) {
        return false;
    }
    return readiness.sufficientData && readiness.projectedScore !== undefined;
}
