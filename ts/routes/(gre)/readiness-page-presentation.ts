// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    DashboardCoverage,
    DashboardTopicInsight,
    EstimatedGreScore,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import { presentCalibration } from "./calibration-presentation";
import {
    coverageAwareReadinessUnlocked,
    coverageBlocksReadiness,
    coverageReadinessReason,
    presentCoverageSummary,
} from "./coverage-presentation";
import { buildReadinessExplainability } from "./prediction-explainability";
import type { PredictionAction } from "./prediction-presentation";
import { readinessNextAction } from "./prediction-presentation";
import { formatGreScoreRange, formatPercent, formatRange, formatRatio } from "./score-format";
import { capitalizeLabel, estimatedGreHero, memoryHero, performanceHero, unmetRequirements } from "./summary-metrics";

export type ReadinessEvidenceLine = {
    label: string;
    detail: string;
};

export type ReadinessPagePresentation = {
    available: boolean;
    unavailableTitle: string;
    unavailableReason: string;
    unavailableDetails: string[];
    readinessScore: string | null;
    estimatedGre: string;
    estimatedGreDetail: string | null;
    confidenceInterval: string;
    confidenceLevel: string;
    evidenceUsed: ReadinessEvidenceLine[];
    evidenceMissing: ReadinessEvidenceLine[];
    coverage: string;
    memory: string;
    performance: string;
    calibrationQuality: string;
    lastUpdated: string;
    nextAction: PredictionAction;
};

function formatTimestampMillis(millis: bigint): string {
    return new Date(Number(millis)).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
    });
}

function estimatedGrePresentation(
    estimate: EstimatedGreScore,
    available: boolean,
): {
    value: string;
    detail: string | null;
} {
    if (!available || estimate.combinedScore === undefined) {
        return {
            value: "Unavailable",
            detail: estimate.abstainReason
                || estimate.detail
                || "Not enough evidence for a GRE score yet.",
        };
    }
    const range = formatGreScoreRange(estimate.combinedScoreLow, estimate.combinedScoreHigh);
    const score = estimatedGreHero(estimate);
    return {
        value: range ? `${score} (${range})` : score,
        detail: estimate.preliminary
            ? "Preliminary mapping from partial evidence."
            : estimate.detail || null,
    };
}

function confidenceIntervalPresentation(
    available: boolean,
    readiness: ReadinessScore,
): string {
    if (!available) {
        return "—";
    }
    const range = formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh);
    return range ?? "—";
}

function confidenceLevelPresentation(
    available: boolean,
    readiness: ReadinessScore,
): string {
    if (!available) {
        return "Insufficient";
    }
    if (readiness.confidenceLevel) {
        return capitalizeLabel(readiness.confidenceLevel);
    }
    return "Medium";
}

function coveragePresentation(coverage: DashboardCoverage): string {
    const summary = presentCoverageSummary(coverage);
    const sectionParts = summary.sections.map((section) => `${section.label} ${section.percent}%`);
    const base = `${summary.totalPercent}% weighted`;
    if (sectionParts.length === 0) {
        return base;
    }
    return `${base} · ${sectionParts.join(" · ")}`;
}

function memoryPresentation(memory: MemoryScore): string {
    const score = memoryHero(memory, formatPercent);
    const parts = [score !== "—" ? `${score} retention` : "Insufficient memory evidence"];
    parts.push(`${memory.studiedCards} cards studied`);
    parts.push(`${formatRatio(memory.coverageRatio)} catalog coverage`);
    return parts.join(" · ");
}

function performancePresentation(performance: PerformanceScore): string {
    const score = performanceHero(performance, formatPercent);
    const parts = [
        score !== "—" ? `${score} accuracy` : "Insufficient practice evidence",
        `${performance.attemptCount} attempts`,
    ];
    return parts.join(" · ");
}

function calibrationQualityPresentation(
    readiness: ReadinessScore,
    calibration: ReadinessCalibrationStats,
): string {
    const model = presentCalibration(readiness, calibration);
    return `${model.predictionQuality} · ${model.predictionQualityDetail}`;
}

function unavailableReason(
    readiness: ReadinessScore,
    coverage: DashboardCoverage,
): { title: string; reason: string; details: string[] } {
    if (coverageBlocksReadiness(coverage)) {
        return {
            title: "Readiness unavailable",
            reason: coverageReadinessReason(coverage),
            details: coverage.uncoveredTopics.slice(0, 5).map((topic) => topic.studyLabel || topic.displayName),
        };
    }

    if (readiness.abstainReason) {
        return {
            title: "Readiness unavailable",
            reason: readiness.abstainReason,
            details: unmetRequirements(readiness.abstentionRequirements).map(
                (req) => `${req.label}: ${req.status}`,
            ),
        };
    }

    const unmet = unmetRequirements(readiness.abstentionRequirements);
    const primary = unmet[0];
    return {
        title: "Readiness unavailable",
        reason: primary?.status || "Not enough evidence to estimate readiness.",
        details: unmet.map((req) => `${req.label}: ${req.status}`),
    };
}

function evidenceUsedLines(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
    weakTopics: DashboardTopicInsight[],
    calibration: ReadinessCalibrationStats,
): ReadinessEvidenceLine[] {
    const explainability = buildReadinessExplainability({
        memory,
        performance,
        readiness,
        weakTopics,
        calibration,
    });

    const lines: ReadinessEvidenceLine[] = explainability.basedOn
        .filter((pillar) => pillar.met)
        .map((pillar) => ({
            label: pillar.label,
            detail: pillar.status,
        }));

    if (readiness.evidenceSummary) {
        lines.unshift({
            label: "Summary",
            detail: readiness.evidenceSummary,
        });
    }

    return lines;
}

function evidenceMissingLines(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
    coverage: DashboardCoverage,
    weakTopics: DashboardTopicInsight[],
    calibration: ReadinessCalibrationStats,
): ReadinessEvidenceLine[] {
    const lines: ReadinessEvidenceLine[] = [];
    const explainability = buildReadinessExplainability({
        memory,
        performance,
        readiness,
        weakTopics,
        calibration,
    });

    for (const pillar of explainability.basedOn.filter((item) => !item.met)) {
        lines.push({ label: pillar.label, detail: pillar.status });
    }

    for (const item of explainability.needsImprovement) {
        lines.push({
            label: item.label,
            detail: item.detail || "Needs more evidence",
        });
    }

    if (coverageBlocksReadiness(coverage)) {
        lines.unshift({
            label: "GRE coverage",
            detail: coverageReadinessReason(coverage),
        });
    }

    for (const req of unmetRequirements(readiness.abstentionRequirements)) {
        lines.push({
            label: req.label,
            detail: req.nextStep || req.status,
        });
    }

    const seen = new Set<string>();
    return lines.filter((line) => {
        const key = `${line.label}:${line.detail}`;
        if (seen.has(key)) {
            return false;
        }
        seen.add(key);
        return true;
    });
}

function nextActionForPage(
    available: boolean,
    readiness: ReadinessScore,
    coverage: DashboardCoverage,
): PredictionAction {
    if (coverageBlocksReadiness(coverage)) {
        const recommendation = coverage.uncoveredTopics[0]?.studyLabel;
        if (recommendation) {
            return { label: recommendation, href: "/study-plan" };
        }
        return { label: "Expand GRE coverage", href: "/study-plan" };
    }
    return readinessNextAction(readiness);
}

export function presentReadinessPage(input: {
    readiness: ReadinessScore;
    calibration: ReadinessCalibrationStats;
    memory: MemoryScore;
    performance: PerformanceScore;
    estimatedGre: EstimatedGreScore;
    coverage: DashboardCoverage;
    weakTopics?: DashboardTopicInsight[];
    computedAtMillis: bigint;
}): ReadinessPagePresentation {
    const available = coverageAwareReadinessUnlocked(input.readiness, input.coverage);
    const unavailable = unavailableReason(input.readiness, input.coverage);
    const estimated = estimatedGrePresentation(input.estimatedGre, available);

    return {
        available,
        unavailableTitle: unavailable.title,
        unavailableReason: unavailable.reason,
        unavailableDetails: unavailable.details,
        readinessScore: available && input.readiness.projectedScore !== undefined
            ? formatPercent(input.readiness.projectedScore)
            : null,
        estimatedGre: estimated.value,
        estimatedGreDetail: estimated.detail,
        confidenceInterval: confidenceIntervalPresentation(available, input.readiness),
        confidenceLevel: confidenceLevelPresentation(available, input.readiness),
        evidenceUsed: evidenceUsedLines(
            input.memory,
            input.performance,
            input.readiness,
            input.weakTopics ?? [],
            input.calibration,
        ),
        evidenceMissing: available
            ? []
            : evidenceMissingLines(
                input.memory,
                input.performance,
                input.readiness,
                input.coverage,
                input.weakTopics ?? [],
                input.calibration,
            ),
        coverage: coveragePresentation(input.coverage),
        memory: memoryPresentation(input.memory),
        performance: performancePresentation(input.performance),
        calibrationQuality: calibrationQualityPresentation(input.readiness, input.calibration),
        lastUpdated: formatTimestampMillis(
            input.readiness.lastUpdatedMillis || input.computedAtMillis,
        ),
        nextAction: nextActionForPage(available, input.readiness, input.coverage),
    };
}
