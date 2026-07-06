// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import {
    AI_EVAL_INSUFFICIENT_MESSAGE,
    AI_EVAL_OFFLINE_MESSAGE,
    parseAiEvalReportJson,
    presentAiEvaluation,
} from "./ai-evaluation-presentation";

const SAMPLE_REPORT = {
    confidence_threshold: 0.55,
    held_out_quality: {
        evaluated_count: 50,
        approved_count: 50,
        wrong_answer_count: 0,
        accuracy: 1.0,
        wrong_answer_rate: 0.0,
    },
    acceptance_criteria: {
        min_accuracy: 0.95,
        max_wrong_answer_rate: 0.0,
        acceptance_cutoff: 0.55,
    },
    verdict: {
        passed: true,
        accuracy: 1.0,
        wrong_answer_rate: 0.0,
        failure_reasons: [],
    },
    systems: [
        { system_id: "baseline_keyword", accuracy: 0.42 },
        { system_id: "baseline_bm25", accuracy: 0.6 },
        { system_id: "baseline_vector_tfidf", accuracy: 0.52 },
        { system_id: "ai_retrieval", accuracy: 0.62 },
        { system_id: "ai_generation_pipeline", accuracy: 0.62 },
    ],
};

describe("parseAiEvalReportJson", () => {
    it("parses a real-shaped AI eval report", () => {
        const report = parseAiEvalReportJson(JSON.stringify(SAMPLE_REPORT));
        expect(report?.held_out_quality.evaluated_count).toBe(50);
        expect(report?.verdict.passed).toBe(true);
        expect(report?.systems).toHaveLength(5);
    });

    it("rejects malformed JSON", () => {
        expect(parseAiEvalReportJson("{")).toBeNull();
    });
});

describe("presentAiEvaluation", () => {
    it("shows offline mode when AI is disabled", () => {
        const model = presentAiEvaluation({
            aiEnabled: false,
            reportJson: JSON.stringify(SAMPLE_REPORT),
        });
        expect(model.mode).toBe("offline");
    });

    it("shows held-out metrics and baseline bars from real report data", () => {
        const model = presentAiEvaluation({
            aiEnabled: true,
            reportJson: JSON.stringify(SAMPLE_REPORT),
        });
        expect(model).toEqual({
            mode: "ready",
            heldOutQuestions: "50",
            accuracy: "100%",
            wrongAnswerRate: "0%",
            confidenceThreshold: "0.55",
            releaseCutoff: "0.55",
            verdict: "PASS",
            baselines: [
                { label: "Keyword Search", accuracyPercent: 42 },
                { label: "Vector Search", accuracyPercent: 52 },
                { label: "BrainLift AI", accuracyPercent: 62 },
            ],
        });
    });

    it("shows insufficient data when baseline systems are missing", () => {
        const model = presentAiEvaluation({
            aiEnabled: true,
            reportJson: JSON.stringify({
                ...SAMPLE_REPORT,
                systems: [{ system_id: "baseline_keyword", accuracy: 0.42 }],
            }),
        });
        expect(model.mode).toBe("insufficient");
    });
});

describe("AI evaluation copy", () => {
    it("uses the required empty-state messages", () => {
        expect(AI_EVAL_OFFLINE_MESSAGE).toBe("Offline mode.");
        expect(AI_EVAL_INSUFFICIENT_MESSAGE).toBe("Insufficient evaluation data.");
    });
});
