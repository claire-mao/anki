// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { MemoryEvalResponse } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";
import {
    MEMORY_CALIBRATION_METRIC_NAME,
    MEMORY_MODEL_EMPTY_MESSAGE,
    parseGreAtlasEvalMemoryJson,
    presentMemoryModel,
} from "./memory-model-presentation";

function memoryEvalResponse(
    overrides: Partial<MemoryEvalResponse> = {},
): MemoryEvalResponse {
    return {
        modelVersion: "fsrs",
        fsrsEnabled: true,
        heldOutReviewCount: 0,
        sufficientData: false,
        assessment: "FSRS is disabled; memory calibration requires FSRS scheduling.",
        calibrationCurve: [],
        computedAtMillis: 0n,
        ...overrides,
    } as MemoryEvalResponse;
}

describe("parseGreAtlasEvalMemoryJson", () => {
    test("parses the memory section from a full eval report", () => {
        const parsed = parseGreAtlasEvalMemoryJson(
            JSON.stringify({
                memory: {
                    model_version: "fsrs",
                    fsrs_enabled: true,
                    held_out_review_count: 12,
                    sufficient_data: true,
                    brier_score: 0.0812,
                    log_loss: 0.2451,
                    assessment: "Held-out FSRS calibration on 12 reviews: Brier score 0.0812.",
                    calibration_curve: [
                        {
                            bin_low: 0.0,
                            bin_high: 0.1,
                            predicted_mean: 0.05,
                            outcome_mean: 0.0,
                            count: 2,
                        },
                    ],
                },
            }),
        );

        expect(parsed?.heldOutReviewCount).toBe(12);
        expect(parsed?.brierScore).toBe(0.0812);
        expect(parsed?.logLoss).toBe(0.2451);
        expect(parsed?.calibrationCurve).toHaveLength(1);
    });

    test("rejects malformed JSON", () => {
        expect(parseGreAtlasEvalMemoryJson("{")).toBeNull();
    });
});

describe("presentMemoryModel", () => {
    test("shows the required empty message when held-out data is unavailable", () => {
        const model = presentMemoryModel(memoryEvalResponse());
        expect(model.available).toBe(false);
        if (model.available) {
            throw new Error("expected unavailable model");
        }
        expect(model.emptyMessage).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
        expect(MEMORY_MODEL_EMPTY_MESSAGE).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
    });

    test("surfaces held-out metrics without inventing missing values", () => {
        const model = presentMemoryModel(
            memoryEvalResponse({
                sufficientData: true,
                heldOutReviewCount: 12,
                brierScore: 0.0812,
                logLoss: 0.2451,
                assessment: "Held-out FSRS calibration on 12 reviews: Brier score 0.0812.",
                calibrationCurve: [
                    {
                        binLow: 0,
                        binHigh: 0.1,
                        predictedMean: 0.05,
                        outcomeMean: 0,
                        count: 2,
                    },
                ],
            } as MemoryEvalResponse),
        );

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.modelName).toBe("fsrs");
        expect(model.fsrs).toBe("Enabled");
        expect(model.heldOutReviews).toBe("12");
        expect(model.calibrationMetric).toBe(MEMORY_CALIBRATION_METRIC_NAME);
        expect(model.brierScore).toBe("0.0812");
        expect(model.logLoss).toBe("0.2451");
        expect(model.calibrationStatus).toContain("Brier score 0.0812");
        expect(model.calibrationCurve).toHaveLength(1);
    });

    test("shows dashes for metrics that are absent in the eval payload", () => {
        const model = presentMemoryModel(
            memoryEvalResponse({
                sufficientData: true,
                heldOutReviewCount: 6,
                assessment: "Held-out FSRS calibration on 6 reviews.",
                calibrationCurve: [],
            } as MemoryEvalResponse),
        );

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.brierScore).toBe("—");
        expect(model.logLoss).toBe("—");
    });
});
