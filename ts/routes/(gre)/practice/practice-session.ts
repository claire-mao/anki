// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Question } from "@generated/anki/brainlift_pb";

export type PracticeSectionFilter = "all" | "quant" | "verbal" | "awa";

/** Matches `DAILY_PRACTICE_TARGET` in `rslib/src/gre_atlas/study_plan.rs`. */
/** Matches AWA focus targets in `rslib/src/gre_atlas/questions/bank.rs`. */
export const AWA_TOPIC_PRACTICE_SESSION_SIZE = 10;

export const TOPIC_PRACTICE_SESSION_SIZE = 8;

/** Matches `PRACTICE_QUESTION_LIST_LIMIT` in `rslib/src/gre_atlas/questions/bank.rs`. */
export const PRACTICE_QUESTION_LIST_LIMIT = 260;

/** Matches `TARGET_PRACTICE_BANK_*` in `rslib/src/gre_atlas/questions/bank.rs`. */
export const PRACTICE_BANK_SECTION_TOTALS = {
    quant: 100,
    verbal: 100,
    awa: 60,
    all: 260,
} as const;

export function practiceSectionQuestionTotal(
    section: PracticeSectionFilter,
): number {
    return PRACTICE_BANK_SECTION_TOTALS[section];
}

export function countQuestionsInSection(
    questions: Question[],
    section: PracticeSectionFilter,
): number {
    if (section === "all") {
        return questions.length;
    }
    return filterQuestions(questions, section).length;
}

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
 * when enough exist; otherwise returns every available question without repeating.
 */
export function buildTopicPracticeQueue(
    questions: Question[],
    sessionSize: number = TOPIC_PRACTICE_SESSION_SIZE,
): Question[] {
    if (questions.length === 0 || sessionSize === 0) {
        return [];
    }
    return questions.slice(0, Math.min(sessionSize, questions.length));
}

export type BuildQuestionQueueOptions = {
    topicFilter?: string;
    sessionSize?: number;
};

export function topicPracticeSessionSize(topicFilter: string): number {
    const normalized = topicFilter.trim();
    if (normalized.startsWith("gre::awa::")) {
        return AWA_TOPIC_PRACTICE_SESSION_SIZE;
    }
    return TOPIC_PRACTICE_SESSION_SIZE;
}

function shuffleQuestions<T>(
    items: T[],
    random: () => number = Math.random,
): T[] {
    const shuffled = items.slice();
    for (let index = shuffled.length - 1; index > 0; index -= 1) {
        const swapIndex = Math.floor(random() * (index + 1));
        [shuffled[index], shuffled[swapIndex]] = [shuffled[swapIndex], shuffled[index]];
    }
    return shuffled;
}

export function buildQuestionQueue(
    questions: Question[],
    section: PracticeSectionFilter,
    options?: BuildQuestionQueueOptions,
): Question[] {
    // `list_questions` orders by practice history (least-answered first). Preserve
    // that order for section/topic filters; mix all sections with a one-time shuffle.
    let filtered = filterQuestions(questions, section);
    const topicFilter = options?.topicFilter?.trim() ?? "";
    if (topicFilter) {
        filtered = filterQuestionsByTopic(filtered, topicFilter);
        return buildTopicPracticeQueue(
            filtered,
            options?.sessionSize ?? topicPracticeSessionSize(topicFilter),
        );
    }
    if (section === "all") {
        return shuffleQuestions(filtered);
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
