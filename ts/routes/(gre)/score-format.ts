// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

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

export function formatGreScoreRange(low: number | undefined, high: number | undefined): string | null {
    if (low === undefined || high === undefined) {
        return null;
    }
    if (Math.round(low) === Math.round(high)) {
        return null;
    }
    return `${Math.round(low)}–${Math.round(high)}`;
}
