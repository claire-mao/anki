// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { ratioToPercent } from "../indicator-utils";
import { formatRatio } from "../score-format";

export const AI_EVAL_BASELINE_KEYWORD_ID = "baseline_keyword";
export const AI_EVAL_BASELINE_VECTOR_ID = "baseline_vector_tfidf";
export const AI_EVAL_BRAINLIFT_AI_ID = "ai_generation_pipeline";

export const AI_EVAL_OFFLINE_MESSAGE = "Offline mode.";
export const AI_EVAL_INSUFFICIENT_MESSAGE = "Insufficient evaluation data.";

export type AiEvalSystemMetrics = {
    system_id: string;
    accuracy: number;
};

export type AiEvalReportJson = {
    confidence_threshold: number;
    held_out_quality: {
        evaluated_count: number;
        accuracy: number;
        wrong_answer_rate: number;
    };
    acceptance_criteria: {
        acceptance_cutoff: number;
    };
    verdict: {
        passed: boolean;
    };
    systems: AiEvalSystemMetrics[];
};

export type AiEvaluationBaselineBar = {
    label: string;
    accuracyPercent: number;
};

export type AiEvaluationPresentation =
    | { mode: "offline" }
    | { mode: "insufficient" }
    | {
          mode: "ready";
          heldOutQuestions: string;
          accuracy: string;
          wrongAnswerRate: string;
          confidenceThreshold: string;
          releaseCutoff: string;
          verdict: "PASS" | "FAIL";
          baselines: AiEvaluationBaselineBar[];
      };

const BASELINE_SPECS: readonly { id: string; label: string }[] = [
    { id: AI_EVAL_BASELINE_KEYWORD_ID, label: "Keyword Search" },
    { id: AI_EVAL_BASELINE_VECTOR_ID, label: "Vector Search" },
    { id: AI_EVAL_BRAINLIFT_AI_ID, label: "BrainLift AI" },
];

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null;
}

function readNumber(value: unknown): number | null {
    return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function readBoolean(value: unknown): boolean | null {
    return typeof value === "boolean" ? value : null;
}

function formatThreshold(value: number): string {
    return value.toFixed(2);
}

function formatAccuracyRatio(ratio: number): string {
    return formatRatio(ratio);
}

function formatWrongAnswerRate(ratio: number): string {
    return formatRatio(ratio);
}

function readSystems(value: unknown): AiEvalSystemMetrics[] | null {
    if (!Array.isArray(value)) {
        return null;
    }
    const systems: AiEvalSystemMetrics[] = [];
    for (const entry of value) {
        if (!isRecord(entry)) {
            return null;
        }
        const systemId = entry.system_id;
        const accuracy = readNumber(entry.accuracy);
        if (typeof systemId !== "string" || accuracy === null) {
            return null;
        }
        systems.push({ system_id: systemId, accuracy });
    }
    return systems;
}

export function parseAiEvalReportJson(json: string): AiEvalReportJson | null {
    let parsed: unknown;
    try {
        parsed = JSON.parse(json);
    } catch {
        return null;
    }
    if (!isRecord(parsed)) {
        return null;
    }

    const confidenceThreshold = readNumber(parsed.confidence_threshold);
    const heldOut = parsed.held_out_quality;
    const acceptance = parsed.acceptance_criteria;
    const verdict = parsed.verdict;
    const systems = readSystems(parsed.systems);

    if (
        confidenceThreshold === null
        || !isRecord(heldOut)
        || !isRecord(acceptance)
        || !isRecord(verdict)
        || systems === null
    ) {
        return null;
    }

    const evaluatedCount = readNumber(heldOut.evaluated_count);
    const heldOutAccuracy = readNumber(heldOut.accuracy);
    const wrongAnswerRate = readNumber(heldOut.wrong_answer_rate);
    const acceptanceCutoff = readNumber(acceptance.acceptance_cutoff);
    const passed = readBoolean(verdict.passed);

    if (
        evaluatedCount === null
        || heldOutAccuracy === null
        || wrongAnswerRate === null
        || acceptanceCutoff === null
        || passed === null
    ) {
        return null;
    }

    return {
        confidence_threshold: confidenceThreshold,
        held_out_quality: {
            evaluated_count: evaluatedCount,
            accuracy: heldOutAccuracy,
            wrong_answer_rate: wrongAnswerRate,
        },
        acceptance_criteria: {
            acceptance_cutoff: acceptanceCutoff,
        },
        verdict: { passed },
        systems,
    };
}

function baselineBars(systems: AiEvalSystemMetrics[]): AiEvaluationBaselineBar[] | null {
    const byId = new Map(systems.map((system) => [system.system_id, system]));
    const bars: AiEvaluationBaselineBar[] = [];

    for (const spec of BASELINE_SPECS) {
        const system = byId.get(spec.id);
        if (!system) {
            return null;
        }
        bars.push({
            label: spec.label,
            accuracyPercent: ratioToPercent(system.accuracy),
        });
    }

    return bars;
}

export function presentAiEvaluation(input: {
    aiEnabled: boolean;
    reportJson: string;
}): AiEvaluationPresentation {
    if (!input.aiEnabled) {
        return { mode: "offline" };
    }

    const report = parseAiEvalReportJson(input.reportJson);
    if (!report) {
        return { mode: "insufficient" };
    }

    const baselines = baselineBars(report.systems);
    if (!baselines) {
        return { mode: "insufficient" };
    }

    return {
        mode: "ready",
        heldOutQuestions: String(report.held_out_quality.evaluated_count),
        accuracy: formatAccuracyRatio(report.held_out_quality.accuracy),
        wrongAnswerRate: formatWrongAnswerRate(report.held_out_quality.wrong_answer_rate),
        confidenceThreshold: formatThreshold(report.confidence_threshold),
        releaseCutoff: formatThreshold(report.acceptance_criteria.acceptance_cutoff),
        verdict: report.verdict.passed ? "PASS" : "FAIL",
        baselines,
    };
}
