// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardTopicInsight } from "@generated/anki/brainlift_pb";
import { describe, expect, it } from "vitest";

import { GRE_CTA_PRACTICE, GRE_CTA_REVIEW, GRE_CTA_STUDY_PLAN } from "./gre-navigation";
import { buildPracticeSessionSummary, buildStudyCaughtUpSummary } from "./session-completion";
import { GRE_EXTRA_STUDY_BATCH } from "./review/extra-study";

function topic(
    overrides: Partial<DashboardTopicInsight> & Pick<DashboardTopicInsight, "displayName">,
): DashboardTopicInsight {
    return {
        topicId: "gre::verbal::awa::analyze_issue",
        section: "awa",
        examWeight: 0.5,
        studiedCards: 0,
        covered: false,
        reason: "",
        studyLabel: "",
        ...overrides,
    } as DashboardTopicInsight;
}

describe("buildPracticeSessionSummary", () => {
    it("summarizes accuracy and topic focus", () => {
        const summary = buildPracticeSessionSummary([
            { topic: "Algebra", correct: true },
            { topic: "Algebra", correct: false },
            { topic: "Geometry", correct: true },
        ]);

        expect(summary.headline).toBe("Session complete");
        expect(summary.rows[0]).toEqual({ label: "Questions answered", value: "3" });
        expect(summary.rows[1]).toEqual({ label: "Accuracy", value: "67%" });
        expect(summary.rows.some((row) => row.label === "Strongest topic" && row.value === "Geometry"))
            .toBe(true);
        expect(summary.rows.some((row) => row.label === "Focus next" && row.value === "Algebra"))
            .toBe(true);
    });
});

describe("buildStudyCaughtUpSummary", () => {
    it("shows built-in flashcard guidance when there is no flashcard evidence", () => {
        const summary = buildStudyCaughtUpSummary({
            weakTopics: [topic({ displayName: "Analyze an Issue" })],
            recommendedTopics: [topic({ displayName: "Analyze an Issue" })],
            dueTotal: 0,
            deckName: "GRE Atlas",
            studiedCards: 0,
            coveredLeafCount: 0,
        });

        expect(summary.headline).toBe("Your GRE flashcards are ready");
        expect(summary.rows).toEqual([{ label: "Flashcards reviewed", value: "0" }]);
        expect(summary.nextAction.label).toBe(GRE_CTA_REVIEW);
        expect(summary.secondaryAction?.label).toBe(GRE_CTA_PRACTICE);
        expect(summary.nextActionDetail).toContain("Start review");
        expect(summary.subline).toContain("no Anki import");
    });

    it("shows caught-up stats without next-step actions when flashcards exist", () => {
        const summary = buildStudyCaughtUpSummary({
            weakTopics: [
                topic({
                    displayName: "Linear equations",
                    studiedCards: 12,
                    memoryScore: 42,
                    covered: true,
                }),
            ],
            recommendedTopics: [],
            dueTotal: 0,
            deckName: "GRE Atlas",
            studiedCards: 48,
            coveredLeafCount: 3,
            extraStudyAvailable: 0,
            nextReviewInDays: 2,
        });

        expect(summary.headline).toBe("Review complete");
        expect(summary.rows[0]).toEqual({ label: "Cards due now", value: "0" });
        expect(summary.rows[1]).toEqual({ label: "Flashcards reviewed", value: "48" });
        expect(summary.rows.some((row) => row.label === "Focus next")).toBe(true);
        expect(summary.rows.some((row) => row.label === "Next flashcard review")).toBe(true);
        expect(summary.nextAction).toBeUndefined();
        expect(summary.nextActionDetail).toBeUndefined();
        expect(summary.secondaryAction).toBeUndefined();
    });

    it("offers paced study ahead when extra flashcards are available", () => {
        const summary = buildStudyCaughtUpSummary({
            weakTopics: [],
            recommendedTopics: [],
            dueTotal: 0,
            deckName: "GRE Atlas",
            studiedCards: 48,
            coveredLeafCount: 3,
            extraStudyAvailable: GRE_EXTRA_STUDY_BATCH,
            availableNewCount: 120,
        });

        expect(summary.nextAction?.label).toBe(`Study ${GRE_EXTRA_STUDY_BATCH} more cards`);
        expect(summary.nextAction?.bridge).toBe("greStartExtraReview");
        expect(summary.nextActionDetail).toContain("without cramming");
        expect(summary.secondaryAction?.label).toBe(GRE_CTA_PRACTICE);
    });

    it("does not show topic insights from catalog defaults alone", () => {
        const summary = buildStudyCaughtUpSummary({
            weakTopics: [topic({ displayName: "Analyze an Issue" })],
            recommendedTopics: [topic({ displayName: "Text completion" })],
            dueTotal: 0,
            deckName: "GRE Atlas",
            studiedCards: 0,
            coveredLeafCount: 2,
        });

        expect(summary.headline).toBe("Review complete");
        expect(summary.rows.some((row) => row.label === "Flashcards reviewed" && row.value === "0"))
            .toBe(true);
        expect(summary.rows.some((row) => row.label === "Strongest area")).toBe(false);
        expect(summary.rows.some((row) => row.label === "Focus next")).toBe(false);
        expect(summary.rows.some((row) => row.label === "Flashcard history")).toBe(true);
    });
});

describe("buildPracticeSessionSummary focus completion", () => {
    it("shows focus completion with dashboard return and flashcard schedule", () => {
        const summary = buildPracticeSessionSummary(
            [{ topic: "Analyze an Argument", correct: true }],
            {
                focusTopicName: "Analyze an Argument",
                focusComplete: true,
                flashcardScheduleHint: "3 flashcards ready now in Study · next batch in 1 day",
            },
        );

        expect(summary.headline).toBe("Focus complete");
        expect(summary.nextAction.label).toBe(GRE_CTA_STUDY_PLAN);
        expect(summary.rows.some((row) => row.label === "Flashcard review")).toBe(true);
        expect(summary.nextActionDetail).toContain("next batch in 1 day");
    });
});
