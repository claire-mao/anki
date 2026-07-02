// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceAttempt, ReadinessCalibrationBin } from "@generated/anki/brainlift_pb";

export type ConfidenceVisual = "empty" | "preliminary" | "low" | "medium" | "high";

export function clampPercent(value: number): number {
    return Math.max(0, Math.min(100, Math.round(value)));
}

export function ratioToPercent(ratio: number): number {
    return clampPercent(ratio * 100);
}

export function confidenceVisual(confidence: string): ConfidenceVisual {
    const normalized = confidence.trim().toLowerCase();
    if (!normalized || normalized === "—" || normalized === "-") {
        return "empty";
    }
    if (normalized.includes("preliminary")) {
        return "preliminary";
    }
    if (normalized.includes("high")) {
        return "high";
    }
    if (normalized.includes("medium") || normalized.includes("med")) {
        return "medium";
    }
    if (normalized.includes("low")) {
        return "low";
    }
    return "medium";
}

export function rollingAccuracySeries(
    attempts: PerformanceAttempt[],
    windowSize = 5,
): number[] {
    if (attempts.length === 0) {
        return [];
    }
    const ordered = [...attempts].reverse();
    return ordered.map((_, index) => {
        const start = Math.max(0, index - windowSize + 1);
        const slice = ordered.slice(start, index + 1);
        const correct = slice.filter((attempt) => attempt.correct).length;
        return clampPercent((correct / slice.length) * 100);
    });
}

export function calibrationOutcomeSeries(bins: ReadinessCalibrationBin[]): number[] {
    return bins
        .filter((bin) => bin.count > 0)
        .sort((a, b) => a.predictedMean - b.predictedMean)
        .map((bin) => clampPercent(bin.outcomeMean));
}
