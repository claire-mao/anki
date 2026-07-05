// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { ReadinessCalibrationStats, ReadinessScore } from "@generated/anki/brainlift_pb";

import { calibrationOutcomeSeries } from "./indicator-utils";
import { formatRange } from "./score-format";
import { capitalizeLabel } from "./summary-metrics";

/** Matches `MIN_CALIBRATION_HELD_OUT` in rslib calibration (display only). */
const MIN_CALIBRATION_HELD_OUT = 5;

/** Short hint for the readiness-page track record row (shown only after 5 checks). */
export const CALIBRATION_EXPLANATION = "How closely past score estimates matched your actual study results.";

/** Whether the track-record metric is meaningful enough to show on Readiness. */
export function calibrationQualityVisible(stats: ReadinessCalibrationStats): boolean {
    return stats.sufficientData;
}

export type CalibrationImprovementItem = {
    id: string;
    label: string;
    detail: string;
    met: boolean;
};

export type CalibrationPresentation = {
    /** True once enough checks exist for the full panel and track-record row. */
    showFullPanel: boolean;
    /** One-line copy for the pre-verification state. */
    earlyStateSummary: string;
    currentConfidence: string;
    confidenceCaption: string;
    historicalAccuracy: string;
    historicalAccuracyDetail: string;
    predictionQuality: string;
    predictionQualityDetail: string;
    trendPoints: number[];
    trendCaption: string;
    trendAvailable: boolean;
    confidenceChangeNotes: string[];
    improvementItems: CalibrationImprovementItem[];
    assessment: string;
};

function predictionQualityLabel(stats: ReadinessCalibrationStats): string {
    if (!stats.sufficientData) {
        return formatCalibrationChecksProgress(stats);
    }
    if (stats.wellCalibrated) {
        return "Good fit";
    }
    if (stats.brierScore !== undefined && stats.brierScore <= 0.15) {
        return "Moderate fit";
    }
    return "Poor fit";
}

function predictionQualityDetail(stats: ReadinessCalibrationStats): string {
    if (!stats.sufficientData) {
        return "";
    }
    const parts: string[] = [];
    if (stats.meanAbsoluteError !== undefined) {
        parts.push(`${stats.meanAbsoluteError.toFixed(1)} pt average error`);
    }
    parts.push(`${stats.heldOutCount} checks completed`);
    return parts.join(" · ");
}

function historicalAccuracy(stats: ReadinessCalibrationStats): {
    value: string;
    detail: string;
} {
    if (stats.meanAbsoluteError !== undefined) {
        return {
            value: `${stats.meanAbsoluteError.toFixed(1)} pt error`,
            detail: `${stats.resolvedOutcomes} resolved of ${stats.totalPredictions} predictions`,
        };
    }
    if (stats.heldOutCount > 0) {
        return {
            value: formatCalibrationChecksProgress(stats),
            detail: `${stats.resolvedOutcomes} results compared`,
        };
    }
    if (stats.totalPredictions > 0) {
        return {
            value: `${stats.totalPredictions}`,
            detail: "predictions logged",
        };
    }
    return {
        value: "None yet",
        detail: "Logs once readiness predictions start",
    };
}

function confidenceCaption(
    readiness: ReadinessScore,
    stats: ReadinessCalibrationStats,
): string {
    const range = formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh);
    if (range) {
        return `Score range ${range}`;
    }
    if (stats.sufficientData && !stats.wellCalibrated && readiness.sufficientData) {
        return "Estimates haven't matched results yet — confidence stays cautious";
    }
    if (!readiness.sufficientData) {
        return "Provisional until readiness gates clear";
    }
    return readiness.evidenceSummary || "Based on memory, practice, and coverage";
}

function confidenceChangeNotes(
    readiness: ReadinessScore,
    stats: ReadinessCalibrationStats,
): string[] {
    const notes: string[] = [];

    if (readiness.sufficientData && readiness.confidenceLevel) {
        notes.push(
            `Evidence sets baseline confidence to ${capitalizeLabel(readiness.confidenceLevel)}.`,
        );
    }

    if (!readiness.sufficientData) {
        for (const req of readiness.abstentionRequirements.filter((item) => !item.met)) {
            const note = req.nextStep || req.status;
            if (note && !notes.includes(note)) {
                notes.push(note);
            }
        }
    }

    if (stats.sufficientData && !stats.wellCalibrated && readiness.sufficientData) {
        notes.push("Past estimates haven't matched your results yet — confidence stays cautious.");
    } else if (stats.sufficientData && stats.wellCalibrated) {
        notes.push("Past estimates match your results — confidence is well supported.");
    } else if (!stats.sufficientData) {
        const remaining = Math.max(0, MIN_CALIBRATION_HELD_OUT - stats.heldOutCount);
        if (remaining > 0) {
            notes.push(
                `${remaining} more check${remaining === 1 ? "" : "s"} before we can score estimate accuracy.`,
            );
        }
    }

    if (
        stats.sufficientData
        && readiness.calibrationNote
        && !notes.some((note) => note.includes(readiness.calibrationNote))
    ) {
        notes.push(readiness.calibrationNote);
    }

    return notes.slice(0, 3);
}

function improvementItems(
    readiness: ReadinessScore,
    stats: ReadinessCalibrationStats,
): CalibrationImprovementItem[] {
    const items: CalibrationImprovementItem[] = [];

    for (const req of readiness.abstentionRequirements) {
        items.push({
            id: req.id,
            label: req.label,
            detail: req.met ? req.status : req.nextStep || req.status,
            met: req.met,
        });
    }

    if (!stats.sufficientData) {
        items.push({
            id: "calibration_history",
            label: "Estimate checks",
            detail: formatCalibrationChecksProgress(stats),
            met: stats.sufficientData,
        });
    } else if (!stats.wellCalibrated) {
        items.push({
            id: "calibration_quality",
            label: "Estimate accuracy",
            detail: "Keep studying so future results match your score estimates",
            met: false,
        });
    }

    return items;
}

function trendCaption(stats: ReadinessCalibrationStats, points: number[]): string {
    if (points.length < 2) {
        return "Needs more study sessions across different score levels";
    }
    if (stats.wellCalibrated) {
        return "Results track your estimates across score levels";
    }
    if (stats.sufficientData) {
        return "How results compare to estimates at each score level";
    }
    return "Track record still building";
}

function earlyStateSummary(stats: ReadinessCalibrationStats): string {
    const progress = formatCalibrationChecksProgress(stats);
    if (stats.heldOutCount === 0) {
        return `We compare past score estimates to your results as you study. ${progress}.`;
    }
    return `${progress}. Keep studying — accuracy scoring unlocks after 5 checks.`;
}

export function presentCalibration(
    readiness: ReadinessScore,
    stats: ReadinessCalibrationStats,
): CalibrationPresentation {
    const accuracy = historicalAccuracy(stats);
    const trendPoints = calibrationOutcomeSeries(stats.calibrationCurve);

    let currentConfidence = capitalizeLabel(readiness.confidenceLevel);
    if (!readiness.confidenceLevel) {
        currentConfidence = readiness.sufficientData ? "Medium" : "Insufficient";
    }

    return {
        showFullPanel: stats.sufficientData,
        earlyStateSummary: earlyStateSummary(stats),
        currentConfidence,
        confidenceCaption: confidenceCaption(readiness, stats),
        historicalAccuracy: accuracy.value,
        historicalAccuracyDetail: accuracy.detail,
        predictionQuality: predictionQualityLabel(stats),
        predictionQualityDetail: predictionQualityDetail(stats),
        trendPoints,
        trendCaption: trendCaption(stats, trendPoints),
        trendAvailable: trendPoints.length >= 2,
        confidenceChangeNotes: confidenceChangeNotes(readiness, stats),
        improvementItems: improvementItems(readiness, stats),
        assessment: stats.assessment || readiness.calibrationNote || "",
    };
}

export function formatCalibrationChecksProgress(stats: ReadinessCalibrationStats): string {
    return `${stats.heldOutCount} of ${MIN_CALIBRATION_HELD_OUT} checks completed`;
}

/** @deprecated Use formatCalibrationChecksProgress */
export const formatCalibrationHeldOutProgress = formatCalibrationChecksProgress;
