// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import type { SessionAttemptRecord } from "../session-completion";
import {
    buildPracticeRevealRows,
    computeSessionAccuracy,
    computeSessionStreak,
    displayQuestionStem,
    formatExplanationCitation,
    formatPracticeMetadataLine,
    formatPracticeSourceLine,
    formatPracticeTopicLabel,
    formatQuestionType,
    isOfflineTemplateProvenance,
    OFFLINE_TEMPLATE_NOTE,
    orderExplanationChoices,
    progressLabelForSession,
    resolveCorrectChoice,
    resolveExplanationProvenanceNote,
} from "./practice-presentation";

const attempts = (values: boolean[]): SessionAttemptRecord[] =>
    values.map((correct, index) => ({
        correct,
        topic: `topic-${index}`,
    }));

describe("practice presentation", () => {
    test("strips legacy ETS attribution prefix from stems for display", () => {
        expect(
            displayQuestionStem(
                "Based on ETS Official GRE Prep Material. What is 15% of 60?",
            ),
        ).toBe("What is 15% of 60?");
        expect(
            displayQuestionStem(
                '(Analytical Writing — Analyze an Argument): Argument: "Downloads rose." What is the flaw?',
            ),
        ).toBe('Argument: "Downloads rose." What is the flaw?');
        expect(displayQuestionStem("What is 15% of 60?")).toBe("What is 15% of 60?");
    });

    test("maps question formats to readable labels", () => {
        expect(formatQuestionType("text_completion")).toBe("Text Completion");
        expect(formatQuestionType("mcq")).toBe("Multiple Choice");
    });

    test("formats practice source metadata with task type", () => {
        expect(
            formatPracticeSourceLine(
                "awa",
                "GRE Atlas Practice Bank",
                "gre::awa::argument",
            ),
        ).toBeNull();
        expect(
            formatPracticeMetadataLine(
                "gre::awa::argument",
                "awa",
                "ETS Official GRE Prep Material",
            ),
        ).toEqual({
            kind: "source",
            text: "ETS Official GRE Prep Material • Analyze an Argument",
        });
        expect(
            formatPracticeMetadataLine(
                "gre::awa::argument",
                "awa",
                "GRE Atlas Practice Bank",
            ),
        ).toEqual({
            kind: "task",
            text: "Analyze an Argument",
        });
    });

    test("advances progress label when a question is completed", () => {
        expect(
            progressLabelForSession({
                questionsCompleted: 0,
                queueLength: 59,
                sessionComplete: false,
                emptyLabel: "Empty",
            }),
        ).toBe("Question 1 of 59");
        expect(
            progressLabelForSession({
                questionsCompleted: 1,
                queueLength: 59,
                sessionComplete: false,
                emptyLabel: "Empty",
            }),
        ).toBe("Question 2 of 59");
    });

    test("humanizes topic ids with catalog labels when known", () => {
        expect(formatPracticeTopicLabel("gre::quant::algebra::linear")).toBe(
            "Linear equations",
        );
        expect(formatPracticeTopicLabel("gre::verbal::reading::inference")).toBe(
            "Inference",
        );
    });

    test("computes session streak from trailing correct attempts", () => {
        expect(computeSessionStreak(attempts([true, true, false, true]))).toBe(1);
        expect(computeSessionStreak(attempts([true, true, true]))).toBe(3);
        expect(computeSessionStreak([])).toBe(0);
    });

    test("computes session accuracy", () => {
        expect(computeSessionAccuracy([])).toBeNull();
        expect(computeSessionAccuracy(attempts([true, false, true]))).toBeCloseTo(66.666, 2);
    });

    test("resolves the correct choice from explanation when attempt was wrong", () => {
        expect(
            resolveCorrectChoice({
                choices: ["$55", "$60", "$65", "$70"],
                selected: "$55",
                correct: false,
                explanation: "25% of $80 is $20, so the sale price is $80 − $20 = $60.",
            }),
        ).toBe("$60");
    });

    test("uses selected answer when attempt was correct", () => {
        expect(
            resolveCorrectChoice({
                choices: ["$55", "$60", "$65", "$70"],
                selected: "$60",
                correct: true,
                explanation: "Correct.",
            }),
        ).toBe("$60");
    });

    test("builds structured reveal metadata rows", () => {
        const rows = buildPracticeRevealRows({
            topic: "gre::verbal::text_completion",
            section: "verbal",
            format: "text_completion",
        });
        expect(rows.map((row) => row.label)).toEqual([
            "Difficulty",
            "Topic",
            "Skill impact",
        ]);
        expect(rows[2]?.value).toBe("Builds verbal accuracy evidence");
    });
});

describe("explanation presentation", () => {
    test("detects offline-template provenance", () => {
        expect(isOfflineTemplateProvenance("offline_template")).toBe(true);
        expect(isOfflineTemplateProvenance("OFFLINE_TEMPLATE")).toBe(true);
        expect(isOfflineTemplateProvenance("ai_generated")).toBe(false);
        expect(isOfflineTemplateProvenance(undefined)).toBe(false);
    });

    test("surfaces the exact offline-template note only for the fallback", () => {
        expect(
            resolveExplanationProvenanceNote({
                provenance: "offline_template",
                provenanceNote: "Generated using offline templates.",
            }),
        ).toBe("Generated using offline templates.");
        // Missing note text still yields the canonical note for the fallback.
        expect(
            resolveExplanationProvenanceNote({
                provenance: "offline_template",
                provenanceNote: "",
            }),
        ).toBe(OFFLINE_TEMPLATE_NOTE);
        // AI-generated content shows no fallback note.
        expect(
            resolveExplanationProvenanceNote({
                provenance: "ai_generated",
                provenanceNote: "Explained by gpt-4o-mini.",
            }),
        ).toBeNull();
    });

    test("formats a source citation with optional section", () => {
        expect(
            formatExplanationCitation({
                sourceName: "ETS Official GRE Prep Material",
                sourceSection: "Quantitative Reasoning — Linear equations",
                excerpt: "",
            }),
        ).toBe("ETS Official GRE Prep Material — Quantitative Reasoning — Linear equations");
        expect(
            formatExplanationCitation({
                sourceName: "ETS Official GRE Prep Material",
                sourceSection: "",
                excerpt: "",
            }),
        ).toBe("ETS Official GRE Prep Material");
        expect(
            formatExplanationCitation({ sourceName: "", sourceSection: "x", excerpt: "" }),
        ).toBeNull();
    });

    test("orders choice reasoning with the correct answer first", () => {
        const ordered = orderExplanationChoices([
            { choice: "A", isCorrect: false, reasoning: "wrong because…" },
            { choice: "B", isCorrect: true, reasoning: "correct because…" },
            { choice: "C", isCorrect: false, reasoning: "" },
        ]);
        expect(ordered.map((row) => row.choice)).toEqual(["B", "A"]);
    });
});
