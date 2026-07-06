// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceChartBucket } from "@generated/anki/brainlift_pb";
import { PerformanceChartHorizon } from "@generated/anki/brainlift_pb";
import type { AccuracyTrendHorizon } from "../indicator-utils";
import { ratioToPercent } from "../indicator-utils";

export function performanceChartHorizonProto(
    horizon: AccuracyTrendHorizon,
): PerformanceChartHorizon {
    switch (horizon) {
        case "1d":
            return PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_1D;
        case "3d":
            return PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_3D;
        case "7d":
            return PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_7D;
        case "30d":
            return PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_30D;
        case "all":
            return PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_ALL;
    }
}

export function performanceChartHasData(buckets: PerformanceChartBucket[]): boolean {
    return buckets.some((bucket) => bucket.questions > 0);
}

export function performanceChartAxisLabel(
    bucket: PerformanceChartBucket,
    bucketCount: number,
): string {
    if (bucketCount === 24) {
        const hour = new Date(Number(bucket.startSecs) * 1000).getHours();
        return `${hour}`;
    }
    return bucket.label;
}

export function performanceChartTooltipLines(bucket: PerformanceChartBucket): string[] {
    const lines = [bucket.label];
    if (bucket.questions > 0 && bucket.accuracy !== undefined) {
        lines.push(`Accuracy: ${ratioToPercent(bucket.accuracy)}%`);
    }
    lines.push(`Correct: ${bucket.correct}`);
    lines.push(`Incorrect: ${bucket.incorrect}`);
    lines.push(`Questions: ${bucket.questions}`);
    return lines;
}

export function performanceChartTooltip(bucket: PerformanceChartBucket): string {
    return performanceChartTooltipLines(bucket).join("\n");
}

export function performanceChartLineSegments(
    buckets: PerformanceChartBucket[],
): PerformanceChartBucket[][] {
    const segments: PerformanceChartBucket[][] = [];
    let current: PerformanceChartBucket[] = [];

    for (const bucket of buckets) {
        if (bucket.questions > 0 && bucket.accuracy !== undefined) {
            current.push(bucket);
            continue;
        }
        if (current.length > 0) {
            segments.push(current);
            current = [];
        }
    }

    if (current.length > 0) {
        segments.push(current);
    }

    return segments;
}

export function performanceChartAccuracyPercent(bucket: PerformanceChartBucket): number {
    if (bucket.accuracy === undefined) {
        return 0;
    }
    return ratioToPercent(bucket.accuracy);
}
