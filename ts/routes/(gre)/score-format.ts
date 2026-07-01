// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceScore } from "@generated/anki/brainlift_pb";

export function formatPercent(value: number): string {
    return `${Math.round(value)}%`;
}

export function formatRatio(ratio: number): string {
    return formatPercent(ratio * 100);
}

export function formatRange(low: number | undefined, high: number | undefined): string | null {
    if (low === undefined || high === undefined) {
        return null;
    }
    return `${formatPercent(low)}–${formatPercent(high)}`;
}

export function formatResponseTimeMs(ms: number): string {
    return `${(ms / 1000).toFixed(1)}s`;
}

export function performanceSummary(score: PerformanceScore): string {
    if (score.sufficientData && score.value !== undefined) {
        const range = formatRange(score.valueLow, score.valueHigh);
        if (range) {
            return `${formatPercent(score.value)} (${range})`;
        }
        return formatPercent(score.value);
    }
    return score.abstainReason;
}
