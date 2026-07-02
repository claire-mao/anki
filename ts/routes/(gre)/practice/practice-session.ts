// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Question } from "@generated/anki/brainlift_pb";

export type PracticeSectionFilter = "all" | "quant" | "verbal" | "awa";

export function shuffleQuestions<T>(items: T[]): T[] {
    const copy = [...items];
    for (let i = copy.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [copy[i], copy[j]] = [copy[j], copy[i]];
    }
    return copy;
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

export function buildQuestionQueue(
    questions: Question[],
    section: PracticeSectionFilter,
): Question[] {
    return shuffleQuestions(filterQuestions(questions, section));
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
            return "AWA";
    }
}
