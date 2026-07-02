// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    AbstentionRequirement,
    DashboardTopicInsight,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
    TopicDetailsResponse,
} from "@generated/anki/brainlift_pb";

import { topicDetailsPath } from "./topic-link";

export type EvidencePillar = {
    id: string;
    label: string;
    met: boolean;
    status: string;
};

export type ImprovementItem = {
    id: string;
    label: string;
    detail?: string;
    href?: string;
};

export type PredictionExplainability = {
    basedOn: EvidencePillar[];
    needsImprovement: ImprovementItem[];
};

const PILLAR_MEMORY = "memory_retention";
const PILLAR_PRACTICE = "practice_accuracy";
const PILLAR_COVERAGE = "topic_coverage";

function requirementById(
    requirements: AbstentionRequirement[],
    id: string,
): AbstentionRequirement | undefined {
    return requirements.find((req) => req.id === id);
}

function memoryRetentionPillar(memory: MemoryScore | undefined): EvidencePillar {
    const studiedReq = memory
        ? requirementById(memory.abstentionRequirements, "studied_cards")
        : undefined;
    const fsrsReq = memory ? requirementById(memory.abstentionRequirements, "fsrs_enabled") : undefined;

    if (memory?.sufficientData) {
        return {
            id: PILLAR_MEMORY,
            label: "Memory retention",
            met: true,
            status: memory.detail || studiedReq?.status || "Memory evidence sufficient",
        };
    }

    const status =
        studiedReq?.status ||
        fsrsReq?.status ||
        memory?.abstainReason ||
        memory?.detail ||
        "Review more GRE flashcards";

    return {
        id: PILLAR_MEMORY,
        label: "Memory retention",
        met: false,
        status,
    };
}

function practiceAccuracyPillar(performance: PerformanceScore | undefined): EvidencePillar {
    const attemptsReq = performance
        ? requirementById(performance.abstentionRequirements, "practice_attempts")
        : undefined;

    if (performance?.sufficientData) {
        return {
            id: PILLAR_PRACTICE,
            label: "Practice accuracy",
            met: true,
            status: performance.detail || attemptsReq?.status || "Practice evidence sufficient",
        };
    }

    return {
        id: PILLAR_PRACTICE,
        label: "Practice accuracy",
        met: false,
        status:
            attemptsReq?.status ||
            performance?.abstainReason ||
            performance?.detail ||
            "Answer more GRE practice questions",
    };
}

function topicCoveragePillar(memory: MemoryScore | undefined, readiness: ReadinessScore | undefined): EvidencePillar {
    const coverageReq = memory
        ? requirementById(memory.abstentionRequirements, "topic_coverage")
        : undefined;

    if (coverageReq?.met) {
        return {
            id: PILLAR_COVERAGE,
            label: "Topic coverage",
            met: true,
            status: coverageReq.status,
        };
    }

    if (readiness && readiness.coverageRatio > 0 && coverageReq) {
        return {
            id: PILLAR_COVERAGE,
            label: "Topic coverage",
            met: false,
            status: coverageReq.status,
        };
    }

    return {
        id: PILLAR_COVERAGE,
        label: "Topic coverage",
        met: coverageReq?.met ?? false,
        status:
            coverageReq?.status ||
            readiness?.abstainReason ||
            "Review cards across more GRE topics",
    };
}

function weakTopicImprovements(
    weakTopics: DashboardTopicInsight[] | undefined,
    limit = 3,
): ImprovementItem[] {
    if (!weakTopics?.length) {
        return [];
    }
    return weakTopics.slice(0, limit).map((topic) => ({
        id: `weak-${topic.topicId}`,
        label: topic.displayName,
        detail: topic.reason || undefined,
        href: topicDetailsPath(topic.topicId),
    }));
}

function unmetRequirementImprovements(
    requirements: AbstentionRequirement[],
    excludeIds: Set<string>,
): ImprovementItem[] {
    return requirements
        .filter((req) => !req.met && !excludeIds.has(req.id))
        .map((req) => ({
            id: req.id,
            label: req.label,
            detail: req.status,
        }));
}

function calibrationImprovement(
    readiness: ReadinessScore | undefined,
    calibration: ReadinessCalibrationStats | undefined,
): ImprovementItem | null {
    if (readiness?.calibrationSufficientData && !readiness.calibrationWellCalibrated) {
        return {
            id: "calibration",
            label: "Calibration",
            detail: readiness.calibrationNote || "More resolved predictions needed for calibration.",
        };
    }
    if (calibration && calibration.resolvedOutcomes > 0 && !calibration.wellCalibrated) {
        return {
            id: "calibration",
            label: "Calibration",
            detail: calibration.assessment || "Prediction accuracy still stabilizing.",
        };
    }
    return null;
}

function lowConfidenceImprovement(readiness: ReadinessScore | undefined): ImprovementItem | null {
    if (!readiness?.sufficientData || readiness.projectedScore === undefined) {
        return null;
    }
    if (readiness.confidenceLevel === "low") {
        return {
            id: "confidence",
            label: "Prediction confidence",
            detail: "Add more review and practice evidence to narrow the estimate.",
        };
    }
    return null;
}

function mergeImprovements(...groups: ImprovementItem[][]): ImprovementItem[] {
    const seen = new Set<string>();
    const merged: ImprovementItem[] = [];
    for (const group of groups) {
        for (const item of group) {
            if (seen.has(item.id)) {
                continue;
            }
            seen.add(item.id);
            merged.push(item);
        }
    }
    return merged;
}

export function buildPredictionExplainability(input: {
    memory?: MemoryScore;
    performance?: PerformanceScore;
    readiness?: ReadinessScore;
    weakTopics?: DashboardTopicInsight[];
    requirements?: AbstentionRequirement[];
    calibration?: ReadinessCalibrationStats;
    weakTopicLimit?: number;
}): PredictionExplainability {
    const basedOn = [
        memoryRetentionPillar(input.memory),
        practiceAccuracyPillar(input.performance),
        topicCoveragePillar(input.memory, input.readiness),
    ];

    const requirementIds = new Set(["studied_cards", "practice_attempts", "topic_coverage"]);

    const requirements = input.requirements ?? [
        ...(input.memory?.abstentionRequirements ?? []),
        ...(input.performance?.abstentionRequirements ?? []),
        ...(input.readiness?.abstentionRequirements ?? []),
    ];

    const needsImprovement = mergeImprovements(
        weakTopicImprovements(input.weakTopics, input.weakTopicLimit ?? 3),
        unmetRequirementImprovements(requirements, requirementIds),
        [calibrationImprovement(input.readiness, input.calibration)].filter(
            (item): item is ImprovementItem => item !== null,
        ),
        [lowConfidenceImprovement(input.readiness)].filter(
            (item): item is ImprovementItem => item !== null,
        ),
    );

    return { basedOn, needsImprovement };
}

export function buildEstimatedGreExplainability(input: {
    memory?: MemoryScore;
    performance?: PerformanceScore;
    readiness?: ReadinessScore;
    weakTopics?: DashboardTopicInsight[];
    requirements?: AbstentionRequirement[];
    calibration?: ReadinessCalibrationStats;
}): PredictionExplainability {
    return buildPredictionExplainability(input);
}

export function buildReadinessExplainability(input: {
    memory?: MemoryScore;
    performance?: PerformanceScore;
    readiness: ReadinessScore;
    weakTopics?: DashboardTopicInsight[];
    calibration?: ReadinessCalibrationStats;
}): PredictionExplainability {
    return buildPredictionExplainability({
        ...input,
        requirements: input.readiness.abstentionRequirements,
    });
}

function practiceStatus(details: TopicDetailsResponse): string {
    if (details.practiceAccuracy !== undefined) {
        return `${Math.round(details.practiceAccuracy)}% (${details.practiceCorrect}/${details.practiceTotal} correct)`;
    }
    if (details.practiceTotal > 0) {
        return `${details.practiceCorrect}/${details.practiceTotal} attempts recorded`;
    }
    return "No practice attempts yet";
}

export function buildTopicExplainability(details: TopicDetailsResponse): PredictionExplainability {
    const basedOn: EvidencePillar[] = [
        {
            id: PILLAR_MEMORY,
            label: "Memory retention",
            met: details.memoryScore !== undefined,
            status:
                details.memoryScore !== undefined
                    ? `${Math.round(details.memoryScore)}% memory on this topic`
                    : `${details.studiedCards} studied cards on this topic`,
        },
        {
            id: PILLAR_PRACTICE,
            label: "Practice accuracy",
            met: details.practiceAccuracy !== undefined,
            status: practiceStatus(details),
        },
        {
            id: PILLAR_COVERAGE,
            label: "Topic coverage",
            met: details.covered,
            status: details.covered
                ? "Topic covered in your GRE deck"
                : "Not yet covered in your GRE deck",
        },
    ];

    const needsImprovement: ImprovementItem[] = [];
    if (!details.covered) {
        needsImprovement.push({
            id: "topic-coverage",
            label: details.displayName,
            detail: "Not yet covered in your GRE deck",
        });
    }
    if (details.covered && details.practiceTotal === 0) {
        needsImprovement.push({
            id: "topic-practice",
            label: "Practice accuracy",
            detail: "Answer questions on this topic",
        });
    }
    if (details.covered && details.studiedCards === 0) {
        needsImprovement.push({
            id: "topic-memory",
            label: "Memory retention",
            detail: "Review flashcards on this topic",
        });
    }

    return { basedOn, needsImprovement };
}
