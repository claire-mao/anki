// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreStudyStatusResponse, StudyPlanDailyTask } from "@generated/anki/brainlift_pb";

import { type GreNavAction, runGreNavAction } from "./gre-navigation";
import { topicDetailsPath } from "./topic-link";

export type MissionIconName =
    | "study"
    | "practice"
    | "memory"
    | "topic";

export interface MissionProgress {
    label: string;
    value: number;
    detail?: string;
    showBar?: boolean;
}

export type MissionAction = GreNavAction;

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
        return "Review flashcards";
    }
    if (task.id === "practice_questions") {
        return "Practice questions";
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
        return "Close a catalog gap with new tagged cards.";
    }
    if (task.title.startsWith("Strengthen")) {
        return "Rebuild mastery where memory is slipping.";
    }
    return "Target this topic with focused practice.";
}

export function missionProgressUnit(task: StudyPlanDailyTask): string {
    if (task.id === "practice_questions" || task.title.startsWith("Practice")) {
        return "questions";
    }
    return "cards";
}

export function missionProgress(
    task: StudyPlanDailyTask,
    studyStatus?: GreStudyStatusResponse,
): MissionProgress {
    const target = task.targetCount;
    const unit = missionProgressUnit(task);

    if (task.id === "review_cards" && target === 0) {
        return {
            label: "All caught up",
            value: 100,
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

    let detail: string | undefined;
    if (task.id === "review_cards" && studyStatus) {
        const due = studyStatus.newCount + studyStatus.learnCount + studyStatus.reviewCount;
        detail = `${due} due now`;
    }

    return {
        label: `Target: ${target} ${unit}`,
        value: 0,
        detail,
        showBar: false,
    };
}

export function missionAction(task: StudyPlanDailyTask): MissionAction {
    if (task.id === "review_cards") {
        return {
            label: task.targetCount > 0 ? "Start review" : "View study plan",
            bridge: task.targetCount > 0 ? "greStartReview" : undefined,
            href: task.targetCount > 0 ? "/review" : "/study-plan",
        };
    }

    if (task.id === "practice_questions") {
        return {
            label: "Start practice",
            href: "/practice",
        };
    }

    if (task.title.startsWith("Cover") && task.topicId) {
        return {
            label: "Add cards",
            href: topicDetailsPath(task.topicId),
        };
    }

    if (task.title.startsWith("Strengthen") && task.topicId) {
        return {
            label: "Review topic",
            href: topicDetailsPath(task.topicId),
        };
    }

    return {
        label: "Start practice",
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
