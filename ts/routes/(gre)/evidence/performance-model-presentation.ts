// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    PerformanceConfusionMatrix,
    PerformanceEvalMetrics,
    PerformanceEvalResponse,
    PerformanceTopicAccuracy,
} from "@generated/anki/brainlift_pb";

import { formatPercent, formatRange } from "../score-format";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";

export const PERFORMANCE_MODEL_EMPTY_MESSAGE = EVIDENCE_INSUFFICIENT_MESSAGE;

export type PerformanceTopicAccuracyRow = {
    topicId: string;
    displayName: string;
    attemptCount: number;
    correctCount: number;
    accuracyPercent: number;
    tooltip: string;
};

export type PerformanceConfusionMatrixPresentation = {
    truePositive: number;
    falsePositive: number;
    trueNegative: number;
    falseNegative: number;
    total: number;
};

export type PerformanceModelPresentation =
    | {
          available: false;
          emptyMessage: typeof PERFORMANCE_MODEL_EMPTY_MESSAGE;
      }
    | {
          available: true;
          heldOutQuestions: number;
          accuracy: string;
          confidenceInterval: string;
          questionsEvaluated: number;
          assessment: string;
          topicRows: PerformanceTopicAccuracyRow[];
          confusion: PerformanceConfusionMatrixPresentation;
      };

function topicRow(topic: PerformanceTopicAccuracy): PerformanceTopicAccuracyRow {
    const accuracyPercent = topic.accuracy;
    return {
        topicId: topic.topicId,
        displayName: topic.displayName || topic.topicId,
        attemptCount: topic.attemptCount,
        correctCount: topic.correctCount,
        accuracyPercent,
        tooltip: `${topic.correctCount}/${topic.attemptCount} correct · ${formatPercent(accuracyPercent)}`,
    };
}

function presentConfusion(
    matrix: PerformanceConfusionMatrix | undefined,
): PerformanceConfusionMatrixPresentation {
    const truePositive = matrix?.truePositive ?? 0;
    const falsePositive = matrix?.falsePositive ?? 0;
    const trueNegative = matrix?.trueNegative ?? 0;
    const falseNegative = matrix?.falseNegative ?? 0;
    return {
        truePositive,
        falsePositive,
        trueNegative,
        falseNegative,
        total: truePositive + falsePositive + trueNegative + falseNegative,
    };
}

function presentMetrics(metrics: PerformanceEvalMetrics | undefined): PerformanceModelPresentation {
    if (!metrics || metrics.attemptCount === 0) {
        return {
            available: false,
            emptyMessage: PERFORMANCE_MODEL_EMPTY_MESSAGE,
        };
    }

    const confidenceInterval =
        formatRange(metrics.accuracyCi?.low, metrics.accuracyCi?.high) ?? "—";

    return {
        available: true,
        heldOutQuestions: metrics.attemptCount,
        accuracy: formatPercent(metrics.accuracy),
        confidenceInterval,
        questionsEvaluated: metrics.attemptCount,
        assessment: "",
        topicRows: metrics.topicAccuracy.map(topicRow),
        confusion: presentConfusion(metrics.confusion),
    };
}

export function presentPerformanceModel(
    response: PerformanceEvalResponse | null | undefined,
): PerformanceModelPresentation {
    if (!response || !response.sufficientData) {
        return {
            available: false,
            emptyMessage: PERFORMANCE_MODEL_EMPTY_MESSAGE,
        };
    }

    const model = presentMetrics(response.test);
    if (!model.available) {
        return model;
    }

    return {
        ...model,
        assessment: response.assessment,
    };
}

export function performanceTopicAccuracyChartHeight(rowCount: number): number {
    const rowHeight = 36;
    const padding = 24;
    return Math.max(160, rowCount * rowHeight + padding);
}
