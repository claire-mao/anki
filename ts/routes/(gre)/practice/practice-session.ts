// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Question } from "@generated/anki/brainlift_pb";

export type PracticeSectionFilter = "all" | "quant" | "verbal" | "awa";

/** Matches `DAILY_PRACTICE_TARGET` in `rslib/src/gre_atlas/study_plan.rs`. */
export const TOPIC_PRACTICE_SESSION_SIZE = 5;

export function filterQuestions(
    questions: Question[],
    section: PracticeSectionFilter,
): Question[] {
    if (section === "all") {
        return questions;
    }
    return questions.filter((question) => question.section === section);
}

export function filterQuestionsByTopic(
    questions: Question[],
    topicFilter: string,
): Question[] {
    const normalized = topicFilter.trim();
    if (!normalized) {
        return questions;
    }
    return questions.filter(
        (question) =>
            question.topic === normalized
            || question.topic.startsWith(`${normalized}::`),
    );
}

/**
 * Build a topic-focused session queue. Uses up to `sessionSize` unique questions
 * when enough exist; otherwise cycles through the available bank to reach the
 * target session length.
 */
export function buildTopicPracticeQueue(
    questions: Question[],
    sessionSize: number = TOPIC_PRACTICE_SESSION_SIZE,
): Question[] {
    if (questions.length === 0 || sessionSize === 0) {
        return [];
    }
    if (questions.length >= sessionSize) {
        return questions.slice(0, sessionSize);
    }

    const queue: Question[] = [];
    for (let index = 0; index < sessionSize; index += 1) {
        queue.push(questions[index % questions.length]!);
    }
    return queue;
}

export type BuildQuestionQueueOptions = {
    topicFilter?: string;
    sessionSize?: number;
};

export function buildQuestionQueue(
    questions: Question[],
    section: PracticeSectionFilter,
    options?: BuildQuestionQueueOptions,
): Question[] {
    // `list_questions` already orders by practice history (least-answered first)
    // with a random tie-break, so preserve that order rather than reshuffling.
    let filtered = filterQuestions(questions, section);
    const topicFilter = options?.topicFilter?.trim() ?? "";
    if (topicFilter) {
        filtered = filterQuestionsByTopic(filtered, topicFilter);
        return buildTopicPracticeQueue(
            filtered,
            options?.sessionSize ?? TOPIC_PRACTICE_SESSION_SIZE,
        );
    }
    return filtered;
}

export function formatSectionLabel(section: PracticeSectionFilter): string {
    switch (section) {
        case "all":
            return "All sections";
        case "quant":
            return "Quant";
        case "verbal":
            return "Verbal";
        case "awa":
            return "AWA (essays)";
    }
}
