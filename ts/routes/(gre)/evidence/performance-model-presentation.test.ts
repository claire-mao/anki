// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceEvalResponse } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";
import {
    PERFORMANCE_MODEL_EMPTY_MESSAGE,
    presentPerformanceModel,
} from "./performance-model-presentation";

function evalResponse(
    overrides: Partial<PerformanceEvalResponse> = {},
): PerformanceEvalResponse {
    return {
        modelVersion: "performance_v1",
        sufficientData: false,
        assessment: "Requires more held-out attempts.",
        computedAtMillis: 1n,
        ...overrides,
    } as PerformanceEvalResponse;
}

describe("presentPerformanceModel", () => {
    test("shows empty message when held-out evaluation is unavailable", () => {
        const model = presentPerformanceModel(evalResponse());
        expect(model.available).toBe(false);
        if (model.available) {
            throw new Error("expected unavailable model");
        }
        expect(model.emptyMessage).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
        expect(PERFORMANCE_MODEL_EMPTY_MESSAGE).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
    });

    test("surfaces held-out metrics without estimating missing values", () => {
        const model = presentPerformanceModel(
            evalResponse({
                sufficientData: true,
                assessment: "Held-out performance on 6 attempts: 66.7% correct.",
                test: {
                    attemptCount: 6,
                    correctCount: 4,
                    accuracy: 66.7,
                    accuracyCi: { level: 0.95, low: 39.4, high: 86.1, method: "wilson_score" },
                    topicAccuracy: [
                        {
                            topicId: "gre::quant::algebra",
                            displayName: "Algebra",
                            attemptCount: 4,
                            correctCount: 3,
                            accuracy: 75,
                        },
                    ],
                    confusion: {
                        truePositive: 3,
                        falsePositive: 1,
                        trueNegative: 1,
                        falseNegative: 1,
                    },
                },
            } as PerformanceEvalResponse),
        );

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.heldOutQuestions).toBe(6);
        expect(model.accuracy).toBe("67%");
        expect(model.confidenceInterval).toBe("39%–86%");
        expect(model.questionsEvaluated).toBe(6);
        expect(model.topicRows).toHaveLength(1);
        expect(model.topicRows[0]?.displayName).toBe("Algebra");
        expect(model.confusion.total).toBe(6);
    });
});
