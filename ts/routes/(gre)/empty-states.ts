// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { AbstentionRequirement } from "@generated/anki/brainlift_pb";

import { GRE_CTA_PRACTICE, GRE_CTA_REVIEW, GRE_CTA_STUDY_PLAN, greNavAction, greNavItem } from "./gre-navigation";

export type EmptyStateAction = {
    label: string;
    href?: string;
    bridge?: string;
};

export type EmptyStateContent = {
    kicker?: string;
    title: string;
    explanation: string;
    unlockGoal: string;
    action: EmptyStateAction;
};

export type EmptyStateKey =
    | "estimatedGre"
    | "readiness"
    | "memory"
    | "performance"
    | "scoreChart"
    | "estimatedGreChart"
    | "topicMasteryChart"
    | "calibrationChart"
    | "weakTopics"
    | "homeWeakTopics"
    | "recommendations"
    | "studyPlanRecommendations"
    | "recentPractice"
    | "homeRecentPractice"
    | "calibrationTable"
    | "studiedCards"
    | "practiceAttempts"
    | "topicQuestions"
    | "topicAttempts"
    | "noQuestionsFilter"
    | "noCardsDue"
    | "missionReviewCaughtUp"
    | "deckMissing";

export const MIN_STUDIED_CARDS = 50;
export const MIN_PRACTICE_ATTEMPTS = 50;
export const MIN_COVERAGE_PERCENT = 50;

const emptyStateRegistry: Record<EmptyStateKey, EmptyStateContent> = {
    estimatedGre: {
        kicker: "Estimated GRE",
        title: "Prediction locked",
        explanation: "Finish these milestones.",
        unlockGoal: "to unlock your first estimated GRE score.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    readiness: {
        kicker: "Readiness",
        title: "Readiness locked",
        explanation: "Build memory and practice evidence.",
        unlockGoal: "to unlock your readiness score.",
        action: { label: GRE_CTA_STUDY_PLAN, bridge: "greOpenStudyPlan", href: "/study-plan" },
    },
    memory: {
        kicker: "Memory",
        title: "Memory score locked",
        explanation: "Review flashcards to build evidence.",
        unlockGoal: "to unlock your memory score.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    performance: {
        kicker: "Performance",
        title: "Performance locked",
        explanation: "Answer practice questions.",
        unlockGoal: "to unlock performance predictions.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    scoreChart: {
        title: "Score locked",
        explanation: "Build review and practice evidence.",
        unlockGoal: "to fill in this chart.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    estimatedGreChart: {
        title: "Prediction locked",
        explanation: "Complete unlock milestones.",
        unlockGoal: "to unlock your GRE score chart.",
        action: { label: "View progress", href: "/progress" },
    },
    topicMasteryChart: {
        title: "Mastery locked",
        explanation: "Review tagged GRE cards.",
        unlockGoal: "to unlock topic mastery.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    calibrationChart: {
        title: "Chart locked",
        explanation: "Keep studying and practicing.",
        unlockGoal: "to see how accurate your score estimates have been.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    weakTopics: {
        title: "No weak spots yet",
        explanation: "Keep building evidence.",
        unlockGoal: "to see focus topics here.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    homeWeakTopics: {
        title: "No weak spots yet",
        explanation: "Keep building evidence.",
        unlockGoal: "to see your weakest topic here.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    recommendations: {
        title: "No recommendations yet",
        explanation: "Coverage and scores need more data.",
        unlockGoal: "to unlock personalized focus topics.",
        action: { label: GRE_CTA_STUDY_PLAN, bridge: "greOpenStudyPlan", href: "/study-plan" },
    },
    studyPlanRecommendations: {
        title: "No recommendations yet",
        explanation: "Keep reviewing and practicing.",
        unlockGoal: "to unlock ranked topic recommendations.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    recentPractice: {
        title: "No practice yet",
        explanation: "Answer questions to track progress.",
        unlockGoal: "to see recent sessions here.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    homeRecentPractice: {
        title: "No practice yet",
        explanation: "Answer questions to track progress.",
        unlockGoal: "to see recent sessions here.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    calibrationTable: {
        title: "Details locked",
        explanation: "More study sessions are needed.",
        unlockGoal: "to compare estimates with your results.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    studiedCards: {
        title: "Memory score locked",
        explanation: "Review cards on this topic.",
        unlockGoal: "to unlock memory for this topic.",
        action: { label: GRE_CTA_REVIEW, href: "/review" },
    },
    practiceAttempts: {
        title: "Performance locked",
        explanation: "Answer questions on this topic.",
        unlockGoal: "to unlock practice accuracy.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    topicQuestions: {
        title: "Questions locked",
        explanation: "Expand coverage for this topic.",
        unlockGoal: "to unlock practice questions here.",
        action: { label: GRE_CTA_STUDY_PLAN, bridge: "greOpenStudyPlan", href: "/study-plan" },
    },
    topicAttempts: {
        title: "No attempts yet",
        explanation: "Answer questions on this topic.",
        unlockGoal: "to see attempts here.",
        action: { label: GRE_CTA_PRACTICE, href: "/practice" },
    },
    noQuestionsFilter: {
        title: "No questions here",
        explanation: "Try another section filter.",
        unlockGoal: "to practice in this section.",
        action: { label: "Show all sections" },
    },
    noCardsDue: {
        title: "All caught up",
        explanation: "Nothing due right now.",
        unlockGoal: "to keep momentum with focus topics.",
        action: { label: GRE_CTA_STUDY_PLAN, bridge: "greOpenStudyPlan", href: "/study-plan" },
    },
    missionReviewCaughtUp: {
        title: "All caught up",
        explanation: "Nothing due right now.",
        unlockGoal: "to keep momentum with focus topics.",
        action: { label: GRE_CTA_STUDY_PLAN, bridge: "greOpenStudyPlan", href: "/study-plan" },
    },
    deckMissing: {
        title: "Flashcards not ready",
        explanation: "Open Study to load built-in GRE flashcards.",
        unlockGoal: "to start building review evidence.",
        action: { ...greNavAction(greNavItem("study")), label: GRE_CTA_REVIEW },
    },
};

export function emptyStateContent(key: EmptyStateKey): EmptyStateContent {
    return emptyStateRegistry[key];
}

export function emptyStateTitle(key: EmptyStateKey): string {
    return emptyStateRegistry[key].title;
}

export function sortRequirementsForProgress(
    requirements: AbstentionRequirement[],
): AbstentionRequirement[] {
    return [...requirements].sort((a, b) => Number(a.met) - Number(b.met));
}

export function requirementUnlockLabel(req: AbstentionRequirement): string {
    if (req.met) {
        switch (req.id) {
            case "studied_cards":
                return `${MIN_STUDIED_CARDS} reviewed cards`;
            case "practice_attempts":
                return `${MIN_PRACTICE_ATTEMPTS} practice questions`;
            case "topic_coverage":
                return `${MIN_COVERAGE_PERCENT}% topic coverage`;
            case "fsrs_enabled":
                return "FSRS scheduling enabled";
            default:
                return req.label;
        }
    }
    return req.status;
}

export function chartEmptyLabel(
    kind: "score" | "estimatedGre" | "topicMastery" | "calibration",
): string {
    switch (kind) {
        case "estimatedGre":
            return emptyStateTitle("estimatedGreChart");
        case "topicMastery":
            return emptyStateTitle("topicMasteryChart");
        case "calibration":
            return emptyStateTitle("calibrationChart");
        default:
            return emptyStateTitle("scoreChart");
    }
}
