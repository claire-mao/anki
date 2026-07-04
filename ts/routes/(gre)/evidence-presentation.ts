// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    DashboardCoverage,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
} from "@generated/anki/brainlift_pb";

import { formatRatio } from "./score-format";

/** A single "the estimate is built from …" evidence line. */
export type EvidenceItem = {
    label: string;
    /** Whether this evidence has actually been collected yet. */
    met: boolean;
};

/** One-line summary of evidence backing the estimate. */
export function buildEvidenceSummary(
    memory: MemoryScore,
    performance: PerformanceScore,
    coverage: DashboardCoverage,
): string {
    const parts: string[] = [];
    if (memory.studiedCards > 0) {
        parts.push(`${memory.studiedCards} reviews`);
    }
    if (performance.attemptCount > 0) {
        parts.push(`${performance.attemptCount} practice`);
    }
    if (coverage.weightedRatio > 0) {
        parts.push(`${formatRatio(coverage.weightedRatio)} coverage`);
    }
    if (parts.length === 0) {
        return "Collect study and practice data to build your estimate.";
    }
    return `Based on ${parts.join(", ")}.`;
}

/**
 * The concrete evidence backing the estimated GRE score. This reinforces the
 * project's core claim — the estimate is grounded in observed study data, not a
 * bare number. Calibration error is only surfaced when there is enough held-out
 * prediction history to compute it honestly.
 */
export function buildEvidenceItems(
    memory: MemoryScore,
    performance: PerformanceScore,
    coverage: DashboardCoverage,
    calibration?: ReadinessCalibrationStats,
): EvidenceItem[] {
    const items: EvidenceItem[] = [
        {
            label: `${memory.studiedCards} flashcards reviewed`,
            met: memory.studiedCards > 0,
        },
        {
            label: `${performance.attemptCount} practice questions`,
            met: performance.attemptCount > 0,
        },
        {
            label: `${formatRatio(coverage.weightedRatio)} GRE topic coverage`,
            met: coverage.weightedRatio > 0,
        },
    ];

    if (
        calibration?.sufficientData
        && calibration.meanAbsoluteError !== undefined
    ) {
        items.push({
            label: `Past estimate error: ±${calibration.meanAbsoluteError.toFixed(1)} points`,
            met: true,
        });
    }

    return items;
}
