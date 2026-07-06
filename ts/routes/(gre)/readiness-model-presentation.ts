// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import type { ReadinessEvidenceLine } from "./readiness-page-presentation";
import { formatPercent, formatRange, formatRatio } from "./score-format";
import { capitalizeLabel } from "./summary-metrics";

/** Documented composite weights — `rslib/src/gre_atlas/readiness.rs`. */
export const READINESS_MEMORY_WEIGHT = 0.45;
export const READINESS_PERFORMANCE_WEIGHT = 0.45;
export const READINESS_COVERAGE_WEIGHT = 0.1;

/** Documented confidence interval thresholds — `readiness.rs::confidence_level`. */
export const READINESS_CONFIDENCE_HIGH_MAX_WIDTH = 8;
export const READINESS_CONFIDENCE_MEDIUM_MAX_WIDTH = 15;
export const READINESS_CONFIDENCE_HIGH_MIN_CARDS = 400;
export const READINESS_CONFIDENCE_HIGH_MIN_ATTEMPTS = 50;
export const READINESS_CONFIDENCE_HIGH_MIN_COVERAGE = 0.7;

export type ReadinessModelStepId =
    | "memory"
    | "performance"
    | "coverage"
    | "calibration"
    | "readiness";

export type ReadinessModelStep = {
    id: ReadinessModelStepId;
    label: string;
    value: string | null;
    interval: string | null;
    detail: string;
    available: boolean;
};

export type ReadinessModelExplanation = {
    score: string | null;
    predictionInterval: string;
    confidence: string;
    confidenceFactors: string[];
    evidenceUsed: ReadinessEvidenceLine[];
    steps: ReadinessModelStep[];
    compositeSummary: string | null;
};

function requirementById(
    requirements: MemoryScore["abstentionRequirements"],
    id: string,
) {
    return requirements.find((req) => req.id === id);
}

/** Mirrors `composite_readiness_score` in `calibration.rs`. */
export function compositeReadinessScore(
    memoryScore: number,
    performanceScore: number,
    coverageRatio: number,
): number {
    const memoryNorm = memoryScore / 100;
    const performanceNorm = performanceScore / 100;
    return (
        (READINESS_MEMORY_WEIGHT * memoryNorm
            + READINESS_PERFORMANCE_WEIGHT * performanceNorm
            + READINESS_COVERAGE_WEIGHT * coverageRatio)
        * 100
    );
}

/** Mirrors interval combination in `compute_readiness_score`. */
export function readinessProjectionInterval(
    memory: MemoryScore,
    performance: PerformanceScore,
    projected: number,
): { low: number; high: number } {
    const memoryWidth = (memory.valueHigh ?? projected) - (memory.valueLow ?? projected);
    const performanceWidth =
        (performance.valueHigh ?? projected) - (performance.valueLow ?? projected);
    const combinedMargin = Math.sqrt(memoryWidth ** 2 + performanceWidth ** 2) / 2;
    return {
        low: Math.max(0, Math.min(100, projected - combinedMargin)),
        high: Math.max(0, Math.min(100, projected + combinedMargin)),
    };
}

function memoryStepDetail(memory: MemoryScore): string {
    if (memory.detail) {
        return memory.detail;
    }
    return memory.abstainReason
        || requirementById(memory.abstentionRequirements, "studied_cards")?.status
        || "Memory evidence insufficient";
}

function performanceStepDetail(performance: PerformanceScore): string {
    if (performance.detail) {
        return performance.detail;
    }
    return performance.abstainReason
        || requirementById(performance.abstentionRequirements, "practice_attempts")?.status
        || "Practice evidence insufficient";
}

function coverageStepDetail(memory: MemoryScore, readiness: ReadinessScore): string {
    const coverageReq = requirementById(memory.abstentionRequirements, "topic_coverage");
    if (coverageReq?.status) {
        return coverageReq.status;
    }
    if (readiness.coverageRatio > 0) {
        return `${formatRatio(readiness.coverageRatio)} exam-weighted catalog covered`;
    }
    return "No catalog coverage recorded";
}

function calibrationStepDetail(
    readiness: ReadinessScore,
    calibration: ReadinessCalibrationStats | undefined,
): string {
    if (readiness.calibrationNote) {
        return readiness.calibrationNote;
    }
    if (calibration?.assessment) {
        return calibration.assessment;
    }
    return "Calibration history not yet available";
}

function readinessStepDetail(
    readiness: ReadinessScore,
    compositeSummary: string | null,
): string {
    if (readiness.evidenceSummary) {
        return compositeSummary
            ? `${readiness.evidenceSummary} · ${compositeSummary}`
            : readiness.evidenceSummary;
    }
    return compositeSummary || readiness.abstainReason || "Readiness not yet computed";
}

function compositeSummaryFromInputs(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
): string | null {
    if (
        memory.value === undefined
        || performance.value === undefined
        || readiness.projectedScore === undefined
    ) {
        return null;
    }

    const memoryPoints = READINESS_MEMORY_WEIGHT * memory.value;
    const performancePoints = READINESS_PERFORMANCE_WEIGHT * performance.value;
    const coveragePoints = READINESS_COVERAGE_WEIGHT * readiness.coverageRatio * 100;
    const projected = compositeReadinessScore(
        memory.value,
        performance.value,
        readiness.coverageRatio,
    );

    return [
        `${formatPercent(memoryPoints)} memory`,
        `${formatPercent(performancePoints)} performance`,
        `${formatPercent(coveragePoints)} coverage`,
        `= ${formatPercent(projected)}`,
    ].join(" + ");
}

function confidenceFactors(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
): string[] {
    const factors: string[] = [
        `${memory.studiedCards} studied cards`,
        `${performance.attemptCount} practice attempts`,
        `${formatRatio(readiness.coverageRatio)} catalog coverage`,
    ];

    if (
        readiness.projectedScoreLow !== undefined
        && readiness.projectedScoreHigh !== undefined
    ) {
        const width = readiness.projectedScoreHigh - readiness.projectedScoreLow;
        factors.push(`${Math.round(width)}-point prediction interval`);
    }

    if (readiness.calibrationSufficientData && readiness.calibrationNote) {
        factors.push(readiness.calibrationNote);
    }

    return factors;
}

function buildEvidenceUsed(readiness: ReadinessScore): ReadinessEvidenceLine[] {
    const lines: ReadinessEvidenceLine[] = [];
    if (readiness.evidenceSummary) {
        lines.push({ label: "Summary", detail: readiness.evidenceSummary });
    }
    return lines;
}

function buildSteps(input: {
    memory: MemoryScore;
    performance: PerformanceScore;
    readiness: ReadinessScore;
    calibration: ReadinessCalibrationStats | undefined;
    compositeSummary: string | null;
}): ReadinessModelStep[] {
    const { memory, performance, readiness, calibration, compositeSummary } = input;

    return [
        {
            id: "memory",
            label: "Memory",
            value: memory.value !== undefined ? formatPercent(memory.value) : null,
            interval: formatRange(memory.valueLow, memory.valueHigh),
            detail: memoryStepDetail(memory),
            available: memory.sufficientData,
        },
        {
            id: "performance",
            label: "Performance",
            value: performance.value !== undefined ? formatPercent(performance.value) : null,
            interval: formatRange(performance.valueLow, performance.valueHigh),
            detail: performanceStepDetail(performance),
            available: performance.sufficientData,
        },
        {
            id: "coverage",
            label: "Coverage",
            value: readiness.coverageRatio > 0
                ? formatRatio(readiness.coverageRatio)
                : null,
            interval: null,
            detail: coverageStepDetail(memory, readiness),
            available: requirementById(memory.abstentionRequirements, "topic_coverage")?.met
                ?? readiness.coverageRatio >= 0.5,
        },
        {
            id: "calibration",
            label: "Calibration",
            value: readiness.calibrationBrierScore !== undefined
                ? `Brier ${readiness.calibrationBrierScore.toFixed(2)}`
                : calibration?.brierScore !== undefined
                    ? `Brier ${calibration.brierScore.toFixed(2)}`
                    : null,
            interval: calibration?.meanAbsoluteError !== undefined
                ? `±${calibration.meanAbsoluteError.toFixed(1)} pts error`
                : null,
            detail: calibrationStepDetail(readiness, calibration),
            available: readiness.calibrationSufficientData || calibration?.sufficientData === true,
        },
        {
            id: "readiness",
            label: "Readiness",
            value: readiness.projectedScore !== undefined
                ? formatPercent(readiness.projectedScore)
                : null,
            interval: formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh),
            detail: readinessStepDetail(readiness, compositeSummary),
            available: readiness.sufficientData && readiness.projectedScore !== undefined,
        },
    ];
}

export function presentReadinessModelExplanation(input: {
    memory: MemoryScore;
    performance: PerformanceScore;
    readiness: ReadinessScore;
    calibration?: ReadinessCalibrationStats;
    evidenceUsed?: ReadinessEvidenceLine[];
}): ReadinessModelExplanation {
    const { memory, performance, readiness, calibration } = input;
    const compositeSummary = compositeSummaryFromInputs(memory, performance, readiness);
    const available = readiness.sufficientData && readiness.projectedScore !== undefined;

    return {
        score: available && readiness.projectedScore !== undefined
            ? formatPercent(readiness.projectedScore)
            : null,
        predictionInterval: available
            ? formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh) ?? "—"
            : "—",
        confidence: available
            ? capitalizeLabel(readiness.confidenceLevel || "medium")
            : "Insufficient",
        confidenceFactors: confidenceFactors(memory, performance, readiness),
        evidenceUsed: input.evidenceUsed?.length
            ? input.evidenceUsed
            : buildEvidenceUsed(readiness),
        steps: buildSteps({
            memory,
            performance,
            readiness,
            calibration,
            compositeSummary,
        }),
        compositeSummary,
    };
}
