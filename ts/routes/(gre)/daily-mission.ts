// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    GreStudyStatusResponse,
    PerformanceAttempt,
    StudyPlanDailyTask,
} from "@generated/anki/brainlift_pb";

import {
    type GreNavAction,
    GRE_CTA_PRACTICE,
    GRE_CTA_PRACTICE_TOPIC,
    GRE_CTA_REVIEW,
    GRE_CTA_STUDY_PLAN,
    GRE_CTA_STUDY_TOPIC,
    runGreNavAction,
} from "./gre-navigation";
import { clampPercent } from "./indicator-utils";
import { dueCardCount } from "./review/study-launch";
import { practicePathForTopic, topicDetailsPath } from "./topic-link";

export type MissionIconName =
    | "study"
    | "practice"
    | "memory"
    | "topic";

export interface MissionProgress {
    label: string;
    value: number;
    current?: number;
    target?: number;
    detail?: string;
    showBar?: boolean;
}

export type MissionAction = GreNavAction;

export type DailyMissionProgressContext = {
    studyStatus?: GreStudyStatusResponse;
    recentAttempts?: PerformanceAttempt[];
    dayStartSecs?: number;
};

export interface MissionProgressCounts {
    current: number;
    target: number;
    unit: string;
}

const REVIEW_BASELINE_KEY = "gre-daily-mission-review-baseline";

type ReviewBaselineStore = { date: string; due: number };

export function missionIcon(task: StudyPlanDailyTask): MissionIconName {
    if (task.id === "review_cards") {
        return "study";
    }
    if (task.id === "practice_questions") {
        return "practice";
    }
    if (task.title.startsWith("Strengthen")) {
        return "memory";
    }
    if (task.title.startsWith("Cover")) {
        return "topic";
    }
    return "practice";
}

export function missionTitle(task: StudyPlanDailyTask): string {
    if (task.id === "review_cards") {
        return GRE_CTA_REVIEW;
    }
    if (task.id === "practice_questions") {
        return GRE_CTA_PRACTICE;
    }
    return task.title.replace(/^GRE /, "");
}

export function missionDescription(task: StudyPlanDailyTask): string {
    if (task.id === "review_cards") {
        return task.targetCount > 0
            ? "Work through due cards and lock in retention."
            : "You're caught up — keep momentum with focus topics.";
    }
    if (task.id === "practice_questions") {
        return "Answer exam-style questions to sharpen accuracy.";
    }
    if (task.title.startsWith("Cover")) {
        return "Answer practice questions to close a catalog gap.";
    }
    if (task.title.startsWith("Strengthen")) {
        return "Rebuild mastery where memory is slipping.";
    }
    return "Target this topic with focused practice.";
}

function missionUsesPracticeProgress(task: StudyPlanDailyTask): boolean {
    return task.id === "practice_questions"
        || task.title.startsWith("Practice")
        || task.title.startsWith("Cover");
}

export function missionProgressUnit(task: StudyPlanDailyTask): string {
    return missionUsesPracticeProgress(task) ? "questions" : "cards";
}

export function localDateKey(date = new Date()): string {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
}

export function startOfLocalDaySecs(date = new Date()): number {
    const start = new Date(date);
    start.setHours(0, 0, 0, 0);
    return Math.floor(start.getTime() / 1000);
}

export function parseReviewBaselineDue(detail: string): number | null {
    const match = detail.match(/(\d+) new,\s*(\d+) learning,\s*(\d+) review/);
    if (!match) {
        return null;
    }
    return Number(match[1]) + Number(match[2]) + Number(match[3]);
}

export function attemptMatchesTopic(attemptTopic: string, topicId?: string): boolean {
    const id = topicId?.trim();
    if (!id) {
        return true;
    }
    const topic = attemptTopic.trim();
    return topic === id || topic.startsWith(`${id}::`);
}

export function countAttemptsSince(
    attempts: PerformanceAttempt[] | undefined,
    sinceSecs: number | undefined,
    topicId?: string,
): number {
    if (!attempts?.length) {
        return 0;
    }
    const since = sinceSecs ?? startOfLocalDaySecs();
    return attempts.filter(
        (attempt) =>
            Number(attempt.answeredAtSecs) >= since
            && attemptMatchesTopic(attempt.topic, topicId),
    ).length;
}

function clampCount(value: number, max: number): number {
    return Math.max(0, Math.min(max, Math.round(value)));
}

function storedReviewBaseline(studyStatus?: GreStudyStatusResponse): number | null {
    if (!studyStatus || typeof window === "undefined") {
        return null;
    }
    const due = dueCardCount(studyStatus);
    const today = localDateKey();
    try {
        const raw = window.localStorage.getItem(REVIEW_BASELINE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw) as ReviewBaselineStore;
            if (parsed.date === today) {
                return parsed.due;
            }
        }
        window.localStorage.setItem(
            REVIEW_BASELINE_KEY,
            JSON.stringify({ date: today, due } satisfies ReviewBaselineStore),
        );
        return due;
    } catch {
        return due;
    }
}

function reviewCardsCompleted(
    task: StudyPlanDailyTask,
    studyStatus?: GreStudyStatusResponse,
): number {
    if (!studyStatus) {
        return 0;
    }
    const baseline =
        parseReviewBaselineDue(task.detail) ?? storedReviewBaseline(studyStatus);
    if (baseline === null) {
        return 0;
    }
    return clampCount(baseline - dueCardCount(studyStatus), task.targetCount);
}

export function missionProgressCounts(
    task: StudyPlanDailyTask,
    context?: DailyMissionProgressContext,
): MissionProgressCounts | null {
    const target = task.targetCount;
    if (target === 0) {
        return null;
    }
    const unit = missionProgressUnit(task);

    if (task.id === "review_cards") {
        return {
            current: reviewCardsCompleted(task, context?.studyStatus),
            target,
            unit,
        };
    }

    if (missionUsesPracticeProgress(task)) {
        return {
            current: countAttemptsSince(
                context?.recentAttempts,
                context?.dayStartSecs,
                task.topicId,
            ),
            target,
            unit,
        };
    }

    return {
        current: 0,
        target,
        unit,
    };
}

export function missionProgress(
    task: StudyPlanDailyTask,
    context?: DailyMissionProgressContext,
): MissionProgress {
    const target = task.targetCount;
    const unit = missionProgressUnit(task);

    if (task.id === "review_cards" && target === 0) {
        return {
            label: "All caught up",
            value: 100,
            current: 0,
            target: 0,
            detail: "Nothing due right now.",
            showBar: true,
        };
    }

    if (target === 0) {
        return {
            label: "Ready to start",
            value: 0,
            showBar: false,
        };
    }

    const counts = missionProgressCounts(task, context)!;
    const value = clampPercent((counts.current / counts.target) * 100);
    let detail = `${counts.current}/${counts.target} ${counts.unit}`;

    if (task.id === "review_cards" && context?.studyStatus) {
        const due = dueCardCount(context.studyStatus);
        detail = `${detail} · ${due} due now`;
    }

    return {
        label: `Target: ${target} ${unit}`,
        value,
        current: counts.current,
        target: counts.target,
        detail,
        showBar: true,
    };
}

export function missionAction(task: StudyPlanDailyTask): MissionAction {
    if (task.id === "review_cards") {
        return {
            label: task.targetCount > 0 ? GRE_CTA_REVIEW : GRE_CTA_STUDY_PLAN,
            bridge: task.targetCount > 0 ? "greStartReview" : "greOpenStudyPlan",
            href: task.targetCount > 0 ? "/review" : "/study-plan",
        };
    }

    if (task.id === "practice_questions") {
        return {
            label: GRE_CTA_PRACTICE,
            bridge: "greOpenPractice",
            href: "/practice",
        };
    }

    if (task.title.startsWith("Cover") && task.topicId) {
        return {
            label: GRE_CTA_PRACTICE_TOPIC,
            href: practicePathForTopic(task.topicId),
        };
    }

    if (task.title.startsWith("Strengthen") && task.topicId) {
        return {
            label: GRE_CTA_STUDY_TOPIC,
            href: topicDetailsPath(task.topicId),
        };
    }

    if (task.title.startsWith("Practice") && task.topicId) {
        return {
            label: GRE_CTA_PRACTICE,
            href: practicePathForTopic(task.topicId),
        };
    }

    return {
        label: GRE_CTA_PRACTICE,
        href: "/practice",
    };
}

export function runMissionAction(action: MissionAction): void {
    runGreNavAction(action);
}

export function missionIntro(taskCount: number): string {
    if (taskCount === 1) {
        return "One focused action to move your score today.";
    }
    return `${taskCount} focused actions to move your score today.`;
}
