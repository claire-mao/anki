// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardCoverage } from "@generated/anki/brainlift_pb";

/** Plain-language explanation of GRE topic coverage (shared across UI). */
export const COVERAGE_EXPLANATION =
    "A topic counts as covered after you review its flashcards at least once. Practice alone does not count.";

/** Official GRE section shares used in weighted readiness coverage. */
export const OFFICIAL_SECTION_WEIGHT_PERCENT: Readonly<Record<string, number>> = {
    quant: 47,
    verbal: 47,
    awa: 6,
};

export const COVERAGE_TOPIC_PERCENT_LABEL = "Topics reviewed";
export const COVERAGE_WEIGHTED_PERCENT_LABEL = "Readiness coverage";

export const COVERAGE_WEIGHTED_FORMULA_INTRO =
    "Readiness coverage weights each GRE section by its official exam share (Quant 47%, Verbal 47%, AWA 6%). Multiply that weight by the share of section topics you have reviewed, then add the three contributions.";

export const COVERAGE_BREAKDOWN_HEADERS = [
    "Section",
    "Coverage",
    "Weight",
    "Contribution",
] as const;

export type CoverageSummaryVariant = "dashboard" | "info";

export type CoverageBreakdownRow = {
    sectionSlug: string;
    label: string;
    /** Rounded percent of section topics reviewed (display only). */
    coveragePercent: number;
    /** Exact reviewed/total fraction for this section (0–1). */
    coverageRatio: number;
    /** Official GRE section weight (47, 47, or 6). */
    weightPercent: number;
    /** Unrounded contribution in percentage points (weight × coverageRatio). */
    contributionPoints: number;
};

export type CoverageSectionPresentation = {
    sectionSlug: string;
    label: string;
    /** Share of catalog topics in this section with at least one flashcard review. */
    topicPercent: number;
    coveredLeafCount: number;
    catalogLeafCount: number;
    /** Exact reviewed/total fraction for this section (0–1). */
    coverageRatio: number;
    /** Unrounded contribution in percentage points toward readiness coverage. */
    contributionPoints: number;
};

export type CoverageSummaryPresentation = {
    /** Weighted coverage used for readiness (official section + topic counts). */
    totalPercent: number;
    /** Share of catalog topics with at least one flashcard review. */
    topicPercent: number;
    coveredLeafCount: number;
    catalogLeafCount: number;
    sections: CoverageSectionPresentation[];
    breakdown: CoverageBreakdownRow[];
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
    return `Only ${pct}% readiness coverage. Review flashcards in more topics to unlock readiness.`;
}

export function sectionCoverageRatio(coveredLeafCount: number, catalogLeafCount: number): number {
    if (catalogLeafCount <= 0) {
        return 0;
    }
    return coveredLeafCount / catalogLeafCount;
}

export function roundCoveragePercent(ratio: number): number {
    return Math.round(ratio * 100);
}

function sectionTopicPercent(coveredLeafCount: number, catalogLeafCount: number): number {
    return roundCoveragePercent(sectionCoverageRatio(coveredLeafCount, catalogLeafCount));
}

export function weightedContributionPoints(
    sectionSlug: string,
    coverageRatio: number,
): number {
    const sectionShare = OFFICIAL_SECTION_WEIGHT_PERCENT[sectionSlug] ?? 0;
    return sectionShare * coverageRatio;
}

/** Mirrors Rust `compute_coverage` weighted sum using exact topic fractions. */
export function computeWeightedCoveragePercent(
    sections: ReadonlyArray<{
        sectionSlug: string;
        coveredLeafCount: number;
        catalogLeafCount: number;
    }>,
): number {
    const weightedRatio = sections.reduce((sum, section) => {
        const ratio = sectionCoverageRatio(section.coveredLeafCount, section.catalogLeafCount);
        return sum + weightedContributionPoints(section.sectionSlug, ratio) / 100;
    }, 0);
    return roundCoveragePercent(weightedRatio);
}

export function formatContributionPoints(points: number): string {
    return `${points.toFixed(1)}%`;
}

export function formatWeightPercent(weightPercent: number): string {
    return `${weightPercent}%`;
}

export function formatBreakdownContributionFormula(row: CoverageBreakdownRow): string {
    return `${formatWeightPercent(row.weightPercent)} × ${row.coveragePercent}% = ${formatContributionPoints(row.contributionPoints)}`;
}

export function coverageShowsBreakdown(variant: CoverageSummaryVariant, compact: boolean): boolean {
    return variant === "dashboard" && !compact;
}

export function presentCoverageSummary(coverage: DashboardCoverage): CoverageSummaryPresentation {
    const recommendations = coverage.uncoveredTopics
        .map((topic) => topic.studyLabel)
        .filter((label) => label.length > 0);

    const sections = coverage.sections.map((section) => {
        const coverageRatio = sectionCoverageRatio(
            section.coveredLeafCount,
            section.catalogLeafCount,
        );
        return {
            sectionSlug: section.section,
            label: section.label,
            topicPercent: sectionTopicPercent(section.coveredLeafCount, section.catalogLeafCount),
            coveredLeafCount: section.coveredLeafCount,
            catalogLeafCount: section.catalogLeafCount,
            coverageRatio,
            contributionPoints: weightedContributionPoints(section.section, coverageRatio),
        };
    });

    const breakdown: CoverageBreakdownRow[] = sections.map((section) => ({
        sectionSlug: section.sectionSlug,
        label: section.label,
        coveragePercent: section.topicPercent,
        coverageRatio: section.coverageRatio,
        weightPercent: OFFICIAL_SECTION_WEIGHT_PERCENT[section.sectionSlug] ?? 0,
        contributionPoints: section.contributionPoints,
    }));

    const totalPercent = roundCoveragePercent(coverage.weightedRatio);

    return {
        totalPercent,
        topicPercent: sectionTopicPercent(coverage.coveredLeafCount, coverage.catalogLeafCount),
        coveredLeafCount: coverage.coveredLeafCount,
        catalogLeafCount: coverage.catalogLeafCount,
        sections,
        breakdown,
        readinessAvailable: coverage.readinessAvailable,
        readinessReason: coverage.readinessAvailable ? null : coverageReadinessReason(coverage),
        recommendations,
        thresholdPercent: roundCoveragePercent(coverage.coverageThreshold),
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

function formatTopicCount(covered: number, total: number): string {
    const noun = total === 1 ? "topic" : "topics";
    return `${covered} of ${total} ${noun}`;
}

export function formatCoverageTopicCount(covered: number, total: number): string {
    return formatTopicCount(covered, total);
}

export function formatSectionTopicPercent(topicPercent: number): string {
    return `${topicPercent}% of section topics`;
}
