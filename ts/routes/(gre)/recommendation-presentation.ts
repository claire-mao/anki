// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardTopicInsight, StudyPlanDailyTask, StudyPlanRecommendation } from "@generated/anki/brainlift_pb";

import { formatPercent } from "./score-format";
import { topicDetailsPath } from "./topic-link";

export type StudyRecommendationAction = {
    label: string;
    href?: string;
    bridge?: string;
};

export type StudyRecommendationPresentation = {
    title: string;
    reason: string;
    expectedImpact: string;
    action: StudyRecommendationAction;
    topicId?: string;
};

const FACTOR_PRIORITY = [
    "coverage_gap",
    "low_mastery",
    "low_performance",
    "no_practice",
    "high_importance",
] as const;

const FACTOR_REASON: Record<string, string> = {
    coverage_gap: "Coverage gap detected",
    low_mastery: "Weak retention detected",
    low_performance: "Low practice accuracy",
    no_practice: "No practice evidence yet",
    high_importance: "High exam importance",
};

const FACTOR_IMPACT: Record<string, string> = {
    coverage_gap: "Expands GRE catalog coverage",
    low_mastery: "Raises memory retention",
    low_performance: "Improves practice accuracy",
    no_practice: "Builds practice evidence",
    high_importance: "High-weight exam topic",
};

function primaryFactor(factors: string[]): string {
    for (const id of FACTOR_PRIORITY) {
        if (factors.includes(id)) {
            return id;
        }
    }
    return factors[0] ?? "high_importance";
}

function priorityImpactLabel(priorityScore: number, maxPriority: number): string {
    if (maxPriority <= 0) {
        return "Ranked by impact";
    }
    const relative = priorityScore / maxPriority;
    if (relative >= 0.9) {
        return "Highest estimated impact";
    }
    if (relative >= 0.65) {
        return "High estimated impact";
    }
    return "Moderate estimated impact";
}

function recommendationAction(rec: StudyPlanRecommendation): StudyRecommendationAction {
    if (rec.factors.includes("coverage_gap")) {
        return {
            label: "Add cards",
            href: topicDetailsPath(rec.topicId),
        };
    }
    if (rec.factors.includes("low_mastery")) {
        return {
            label: "Start review",
            href: topicDetailsPath(rec.topicId),
        };
    }
    if (rec.factors.includes("low_performance") || rec.factors.includes("no_practice")) {
        return {
            label: "Start practice",
            href: "/practice",
        };
    }
    return {
        label: "Study topic",
        href: topicDetailsPath(rec.topicId),
    };
}

function formatExpectedImpact(
    rec: StudyPlanRecommendation,
    maxPriority: number,
): string {
    const factor = primaryFactor(rec.factors);
    const parts = [
        priorityImpactLabel(rec.priorityScore, maxPriority),
        FACTOR_IMPACT[factor] ?? FACTOR_IMPACT.high_importance,
        `${formatPercent(rec.examWeight * 100)} exam weight`,
    ];
    return parts.join(" · ");
}

export function presentStudyPlanRecommendation(
    rec: StudyPlanRecommendation,
    maxPriority: number,
): StudyRecommendationPresentation {
    const factor = primaryFactor(rec.factors);
    const reason = FACTOR_REASON[factor]
        ?? rec.explanation.split(" · ")[0]
        ?? rec.explanation;

    return {
        title: rec.displayName,
        reason,
        expectedImpact: formatExpectedImpact(rec, maxPriority),
        action: recommendationAction(rec),
        topicId: rec.topicId,
    };
}

export function presentStudyPlanRecommendations(
    recommendations: StudyPlanRecommendation[],
): StudyRecommendationPresentation[] {
    const maxPriority = recommendations.reduce(
        (max, rec) => Math.max(max, rec.priorityScore),
        0,
    );
    return recommendations.map((rec) => presentStudyPlanRecommendation(rec, maxPriority));
}

export function presentTopicInsight(topic: DashboardTopicInsight): StudyRecommendationPresentation {
    const impactParts = [`${formatPercent(topic.examWeight * 100)} exam weight`];
    if (topic.memoryScore !== undefined) {
        impactParts.push(`${formatPercent(topic.memoryScore)} memory`);
    }
    if (topic.practiceAccuracy !== undefined) {
        impactParts.push(`${formatPercent(topic.practiceAccuracy)} practice`);
    }

    let action: StudyRecommendationAction;
    if (!topic.covered) {
        action = { label: "Add cards", href: topicDetailsPath(topic.topicId) };
    } else if (topic.practiceAccuracy === undefined) {
        action = { label: "Start practice", href: "/practice" };
    } else {
        action = { label: "Start review", href: topicDetailsPath(topic.topicId) };
    }

    return {
        title: topic.displayName,
        reason: topic.reason,
        expectedImpact: impactParts.join(" · "),
        action,
        topicId: topic.topicId,
    };
}

export function presentTopicInsights(
    topics: DashboardTopicInsight[],
): StudyRecommendationPresentation[] {
    return topics.map(presentTopicInsight);
}

export function presentDailyFocusTask(task: StudyPlanDailyTask): StudyRecommendationPresentation | null {
    if (task.id !== "focus_topic" || !task.topicId) {
        return null;
    }

    const title = task.topicDisplayName || task.title.replace(/^(Cover|Strengthen|Practice)\s+/, "");

    let reason = task.detail;
    if (task.title.startsWith("Cover")) {
        reason = "Coverage gap detected";
    } else if (task.title.startsWith("Strengthen")) {
        reason = "Weak retention detected";
    } else if (task.title.startsWith("Practice")) {
        reason = "Low practice accuracy";
    }

    let expectedImpact = "Focused topic impact";
    if (task.targetCount > 0) {
        const unit = task.title.startsWith("Practice") ? "questions" : "cards";
        expectedImpact = `Target ${task.targetCount} ${unit}`;
    }

    let action: StudyRecommendationAction;
    if (task.title.startsWith("Cover")) {
        action = { label: "Add cards", href: topicDetailsPath(task.topicId) };
    } else if (task.title.startsWith("Strengthen")) {
        action = { label: "Start review", href: topicDetailsPath(task.topicId) };
    } else {
        action = { label: "Start practice", href: "/practice" };
    }

    return {
        title,
        reason,
        expectedImpact,
        action,
        topicId: task.topicId,
    };
}
