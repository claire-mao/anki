// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    DashboardTopicInsight,
    EstimatedGreScore,
    MemoryScore,
    PerformanceAttempt,
    PerformanceScore,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";
import type { TopicMasteryEntry, TopicMasterySummary } from "@generated/anki/stats_pb";

import { rollingAccuracySeries } from "./indicator-utils";

const STORAGE_KEY = "brainlift-gre-metric-snapshot";

export type GreTopicSnapshot = {
    displayName: string;
    retrievability: number;
    masteredCards: number;
    practiceAccuracy?: number;
};

export type GreMetricSnapshot = {
    version: 1;
    savedAtMillis: number;
    readinessScore?: number;
    readinessUnlocked: boolean;
    estimatedGreScore?: number;
    estimatedGreUnlocked: boolean;
    quantScore?: number;
    verbalScore?: number;
    topicMasteryPercent?: number;
    topicMasteryUnlocked: boolean;
    confidenceLevel: string;
    calibrationWellCalibrated: boolean;
    calibrationNote: string;
    memoryValue?: number;
    performanceValue?: number;
    coverageRatio: number;
    studiedCards: number;
    masteredCards: number;
    practiceAttempts: number;
    recentAccuracy?: number;
    requirements: Record<string, { label: string; met: boolean }>;
    topics: Record<string, GreTopicSnapshot>;
};

export type MetricSnapshotInput = {
    memory: MemoryScore;
    performance: PerformanceScore;
    readiness: ReadinessScore;
    estimatedGre: EstimatedGreScore;
    topicMasterySummary?: TopicMasterySummary;
    topicMasteryTopics?: TopicMasteryEntry[];
    topicInsights?: DashboardTopicInsight[];
    recentActivity?: PerformanceAttempt[];
};

function topicMapFromInsights(insights: DashboardTopicInsight[] | undefined): Record<string, GreTopicSnapshot> {
    const topics: Record<string, GreTopicSnapshot> = {};
    for (const topic of insights ?? []) {
        topics[topic.topicId] = {
            displayName: topic.displayName,
            retrievability: topic.memoryScore ?? 0,
            masteredCards: topic.studiedCards,
            practiceAccuracy: topic.practiceAccuracy,
        };
    }
    return topics;
}

function mergeTopicMastery(
    topics: Record<string, GreTopicSnapshot>,
    entries: TopicMasteryEntry[] | undefined,
): Record<string, GreTopicSnapshot> {
    const merged = { ...topics };
    for (const entry of entries ?? []) {
        const existing = merged[entry.topicId];
        merged[entry.topicId] = {
            displayName: entry.displayName,
            retrievability: entry.avgRetrievability * 100,
            masteredCards: entry.masteredCards,
            practiceAccuracy: existing?.practiceAccuracy,
        };
    }
    return merged;
}

function requirementSnapshot(
    memory: MemoryScore,
    performance: PerformanceScore,
    readiness: ReadinessScore,
): Record<string, { label: string; met: boolean }> {
    const requirements: Record<string, { label: string; met: boolean }> = {};
    for (const req of [
        ...memory.abstentionRequirements,
        ...performance.abstentionRequirements,
        ...readiness.abstentionRequirements,
    ]) {
        requirements[req.id] = { label: req.label, met: req.met };
    }
    return requirements;
}

export function extractGreMetricSnapshot(input: MetricSnapshotInput): GreMetricSnapshot {
    const { memory, performance, readiness, estimatedGre, topicMasterySummary } = input;
    const trend = rollingAccuracySeries(input.recentActivity ?? []);
    const topics = mergeTopicMastery(
        topicMapFromInsights(input.topicInsights),
        input.topicMasteryTopics,
    );

    return {
        version: 1,
        savedAtMillis: Date.now(),
        readinessScore: readiness.projectedScore,
        readinessUnlocked: readiness.sufficientData && readiness.projectedScore !== undefined,
        estimatedGreScore: estimatedGre.combinedScore,
        estimatedGreUnlocked: estimatedGre.combinedScore !== undefined,
        quantScore: estimatedGre.quantScore,
        verbalScore: estimatedGre.verbalScore,
        topicMasteryPercent:
            topicMasterySummary?.sufficientData && topicMasterySummary.overallAvgRetrievability !== undefined
                ? topicMasterySummary.overallAvgRetrievability * 100
                : undefined,
        topicMasteryUnlocked: topicMasterySummary?.sufficientData ?? false,
        confidenceLevel: readiness.confidenceLevel || "",
        calibrationWellCalibrated: readiness.calibrationWellCalibrated,
        calibrationNote: readiness.calibrationNote || "",
        memoryValue: memory.value,
        performanceValue: performance.value,
        coverageRatio: readiness.coverageRatio,
        studiedCards: memory.studiedCards,
        masteredCards: topicMasterySummary?.masteredCards ?? 0,
        practiceAttempts: performance.attemptCount,
        recentAccuracy: trend.length > 0 ? trend[trend.length - 1] : undefined,
        requirements: requirementSnapshot(memory, performance, readiness),
        topics,
    };
}

export function loadGreMetricSnapshot(): GreMetricSnapshot | null {
    if (typeof window === "undefined") {
        return null;
    }
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) {
            return null;
        }
        const parsed = JSON.parse(raw) as GreMetricSnapshot;
        if (parsed.version !== 1) {
            return null;
        }
        return parsed;
    } catch {
        return null;
    }
}

export function saveGreMetricSnapshot(snapshot: GreMetricSnapshot): void {
    if (typeof window === "undefined") {
        return;
    }
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
}
