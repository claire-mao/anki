// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { ReadinessCalibrationStats, ReadinessScore } from "@generated/anki/brainlift_pb";

import { calibrationOutcomeSeries } from "./indicator-utils";
import { capitalizeLabel } from "./summary-metrics";
import { formatRange } from "./score-format";

/** Matches `MIN_CALIBRATION_HELD_OUT` in rslib calibration (display only). */
const MIN_CALIBRATION_HELD_OUT = 5;

export type CalibrationImprovementItem = {
    id: string;
    label: string;
    detail: string;
    met: boolean;
};

export type CalibrationPresentation = {
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
        return "Unverified";
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
        return stats.assessment || "Not enough held-out outcomes yet.";
    }
    const parts: string[] = [];
    if (stats.brierScore !== undefined) {
        parts.push(`Brier ${stats.brierScore.toFixed(3)}`);
    }
    if (stats.meanAbsoluteError !== undefined) {
        parts.push(`${stats.meanAbsoluteError.toFixed(1)} pt error`);
    }
    parts.push(`${stats.heldOutCount} held-out pairs`);
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
            value: `${stats.heldOutCount} held-out pairs`,
            detail: `${stats.resolvedOutcomes} outcomes recorded`,
        };
    }
    return {
        value: "Building history",
        detail: `${stats.totalPredictions} predictions logged`,
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
        return "Calibration capped confidence while accuracy stabilizes";
    }
    if (!readiness.sufficientData) {
        return readiness.abstainReason || "Unlock readiness to set confidence";
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

    if (stats.sufficientData && !stats.wellCalibrated && readiness.sufficientData) {
        notes.push("Poor calibration lowers confidence until predictions match outcomes.");
    } else if (stats.sufficientData && stats.wellCalibrated) {
        notes.push("Verified calibration supports the current confidence level.");
    } else if (!stats.sufficientData) {
        notes.push(
            `Confidence stays provisional until ${MIN_CALIBRATION_HELD_OUT} held-out outcomes are recorded.`,
        );
    }

    if (readiness.calibrationNote && !notes.some((note) => note.includes(readiness.calibrationNote))) {
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
        const remaining = Math.max(0, MIN_CALIBRATION_HELD_OUT - stats.heldOutCount);
        items.push({
            id: "calibration_history",
            label: "Calibration history",
            detail:
                remaining > 0
                    ? `${remaining} more held-out outcome${remaining === 1 ? "" : "s"} needed`
                    : stats.assessment,
            met: stats.sufficientData,
        });
    } else if (!stats.wellCalibrated) {
        items.push({
            id: "calibration_quality",
            label: "Prediction accuracy",
            detail: "Keep studying and practicing so later outcomes match predictions",
            met: false,
        });
    }

    return items;
}

function trendCaption(stats: ReadinessCalibrationStats, points: number[]): string {
    if (points.length < 2) {
        return stats.assessment || "Trend appears after calibration bins fill in";
    }
    if (stats.wellCalibrated) {
        return "Outcomes rise with predicted bins — predictions track reality";
    }
    if (stats.sufficientData) {
        return "Outcome trend across predicted score bins";
    }
    return "Calibration trend building";
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

export function formatCalibrationHeldOutProgress(stats: ReadinessCalibrationStats): string {
    return `${stats.heldOutCount} / ${MIN_CALIBRATION_HELD_OUT} held-out pairs`;
}
