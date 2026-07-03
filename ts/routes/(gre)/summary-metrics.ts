// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    AbstentionRequirement,
    EstimatedGreScore,
    MemoryScore,
    PerformanceScore,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import { emptyStateTitle, sortRequirementsForProgress } from "./empty-states";

export function capitalizeLabel(value: string): string {
    if (!value) {
        return "—";
    }
    return value.charAt(0).toUpperCase() + value.slice(1);
}

export function unmetRequirements(
    ...groups: AbstentionRequirement[][]
): AbstentionRequirement[] {
    return groups.flat().filter((req) => !req.met);
}

export function fsrsStatus(requirements: AbstentionRequirement[]): string {
    const fsrs = requirements.find((req) => req.id === "fsrs_enabled");
    if (!fsrs) {
        return "—";
    }
    return fsrs.met ? "Enabled" : "Disabled";
}

export function estimatedGreConfidence(
    estimate: EstimatedGreScore,
    readiness?: ReadinessScore,
): string {
    if (estimate.preliminary) {
        return "Preliminary";
    }
    if (readiness?.confidenceLevel) {
        return capitalizeLabel(readiness.confidenceLevel);
    }
    if (estimate.sufficientData) {
        return "Medium";
    }
    return "—";
}

export function estimatedGreHero(estimate: EstimatedGreScore): string {
    if (estimate.combinedScore !== undefined) {
        return String(Math.round(estimate.combinedScore));
    }
    return "—";
}

export function memoryHero(memory: MemoryScore, formatPercent: (v: number) => string): string {
    if (memory.sufficientData && memory.value !== undefined) {
        return formatPercent(memory.value);
    }
    return "—";
}

export function performanceHero(
    performance: PerformanceScore,
    formatPercent: (v: number) => string,
): string {
    if (performance.sufficientData && performance.value !== undefined) {
        return formatPercent(performance.value);
    }
    return "—";
}

export function readinessHero(
    readiness: ReadinessScore,
    formatPercent: (v: number) => string,
): string {
    if (readiness.sufficientData && readiness.projectedScore !== undefined) {
        return formatPercent(readiness.projectedScore);
    }
    return "—";
}

export function checklistRequirementsForEstimatedGre(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
): AbstentionRequirement[] {
    return unmetRequirements(
        memory.abstentionRequirements,
        performance.abstentionRequirements,
        readiness.abstentionRequirements,
    );
}

export function milestoneRequirementsForEstimatedGre(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
): AbstentionRequirement[] {
    const byId = new Map<string, AbstentionRequirement>();
    for (
        const req of [
            ...memory.abstentionRequirements,
            ...performance.abstentionRequirements,
            ...readiness.abstentionRequirements,
        ]
    ) {
        byId.set(req.id, req);
    }
    return sortRequirementsForProgress([...byId.values()]);
}

export function memoryChartContext(
    memory: MemoryScore,
    formatRatio: (v: number) => string,
): string {
    if (!memory.sufficientData) {
        return `${memory.studiedCards} cards studied · ${formatRatio(memory.coverageRatio)} coverage`;
    }
    return `${memory.studiedCards} cards · ${formatRatio(memory.coverageRatio)} coverage · FSRS ${
        fsrsStatus(memory.abstentionRequirements)
    }`;
}

export function performanceChartContext(performance: PerformanceScore): string {
    return `${performance.attemptCount} practice attempts recorded`;
}

export function readinessChartContext(
    readiness: ReadinessScore,
    formatRatio: (v: number) => string,
): string {
    if (readiness.evidenceSummary) {
        return readiness.evidenceSummary;
    }
    if (!readiness.sufficientData || readiness.projectedScore === undefined) {
        return readiness.abstainReason || emptyStateTitle("readiness");
    }
    const parts = [`${formatRatio(readiness.coverageRatio)} topic coverage`];
    if (readiness.confidenceLevel) {
        parts.push(`${capitalizeLabel(readiness.confidenceLevel)} confidence`);
    }
    return parts.join(" · ");
}

export function estimatedGreChartContext(
    estimate: EstimatedGreScore,
    readiness?: ReadinessScore,
): string {
    if (estimate.detail) {
        return estimate.detail;
    }
    if (estimate.combinedScore === undefined) {
        return estimate.abstainReason || emptyStateTitle("estimatedGreChart");
    }
    return `Confidence ${estimatedGreConfidence(estimate, readiness)}`;
}

export function memoryChartSubtitle(
    memory: MemoryScore,
    formatPercent: (v: number) => string,
    formatRatio: (v: number) => string,
): string {
    return [
        memoryHero(memory, formatPercent),
        `${memory.studiedCards} cards`,
        `${formatRatio(memory.coverageRatio)} coverage`,
        `FSRS ${fsrsStatus(memory.abstentionRequirements)}`,
    ].join(" · ");
}

export function performanceChartSubtitle(
    performance: PerformanceScore,
    formatPercent: (v: number) => string,
): string {
    return [
        performanceHero(performance, formatPercent),
        `${performance.attemptCount} attempts`,
    ].join(" · ");
}

export function readinessChartSubtitle(
    readiness: ReadinessScore,
    formatPercent: (v: number) => string,
    formatRatio: (v: number) => string,
): string {
    if (!readiness.sufficientData || readiness.projectedScore === undefined) {
        return emptyStateTitle("readiness");
    }
    const parts = [readinessHero(readiness, formatPercent)];
    parts.push(`${formatRatio(readiness.coverageRatio)} coverage`);
    if (readiness.confidenceLevel) {
        parts.push(`${capitalizeLabel(readiness.confidenceLevel)} confidence`);
    }
    return parts.join(" · ");
}

export function estimatedGreChartSubtitle(
    estimate: EstimatedGreScore,
    readiness?: ReadinessScore,
): string {
    if (estimate.combinedScore === undefined) {
        return emptyStateTitle("estimatedGreChart");
    }
    return `${estimatedGreHero(estimate)} · Confidence ${estimatedGreConfidence(estimate, readiness)}`;
}
