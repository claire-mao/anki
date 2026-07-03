// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    AbstentionRequirement,
    EstimatedGreScore,
    MemoryScore,
    PerformanceScore,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import { emptyStateContent } from "./empty-states";
import { settingsNavAction } from "./gre-navigation";
import { formatGreScoreRange, formatPercent, formatRange } from "./score-format";
import { capitalizeLabel, estimatedGreConfidence, unmetRequirements } from "./summary-metrics";

export type PredictionAction = {
    label: string;
    href?: string;
    bridge?: string;
};

export type PredictionDetailRow = {
    label: string;
    value: string;
};

export function estimatedGreUnlocked(estimate: EstimatedGreScore): boolean {
    return estimate.combinedScore !== undefined;
}

export function readinessUnlocked(readiness: ReadinessScore): boolean {
    return readiness.sufficientData && readiness.projectedScore !== undefined;
}

export function estimatedGreWhy(estimate: EstimatedGreScore): string {
    if (estimate.combinedScore === undefined) {
        return "Combines memory, practice, and coverage into the 260–340 GRE scale.";
    }
    if (estimate.preliminary) {
        return "Early Quant + Verbal mapping from partial evidence.";
    }
    return "Quant + Verbal GRE score mapped from your readiness index.";
}

export function estimatedGreEvidence(
    estimate: EstimatedGreScore,
    memory?: MemoryScore,
    performance?: PerformanceScore,
): string {
    if (estimate.detail) {
        return estimate.detail;
    }
    if (estimate.abstainReason) {
        return estimate.abstainReason;
    }
    const parts: string[] = [];
    if (memory?.detail) {
        parts.push(memory.detail);
    }
    if (performance?.detail) {
        parts.push(performance.detail);
    }
    return parts.join(" · ") || "Complete unlock milestones to build evidence.";
}

export function estimatedGreNextAction(
    estimate: EstimatedGreScore,
    requirements: AbstentionRequirement[] = [],
): PredictionAction {
    if (estimatedGreUnlocked(estimate) && !estimate.preliminary) {
        return { label: "View progress", href: "/progress" };
    }
    if (estimate.preliminary) {
        return { label: "View study plan", href: "/study-plan" };
    }
    const unmet = unmetRequirements(requirements);
    return actionForRequirements(unmet, emptyStateContent("estimatedGre").action);
}

export function estimatedGreDetailRows(estimate: EstimatedGreScore): PredictionDetailRow[] {
    const rows: PredictionDetailRow[] = [];
    const combinedRange = formatGreScoreRange(
        estimate.combinedScoreLow,
        estimate.combinedScoreHigh,
    );
    if (combinedRange) {
        rows.push({ label: "Combined range", value: combinedRange });
    }
    if (estimate.quantScore !== undefined) {
        rows.push({
            label: "Quant",
            value: formatSectionGreScore(
                estimate.quantScore,
                estimate.quantScoreLow,
                estimate.quantScoreHigh,
            ),
        });
    }
    if (estimate.verbalScore !== undefined) {
        rows.push({
            label: "Verbal",
            value: formatSectionGreScore(
                estimate.verbalScore,
                estimate.verbalScoreLow,
                estimate.verbalScoreHigh,
            ),
        });
    }
    if (estimate.preliminary) {
        rows.push({ label: "Status", value: "Preliminary — readiness not yet unlocked" });
    }
    return rows;
}

export function readinessWhy(): string {
    return "Projected score from memory, practice, and topic coverage.";
}

export function readinessEvidence(readiness: ReadinessScore): string {
    if (readiness.evidenceSummary) {
        return readiness.evidenceSummary;
    }
    if (readiness.abstainReason) {
        return readiness.abstainReason;
    }
    return `${formatPercent(readiness.coverageRatio * 100)} topic coverage recorded.`;
}

export function readinessConfidenceLabel(readiness: ReadinessScore): string {
    if (readiness.confidenceLevel) {
        return capitalizeLabel(readiness.confidenceLevel);
    }
    if (!readinessUnlocked(readiness)) {
        return "Insufficient";
    }
    return "—";
}

export function readinessNextAction(readiness: ReadinessScore): PredictionAction {
    if (readinessUnlocked(readiness)) {
        if (
            readiness.confidenceLevel === "low"
            || readiness.coverageRatio < 0.5
        ) {
            return { label: "Expand coverage", href: "/study-plan" };
        }
        return { label: "View calibration", href: "/readiness" };
    }
    return actionForRequirements(
        unmetRequirements(readiness.abstentionRequirements),
        emptyStateContent("readiness").action,
    );
}

export function readinessDetailRows(readiness: ReadinessScore): PredictionDetailRow[] {
    const rows: PredictionDetailRow[] = [];
    const range = formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh);
    if (range) {
        rows.push({ label: "Projected range", value: range });
    }
    rows.push({
        label: "Coverage",
        value: formatPercent(readiness.coverageRatio * 100),
    });
    if (readiness.calibrationNote) {
        rows.push({ label: "Calibration", value: readiness.calibrationNote });
    }
    return rows;
}

function formatSectionGreScore(
    score: number,
    low: number | undefined,
    high: number | undefined,
): string {
    const range = formatGreScoreRange(low, high);
    return range ? `${score} (${range})` : String(score);
}

function actionForRequirements(
    unmet: AbstentionRequirement[],
    fallback: PredictionAction,
): PredictionAction {
    const next = unmet[0];
    if (!next) {
        return fallback;
    }
    switch (next.id) {
        case "practice_attempts":
            return { label: "Start practice", href: "/practice" };
        case "studied_cards":
            return { label: "Start review", href: "/review" };
        case "topic_coverage":
            return { label: "View study plan", href: "/study-plan" };
        case "fsrs_enabled":
            return { ...settingsNavAction(), label: "Set up deck" };
        default:
            return fallback;
    }
}

export function topicGlobalReadinessScore(score: number | undefined, formatPercent: (v: number) => string): string {
    if (score !== undefined) {
        return formatPercent(score);
    }
    return "—";
}

export function topicGlobalReadinessEvidence(
    summary: string,
    topicContribution: number | undefined,
): string {
    const parts: string[] = [];
    if (summary) {
        parts.push(summary);
    }
    if (topicContribution !== undefined) {
        parts.push(`This topic contributes ~${topicContribution.toFixed(1)} readiness pts`);
    }
    return parts.join(" · ") || "Review and practice this topic to add evidence.";
}

export function topicGlobalReadinessNextAction(
    covered: boolean,
    practiceTotal: number,
): PredictionAction {
    if (!covered) {
        return { label: "Start review", href: "/review" };
    }
    if (practiceTotal === 0) {
        return { label: "Start practice", href: "/practice" };
    }
    return { label: "View calibration", href: "/readiness" };
}

export function topicGlobalReadinessConfidence(score: number | undefined): string {
    return score !== undefined ? "See calibration page" : "Insufficient";
}

export function estimatedGreConfidenceLabel(
    estimate: EstimatedGreScore,
    readiness?: ReadinessScore,
): string {
    return estimatedGreConfidence(estimate, readiness);
}
