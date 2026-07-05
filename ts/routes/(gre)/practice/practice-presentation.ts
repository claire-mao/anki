// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { formatPercent } from "../score-format";
import type { SessionAttemptRecord } from "../session-completion";
import { formatSectionLabel, type PracticeSectionFilter } from "./practice-session";

const ETS_STEM_PREFIX = /^Based on ETS Official GRE Prep Material[.:]?\s*/i;

/** Legacy generated stems prefix section and task before the prompt. */
const SECTION_TASK_STEM_PREFIX = /^\([^)]+\):\s*/;

const PRACTICE_BANK_SOURCE = "GRE Atlas Practice Bank";

const INTERNAL_PRACTICE_SOURCES = new Set([
    PRACTICE_BANK_SOURCE.toLowerCase(),
]);

/** Provenance flag stored for deterministic (non-AI) generated questions. */
export const PROVENANCE_OFFLINE_TEMPLATE = "offline_template";
/** Provenance flag stored for real LLM output that passed the eval gate. */
export const PROVENANCE_AI_GENERATED = "ai_generated";
/**
 * Exact note surfaced whenever the deterministic fallback produced content.
 * Kept in sync with the Rust `OFFLINE_TEMPLATE_NOTE` constant.
 */
export const OFFLINE_TEMPLATE_NOTE = "Generated using offline templates.";

const FORMAT_LABELS: Record<string, string> = {
    mcq: "Multiple Choice",
    multiple_choice: "Multiple Choice",
    text_completion: "Text Completion",
    sentence_equivalence: "Sentence Equivalence",
    reading_comprehension: "Reading Comprehension",
    data_interpretation: "Data Interpretation",
    essay_prompt: "Essay Prompt",
};

const SECTION_DISPLAY: Record<string, string> = {
    quant: "Quantitative Reasoning",
    verbal: "Verbal Reasoning",
    awa: "Analytical Writing",
};

const TOPIC_DISPLAY: Record<string, string> = {
    "gre::quant::arithmetic::percent": "Percents",
    "gre::quant::arithmetic::ratio": "Ratios & proportions",
    "gre::quant::algebra::linear": "Linear equations",
    "gre::quant::algebra::quadratic": "Quadratic equations",
    "gre::quant::geometry::triangles": "Triangles",
    "gre::quant::geometry::circles": "Circles",
    "gre::quant::data_interpretation": "Data interpretation",
    "gre::quant::statistics::probability": "Probability",
    "gre::quant::statistics::data_analysis": "Data analysis",
    "gre::quant::word_problems": "Word problems",
    "gre::quant::number_properties": "Number properties",
    "gre::verbal::reading::inference": "Inference",
    "gre::verbal::reading::main_idea": "Main idea",
    "gre::verbal::reading::supporting_detail": "Supporting detail",
    "gre::verbal::reading::function": "Function of a sentence",
    "gre::verbal::text_completion": "Text completion",
    "gre::verbal::sentence_equivalence": "Sentence equivalence",
    "gre::verbal::vocabulary::context": "Context clues",
    "gre::verbal::vocabulary::advanced": "Advanced vocabulary",
    "gre::awa::issue": "Analyze an Issue",
    "gre::awa::argument": "Analyze an Argument",
};

/** Display-only stem cleanup for legacy attribution prefixes in stored questions. */
export function displayQuestionStem(stem: string): string {
    const trimmed = stem.trim();
    if (!trimmed) {
        return trimmed;
    }
    return trimmed
        .replace(ETS_STEM_PREFIX, "")
        .replace(SECTION_TASK_STEM_PREFIX, "")
        .trim();
}

export function formatQuestionType(format: string): string {
    const normalized = format.trim().toLowerCase();
    if (!normalized) {
        return "Question";
    }
    return FORMAT_LABELS[normalized] ?? normalized.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function formatPracticeSection(section: string): string {
    const normalized = section.trim().toLowerCase();
    return SECTION_DISPLAY[normalized] ?? formatSectionLabel(normalized as PracticeSectionFilter);
}

export function isInternalPracticeSource(sourceName: string | undefined): boolean {
    if (!sourceName?.trim()) {
        return true;
    }
    return INTERNAL_PRACTICE_SOURCES.has(sourceName.trim().toLowerCase());
}

export type PracticeMetadataLine = {
    kind: "source" | "task";
    text: string;
};

/** Source attribution for external items; task label for GRE Atlas–authored items. */
export function formatPracticeMetadataLine(
    topicId: string,
    section: string,
    sourceName?: string,
): PracticeMetadataLine {
    const task = formatPracticeTopicLabel(topicId);
    if (isInternalPracticeSource(sourceName)) {
        return { kind: "task", text: task };
    }
    if (!sourceName?.trim()) {
        return { kind: "task", text: task };
    }
    return {
        kind: "source",
        text: `${sourceName.trim()} • ${task}`,
    };
}

/** @deprecated Use formatPracticeMetadataLine */
export function formatPracticeSourceLine(
    section: string,
    sourceName?: string,
    topicId?: string,
): string | null {
    const line = formatPracticeMetadataLine(topicId ?? "", section, sourceName);
    return line.kind === "source" ? line.text : null;
}

export function progressQuestionNumber(input: {
    questionsCompleted: number;
    queueLength: number;
    sessionComplete: boolean;
}): number {
    if (input.sessionComplete || input.queueLength === 0) {
        return input.queueLength;
    }
    return Math.min(input.questionsCompleted + 1, input.queueLength);
}

export function progressPercentForSession(input: {
    questionsCompleted: number;
    queueLength: number;
    sessionComplete: boolean;
}): number {
    if (input.queueLength === 0) {
        return 0;
    }
    if (input.sessionComplete) {
        return 100;
    }
    return Math.round(
        (progressQuestionNumber(input) / input.queueLength) * 100,
    );
}

export function progressLabelForSession(input: {
    questionsCompleted: number;
    queueLength: number;
    sessionComplete: boolean;
    emptyLabel: string;
}): string {
    if (input.queueLength === 0) {
        return input.emptyLabel;
    }
    if (input.sessionComplete) {
        return "Session complete";
    }
    const number = progressQuestionNumber(input);
    return `Question ${number} of ${input.queueLength}`;
}

export function formatPracticeTopicLabel(topicId: string): string {
    const normalized = topicId.trim();
    if (!normalized) {
        return "Unknown topic";
    }
    if (TOPIC_DISPLAY[normalized]) {
        return TOPIC_DISPLAY[normalized];
    }
    const leaf = normalized.split("::").pop() ?? normalized;
    return leaf.replace(/_/g, " ").replace(/\b\w/g, (character) => character.toUpperCase());
}

export function computeSessionStreak(attempts: SessionAttemptRecord[]): number {
    let streak = 0;
    for (let index = attempts.length - 1; index >= 0; index -= 1) {
        if (!attempts[index]?.correct) {
            break;
        }
        streak += 1;
    }
    return streak;
}

export function computeSessionAccuracy(attempts: SessionAttemptRecord[]): number | null {
    if (attempts.length === 0) {
        return null;
    }
    const correct = attempts.filter((attempt) => attempt.correct).length;
    return (correct / attempts.length) * 100;
}

export function formatSessionAccuracy(attempts: SessionAttemptRecord[]): string {
    const accuracy = computeSessionAccuracy(attempts);
    return accuracy === null ? "—" : formatPercent(accuracy);
}

/**
 * Resolve the correct choice for reveal UI. When the attempt was correct, the
 * selected answer is authoritative. Otherwise infer from the explanation text.
 */
export function resolveCorrectChoice(input: {
    choices: string[];
    selected: string;
    correct: boolean;
    explanation: string;
}): string | null {
    if (input.correct) {
        return input.selected;
    }

    const matches = [...input.choices]
        .filter((choice) => choice.trim() && input.explanation.includes(choice))
        .sort((left, right) => {
            const positionDelta = input.explanation.lastIndexOf(right) - input.explanation.lastIndexOf(left);
            if (positionDelta !== 0) {
                return positionDelta;
            }
            return right.length - left.length;
        });

    if (matches.length === 0) {
        return null;
    }

    const distinct = new Set(matches);
    if (distinct.size === 1) {
        return matches[0] ?? null;
    }

    return matches[0] ?? null;
}

/** Qualitative difficulty label when numeric difficulty is not exposed client-side. */
export function estimateDifficultyLabel(format: string, section: string): string {
    const normalized = format.trim().toLowerCase();
    if (normalized === "essay_prompt" || section === "awa") {
        return "Advanced";
    }
    if (normalized === "reading_comprehension") {
        return "Moderate–Hard";
    }
    if (normalized === "text_completion" || normalized === "sentence_equivalence") {
        return "Moderate";
    }
    if (normalized === "data_interpretation") {
        return "Moderate";
    }
    return "Standard";
}

export function buildSkillImpactPhrase(section: string): string {
    switch (section.trim().toLowerCase()) {
        case "verbal":
            return "Builds verbal accuracy evidence";
        case "quant":
            return "Builds quantitative reasoning evidence";
        case "awa":
            return "Builds analytical writing evidence";
        default:
            return "Builds practice performance evidence";
    }
}

export type PracticeRevealRow = {
    label: string;
    value: string;
};

export function buildPracticeRevealRows(input: {
    topic: string;
    section: string;
    format: string;
}): PracticeRevealRow[] {
    return [
        {
            label: "Difficulty",
            value: estimateDifficultyLabel(input.format, input.section),
        },
        {
            label: "Topic",
            value: formatPracticeTopicLabel(input.topic),
        },
        {
            label: "Skill impact",
            value: buildSkillImpactPhrase(input.section),
        },
    ];
}

/** Whether a provenance flag denotes the deterministic offline-template path. */
export function isOfflineTemplateProvenance(provenance: string | undefined): boolean {
    return (provenance ?? "").trim().toLowerCase() === PROVENANCE_OFFLINE_TEMPLATE;
}

/**
 * The provenance note to show under an explanation. Returns exactly
 * "Generated using offline templates." for the deterministic fallback, and
 * `null` for AI-generated content (no fallback note needed).
 */
export function resolveExplanationProvenanceNote(input: {
    provenance: string | undefined;
    provenanceNote: string | undefined;
}): string | null {
    if (!isOfflineTemplateProvenance(input.provenance)) {
        return null;
    }
    const note = input.provenanceNote?.trim();
    return note && note.length > 0 ? note : OFFLINE_TEMPLATE_NOTE;
}

export type ExplanationCitation = {
    sourceName: string;
    sourceSection: string;
    excerpt: string;
};

/**
 * Format a one-line citation for the grounding source, or `null` when no
 * source is available. Includes the section when present.
 */
export function formatExplanationCitation(
    citation: ExplanationCitation | undefined,
): string | null {
    const name = citation?.sourceName?.trim();
    if (!name) {
        return null;
    }
    const section = citation?.sourceSection?.trim();
    return section ? `${name} — ${section}` : name;
}

export type ExplanationChoiceRow = {
    choice: string;
    isCorrect: boolean;
    reasoning: string;
};

/**
 * Order the per-choice reasoning so the correct answer appears first, matching
 * how learners scan the result panel. Filters out entries without reasoning.
 */
export function orderExplanationChoices(
    choices: ExplanationChoiceRow[] | undefined,
): ExplanationChoiceRow[] {
    if (!choices?.length) {
        return [];
    }
    return [...choices]
        .filter((row) => row.reasoning.trim().length > 0)
        .sort((left, right) => Number(right.isCorrect) - Number(left.isCorrect));
}
