// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    EstimatedGreScore,
    MemoryScore,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import { clampPercent } from "./indicator-utils";
import { settingsNavAction } from "./gre-navigation";
import { topicDetailsPath } from "./topic-link";

/** Display-only thresholds matching rslib abstention / calibration gates. */
const MIN_STUDIED_CARDS = 200;
const MIN_PRACTICE_ATTEMPTS = 20;
const MIN_COVERAGE_RATIO = 0.5;
const MIN_CALIBRATION_HELD_OUT = 5;

export type EvidenceCategoryId = "memory" | "practice" | "coverage" | "calibration";

export type EvidenceCategoryPresentation = {
    id: EvidenceCategoryId;
    label: string;
    percent: number;
};

export type PredictionReadinessAction = {
    label: string;
    href: string;
    bridge?: string;
    buttonLabel: string;
    estimatedImpact: number;
};

export type OnboardingContext = "home" | "dashboard" | "progress";

export type PredictionReadinessPresentation = {
    active: boolean;
    title: string;
    lead: string | null;
    evidencePercent: number;
    meterCaption: string;
    ringLabel: string;
    categories: EvidenceCategoryPresentation[];
    nextAction: PredictionReadinessAction;
};

export type PredictionReadinessInput = {
    deckExists: boolean;
    deckName: string;
    memory: MemoryScore;
    performance: PerformanceScore;
    estimatedGre: EstimatedGreScore;
    readiness?: ReadinessScore;
    calibration?: ReadinessCalibrationStats;
    dueTotal?: number;
    weakestTopicId?: string;
    weakestTopicName?: string;
    context?: OnboardingContext;
};

function requirementMet(
    requirements: { id: string; met: boolean }[],
    id: string,
): boolean {
    return requirements.find((req) => req.id === id)?.met ?? true;
}

function memoryCategoryPercent(memory: MemoryScore): number {
    if (memory.sufficientData && memory.value !== undefined) {
        return clampPercent(memory.value);
    }
    return clampPercent((memory.studiedCards / MIN_STUDIED_CARDS) * 100);
}

function practiceCategoryPercent(performance: PerformanceScore): number {
    if (performance.sufficientData && performance.value !== undefined) {
        return clampPercent(performance.value);
    }
    return clampPercent((performance.attemptCount / MIN_PRACTICE_ATTEMPTS) * 100);
}

function coverageCategoryPercent(memory: MemoryScore): number {
    if (requirementMet(memory.abstentionRequirements, "topic_coverage")) {
        return 100;
    }
    return clampPercent((memory.coverageRatio / MIN_COVERAGE_RATIO) * 100);
}

function calibrationCategoryPercent(
    readiness: ReadinessScore | undefined,
    calibration: ReadinessCalibrationStats | undefined,
): number {
    if (calibration?.wellCalibrated || readiness?.calibrationWellCalibrated) {
        return 100;
    }
    if (calibration && calibration.heldOutCount > 0) {
        return clampPercent((calibration.heldOutCount / MIN_CALIBRATION_HELD_OUT) * 100);
    }
    return 0;
}

function averagePercent(values: number[]): number {
    if (values.length === 0) {
        return 0;
    }
    return clampPercent(values.reduce((sum, value) => sum + value, 0) / values.length);
}

function impactFromCategoryGap(categoryPercent: number): number {
    return Math.max(1, Math.round((100 - categoryPercent) / 10));
}

function practiceEvidenceLacking(performance: PerformanceScore): boolean {
    return (
        performance.attemptCount < MIN_PRACTICE_ATTEMPTS ||
        !requirementMet(performance.abstentionRequirements, "practice_attempts")
    );
}

function coverageEvidenceLacking(memory: MemoryScore): boolean {
    return !requirementMet(memory.abstentionRequirements, "topic_coverage");
}

function buildNextAction(
    input: PredictionReadinessInput,
    categories: EvidenceCategoryPresentation[],
): PredictionReadinessAction {
    const memory = categories.find((cat) => cat.id === "memory")!;
    const practice = categories.find((cat) => cat.id === "practice")!;
    const coverage = categories.find((cat) => cat.id === "coverage")!;
    const calibration = categories.find((cat) => cat.id === "calibration")!;

    if (!input.deckExists) {
        const settings = settingsNavAction();
        return {
            label: `Set up "${input.deckName}" in Settings`,
            buttonLabel: "Set up deck",
            href: settings.href!,
            bridge: settings.bridge,
            estimatedImpact: impactFromCategoryGap(memory.percent),
        };
    }

    const dueTotal = input.dueTotal ?? 0;
    if (dueTotal > 0) {
        const cardsLabel = dueTotal === 1 ? "1 due card" : `${dueTotal} due cards`;
        return {
            label: `Review ${cardsLabel}`,
            buttonLabel: "Continue reviewing",
            href: "/review",
            estimatedImpact: impactFromCategoryGap(memory.percent),
        };
    }

    if (practiceEvidenceLacking(input.performance)) {
        const remaining = Math.max(1, MIN_PRACTICE_ATTEMPTS - input.performance.attemptCount);
        const questionLabel = remaining === 1 ? "1 practice question" : `${remaining} practice questions`;
        return {
            label: `Answer ${questionLabel}`,
            buttonLabel: "Continue practicing",
            href: "/practice",
            estimatedImpact: impactFromCategoryGap(practice.percent),
        };
    }

    if (coverageEvidenceLacking(input.memory)) {
        if (input.weakestTopicName && input.weakestTopicId) {
            return {
                label: `Study ${input.weakestTopicName}`,
                buttonLabel: "Study weakest topic",
                href: topicDetailsPath(input.weakestTopicId),
                estimatedImpact: impactFromCategoryGap(coverage.percent),
            };
        }
        return {
            label: "Review cards across more GRE topics",
            buttonLabel: "Continue studying",
            href: "/review",
            estimatedImpact: impactFromCategoryGap(coverage.percent),
        };
    }

    if (input.memory.studiedCards < MIN_STUDIED_CARDS) {
        const remaining = MIN_STUDIED_CARDS - input.memory.studiedCards;
        const cardLabel = remaining === 1 ? "1 more card" : `${remaining} more cards`;
        return {
            label: `Review ${cardLabel}`,
            buttonLabel: "Continue studying",
            href: "/review",
            estimatedImpact: impactFromCategoryGap(memory.percent),
        };
    }

    if (calibration.percent < 100) {
        const note = input.readiness?.calibrationNote;
        return {
            label: note || "Keep studying so predictions can be verified later",
            buttonLabel: "Continue studying",
            href: "/review",
            estimatedImpact: impactFromCategoryGap(calibration.percent),
        };
    }

    return {
        label: "Keep reviewing and practicing",
        buttonLabel: "Continue studying",
        href: "/review",
        estimatedImpact: impactFromCategoryGap(averagePercent(categories.map((cat) => cat.percent))),
    };
}

function onboardingCopy(
    context: OnboardingContext,
    evidencePercent: number,
): Pick<PredictionReadinessPresentation, "title" | "lead" | "meterCaption" | "ringLabel"> {
    const meterCaption = `${evidencePercent}% collected`;
    const ringLabel = `${evidencePercent}% collected toward first estimate`;

    switch (context) {
        case "dashboard":
            return {
                title: "Evidence for your first estimate",
                lead: "Complete study and practice to unlock your first estimated GRE score.",
                meterCaption,
                ringLabel,
            };
        case "progress":
            return {
                title: "Evidence for your first estimate",
                lead: null,
                meterCaption,
                ringLabel,
            };
        case "home":
        default:
            return {
                title: "Evidence for your first estimate",
                lead: null,
                meterCaption,
                ringLabel,
            };
    }
}

export function needsPredictionReadiness(estimatedGre: EstimatedGreScore): boolean {
    return estimatedGre.combinedScore === undefined;
}

export function presentPredictionReadiness(
    input: PredictionReadinessInput,
): PredictionReadinessPresentation {
    const active = needsPredictionReadiness(input.estimatedGre);

    const categories: EvidenceCategoryPresentation[] = input.deckExists
        ? [
              { id: "memory", label: "Memory", percent: memoryCategoryPercent(input.memory) },
              {
                  id: "practice",
                  label: "Practice",
                  percent: practiceCategoryPercent(input.performance),
              },
              {
                  id: "coverage",
                  label: "Coverage",
                  percent: coverageCategoryPercent(input.memory),
              },
              {
                  id: "calibration",
                  label: "Calibration",
                  percent: calibrationCategoryPercent(input.readiness, input.calibration),
              },
          ]
        : [
              { id: "memory", label: "Memory", percent: 0 },
              { id: "practice", label: "Practice", percent: 0 },
              { id: "coverage", label: "Coverage", percent: 0 },
              { id: "calibration", label: "Calibration", percent: 0 },
          ];

    const evidencePercent = averagePercent(categories.map((cat) => cat.percent));
    const nextAction = buildNextAction(input, categories);
    const copy = onboardingCopy(input.context ?? "home", evidencePercent);

    return {
        active,
        ...copy,
        evidencePercent,
        categories,
        nextAction,
    };
}
