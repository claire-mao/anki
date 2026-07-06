// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";
import {
    parseEvalReportSections,
    presentEvalAbstentionSummary,
    presentReadinessModel,
    presentStudyFeatureExperiment,
} from "./eval-report-presentation";

const SAMPLE_EVAL_REPORT = {
    calibration: {
        total_predictions: 0,
        resolved_outcomes: 0,
        held_out_count: 0,
        sufficient_data: false,
        well_calibrated: false,
        assessment:
            "Calibration requires at least 5 held-out predictions with observed outcomes (current: 0). Model confidence is unverified.",
        brier_score: null,
        mean_absolute_error: null,
        bins: [],
    },
    abstention: {
        abstention_rate: 1.0,
        memory_abstaining: true,
        performance_abstaining: true,
        readiness_abstaining: true,
        unmet_requirements: [
            {
                id: "practice_attempts",
                label: "GRE practice attempts",
                status: "4 of 50 practice attempts",
                met: false,
            },
        ],
    },
    ablation: {
        focus_topic_count: 3,
        collection: {
            label: "Collection",
            sufficient_data: true,
            assessment:
                "GRE Atlas priority sum 1.800 vs random 0.390 vs vanilla 1.650; coverage winners: GRE Atlas priority; readiness winners: n/a",
            policies: [
                {
                    policy_label: "GRE Atlas priority",
                    expected_learning_gain: 1.8,
                    topic_coverage_gain: 0.154,
                    readiness_improvement: null,
                },
                {
                    policy_label: "Random topic order",
                    expected_learning_gain: 0.39,
                    topic_coverage_gain: 0.1222,
                    readiness_improvement: null,
                },
            ],
            winners: {
                expected_learning_gain: "GRE Atlas priority",
                topic_coverage_gain: "GRE Atlas priority",
                readiness_improvement: "n/a",
            },
        },
    },
};

describe("parseEvalReportSections", () => {
    test("parses calibration, abstention, and ablation from a full eval report", () => {
        const sections = parseEvalReportSections(JSON.stringify(SAMPLE_EVAL_REPORT));
        expect(sections.calibration?.heldOutCount).toBe(0);
        expect(sections.abstention?.abstentionRate).toBe(1);
        expect(sections.ablation?.focusTopicCount).toBe(3);
    });

    test("returns null sections for malformed JSON", () => {
        const sections = parseEvalReportSections("{");
        expect(sections.calibration).toBeNull();
        expect(sections.abstention).toBeNull();
        expect(sections.ablation).toBeNull();
    });
});

describe("presentReadinessModel", () => {
    test("shows insufficient data when held-out calibration is unavailable", () => {
        const model = presentReadinessModel(
            parseEvalReportSections(JSON.stringify(SAMPLE_EVAL_REPORT)).calibration,
        );
        expect(model.available).toBe(false);
        if (model.available) {
            throw new Error("expected unavailable model");
        }
        expect(model.emptyMessage).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
    });

    test("surfaces held-out readiness metrics without inventing values", () => {
        const model = presentReadinessModel({
            heldOutCount: 8,
            sufficientData: true,
            wellCalibrated: true,
            assessment: "Held-out readiness calibration on 8 predictions.",
            brierScore: 0.0812,
            meanAbsoluteError: 4.8,
            bins: [{ predictedMean: 0.6, outcomeMean: 0.5, count: 3 }],
        });

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.heldOutPredictions).toBe("8");
        expect(model.brierScore).toBe("0.0812");
        expect(model.meanAbsoluteError).toBe("4.8000");
        expect(model.calibrationCurve).toHaveLength(1);
    });
});

describe("presentEvalAbstentionSummary", () => {
    test("shows abstention evidence from the eval report", () => {
        const model = presentEvalAbstentionSummary(
            parseEvalReportSections(JSON.stringify(SAMPLE_EVAL_REPORT)).abstention,
        );
        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.abstentionRate).toBe("100%");
        expect(model.unmetRequirements).toHaveLength(1);
    });
});

describe("presentStudyFeatureExperiment", () => {
    test("shows ablation results from the eval report", () => {
        const sections = parseEvalReportSections(JSON.stringify(SAMPLE_EVAL_REPORT));
        const model = presentStudyFeatureExperiment(sections.ablation);
        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.scenarioLabel).toBe("Collection");
        expect(model.policyResults.length).toBeGreaterThan(0);
        expect(model.winners.some((winner) => winner.value === "GRE Atlas priority")).toBe(true);
    });

    test("shows insufficient data when ablation results are unavailable", () => {
        const model = presentStudyFeatureExperiment(null);
        expect(model.available).toBe(false);
        if (model.available) {
            throw new Error("expected unavailable model");
        }
        expect(model.emptyMessage).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
    });
});
