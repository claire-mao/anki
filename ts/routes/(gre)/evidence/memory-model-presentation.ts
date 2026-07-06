// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    MemoryCalibrationBin,
    MemoryEvalResponse,
} from "@generated/anki/brainlift_pb";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";

export const MEMORY_MODEL_EMPTY_MESSAGE = EVIDENCE_INSUFFICIENT_MESSAGE;
export const MEMORY_CALIBRATION_METRIC_NAME = "Brier score";

export type CalibrationChartPoint = {
    predictedMean: number;
    outcomeMean: number;
    count: number;
};

export type MemoryModelPresentation =
    | {
          available: false;
          emptyMessage: typeof MEMORY_MODEL_EMPTY_MESSAGE;
      }
    | {
          available: true;
          modelName: string;
          fsrs: string;
          heldOutReviews: string;
          calibrationMetric: string;
          brierScore: string;
          logLoss: string;
          calibrationStatus: string;
          calibrationCurve: CalibrationChartPoint[];
      };

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null;
}

function readNumber(value: unknown): number | undefined {
    return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function readBoolean(value: unknown): boolean | undefined {
    return typeof value === "boolean" ? value : undefined;
}

function readString(value: unknown): string | undefined {
    return typeof value === "string" ? value : undefined;
}

function formatEvalMetric(value: number | undefined): string {
    return value === undefined ? "—" : value.toFixed(4);
}

function calibrationPoints(
    bins: MemoryCalibrationBin[] | undefined,
): CalibrationChartPoint[] {
    if (!bins) {
        return [];
    }
    return bins
        .filter((bin) => bin.count > 0)
        .map((bin) => ({
            predictedMean: bin.predictedMean,
            outcomeMean: bin.outcomeMean,
            count: bin.count,
        }));
}

function readCalibrationBins(value: unknown): MemoryCalibrationBin[] | null {
    if (!Array.isArray(value)) {
        return null;
    }
    const bins: MemoryCalibrationBin[] = [];
    for (const entry of value) {
        if (!isRecord(entry)) {
            return null;
        }
        const predictedMean = readNumber(entry.predicted_mean);
        const outcomeMean = readNumber(entry.outcome_mean);
        const count = readNumber(entry.count);
        if (
            predictedMean === undefined
            || outcomeMean === undefined
            || count === undefined
        ) {
            return null;
        }
        bins.push({
            binLow: readNumber(entry.bin_low) ?? 0,
            binHigh: readNumber(entry.bin_high) ?? 0,
            predictedMean,
            outcomeMean,
            count,
        } as MemoryCalibrationBin);
    }
    return bins;
}

export function parseGreAtlasEvalMemoryJson(json: string): MemoryEvalResponse | null {
    let parsed: unknown;
    try {
        parsed = JSON.parse(json);
    } catch {
        return null;
    }
    if (!isRecord(parsed) || !isRecord(parsed.memory)) {
        return null;
    }

    const memory = parsed.memory;
    const modelVersion = readString(memory.model_version);
    const fsrsEnabled = readBoolean(memory.fsrs_enabled);
    const heldOutReviewCount = readNumber(memory.held_out_review_count);
    const sufficientData = readBoolean(memory.sufficient_data);
    const assessment = readString(memory.assessment);
    const calibrationCurve = readCalibrationBins(memory.calibration_curve);

    if (
        modelVersion === undefined
        || fsrsEnabled === undefined
        || heldOutReviewCount === undefined
        || sufficientData === undefined
        || assessment === undefined
        || calibrationCurve === null
    ) {
        return null;
    }

    return {
        modelVersion,
        fsrsEnabled,
        heldOutReviewCount,
        sufficientData,
        brierScore: readNumber(memory.brier_score),
        logLoss: readNumber(memory.log_loss),
        assessment,
        calibrationCurve,
        computedAtMillis: 0n,
    } as MemoryEvalResponse;
}

export function presentMemoryModel(
    response: MemoryEvalResponse | null | undefined,
): MemoryModelPresentation {
    if (!response?.sufficientData || response.heldOutReviewCount === 0) {
        return {
            available: false,
            emptyMessage: MEMORY_MODEL_EMPTY_MESSAGE,
        };
    }

    return {
        available: true,
        modelName: response.modelVersion,
        fsrs: response.fsrsEnabled ? "Enabled" : "Disabled",
        heldOutReviews: String(response.heldOutReviewCount),
        calibrationMetric: MEMORY_CALIBRATION_METRIC_NAME,
        brierScore: formatEvalMetric(response.brierScore),
        logLoss: formatEvalMetric(response.logLoss),
        calibrationStatus: response.assessment,
        calibrationCurve: calibrationPoints(response.calibrationCurve),
    };
}
