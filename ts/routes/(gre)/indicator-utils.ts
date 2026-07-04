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

export type AccuracyTrendHorizon = "1d" | "3d" | "7d" | "30d" | "all";

export interface AccuracyTrendPoint {
    answeredAtSecs: number;
    accuracy: number;
}

const SECONDS_PER_DAY = 86_400;

export function filterAttemptsByHorizon(
    attempts: PerformanceAttempt[],
    horizon: AccuracyTrendHorizon,
    nowSecs = Math.floor(Date.now() / 1000),
): PerformanceAttempt[] {
    if (horizon === "all") {
        return attempts;
    }
    const horizonDays: Record<Exclude<AccuracyTrendHorizon, "all">, number> = {
        "1d": 1,
        "3d": 3,
        "7d": 7,
        "30d": 30,
    };
    const cutoff = BigInt(nowSecs - horizonDays[horizon] * SECONDS_PER_DAY);
    return attempts.filter((attempt) => attempt.answeredAtSecs >= cutoff);
}

export function rollingAccuracyTrendPoints(
    attempts: PerformanceAttempt[],
    windowSize = 5,
): AccuracyTrendPoint[] {
    if (attempts.length === 0) {
        return [];
    }
    const ordered = [...attempts].reverse();
    return ordered.map((attempt, index) => {
        const start = Math.max(0, index - windowSize + 1);
        const slice = ordered.slice(start, index + 1);
        const correct = slice.filter((item) => item.correct).length;
        return {
            answeredAtSecs: Number(attempt.answeredAtSecs),
            accuracy: clampPercent((correct / slice.length) * 100),
        };
    });
}

export function rollingAccuracySeries(
    attempts: PerformanceAttempt[],
    windowSize = 5,
): number[] {
    return rollingAccuracyTrendPoints(attempts, windowSize).map((point) => point.accuracy);
}

export function accuracyHorizonLabel(horizon: AccuracyTrendHorizon): string {
    switch (horizon) {
        case "1d":
            return "Last 1 day accuracy";
        case "3d":
            return "Last 3 days accuracy";
        case "7d":
            return "Last 7 days accuracy";
        case "30d":
            return "Last 30 days accuracy";
        case "all":
            return "All-time accuracy";
    }
}

export function calibrationOutcomeSeries(bins: ReadinessCalibrationBin[]): number[] {
    return bins
        .filter((bin) => bin.count > 0)
        .sort((a, b) => a.predictedMean - b.predictedMean)
        .map((bin) => clampPercent(bin.outcomeMean));
}
