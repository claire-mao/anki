// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { TopicMasteryEntry } from "@generated/anki/stats_pb";
import { describe, expect, test } from "vitest";

import {
    isTopicMasteryStarted,
    topicDisplayMasteryPercent,
    topicMasteryClusterNote,
    topicMasteryConfidenceLabel,
    topicMasteryRowTooltip,
    topicMasterySectionExplanation,
    topicMasteryVerificationRows,
} from "./topic-mastery-presentation";

function topic(
    id: string,
    studiedCards: number,
    displayMastery: number,
    extra: Partial<TopicMasteryEntry> = {},
): TopicMasteryEntry {
    return {
        topicId: id,
        displayName: id,
        studiedCards,
        avgRetrievability: displayMastery,
        displayMastery,
        practiceAttempts: 0,
        practiceAccuracy: 0,
        confidenceLabel: "",
        masteryConfidence: 0,
        evidenceCount: studiedCards,
        ...extra,
    } as TopicMasteryEntry;
}

describe("topic mastery presentation", () => {
    test("topicDisplayMasteryPercent uses the blended display mastery", () => {
        expect(topicDisplayMasteryPercent(topic("gre::quant::algebra", 5, 0.72))).toBe(72);
    });

    test("verification rows expose blended mastery and confidence", () => {
        const rows = topicMasteryVerificationRows([
            topic("gre::quant::algebra::linear", 2, 0.82, {
                confidenceLabel: "Moderate",
                masteryConfidence: 0.5,
            }),
            topic("gre::verbal::text_completion", 0, 0),
        ]);

        expect(rows[0]).toEqual({
            topicId: "gre::quant::algebra::linear",
            displayName: "gre::quant::algebra::linear",
            studiedCards: 2,
            displayPercent: 82,
            confidenceLabel: "Moderate",
            confidencePercent: 50,
        });
        expect(rows[1].displayPercent).toBeUndefined();
        expect(rows[1].confidenceLabel).toBeUndefined();
    });

    test("topicDisplayMasteryPercent ignores practice-only topics", () => {
        expect(
            topicDisplayMasteryPercent(topic("gre::verbal::tc", 0, 0, { practiceAttempts: 3 })),
        ).toBeUndefined();
    });

    test("topicDisplayMasteryPercent is undefined when not started", () => {
        expect(topicDisplayMasteryPercent(topic("gre::quant::algebra", 0, 0))).toBeUndefined();
    });

    test("isTopicMasteryStarted requires reviewed flashcards", () => {
        expect(isTopicMasteryStarted(topic("gre::verbal::tc", 0, 0, { practiceAttempts: 3 }))).toBe(
            false,
        );
        expect(isTopicMasteryStarted(topic("gre::quant::algebra", 1, 0.8))).toBe(true);
    });

    test("confidence label surfaces separately from mastery", () => {
        expect(
            topicMasteryConfidenceLabel(
                topic("gre::quant::algebra", 1, 0.9, { confidenceLabel: "Low" }),
            ),
        ).toBe("Low");
        expect(topicMasteryConfidenceLabel(topic("gre::quant::algebra", 0, 0))).toBeUndefined();
    });

    test("section explanation describes blended evidence and confidence", () => {
        expect(topicMasterySectionExplanation()).toMatch(/blends/i);
        expect(topicMasterySectionExplanation()).toMatch(/confidence/i);
    });

    test("cluster note appears when studied topics share similar mastery", () => {
        const clustered = [
            topic("a", 2, 0.97),
            topic("b", 3, 0.96),
            topic("c", 1, 0.95),
        ];

        expect(topicMasteryClusterNote(clustered)).toMatch(/Similar percentages/i);
    });

    test("cluster note is omitted when mastery spread is wide", () => {
        const spread = [
            topic("a", 2, 0.97),
            topic("b", 3, 0.82),
            topic("c", 1, 0.65),
        ];

        expect(topicMasteryClusterNote(spread)).toBeUndefined();
    });

    test("row tooltip reports mastery, confidence, and evidence", () => {
        expect(
            topicMasteryRowTooltip(
                topic("gre::quant::algebra", 4, 0.82, {
                    confidenceLabel: "Moderate",
                    evidenceCount: 4,
                }),
            ),
        ).toBe("82% mastery · Moderate confidence · 4 evidence items");
        expect(
            topicMasteryRowTooltip(
                topic("gre::quant::algebra", 1, 0.82, {
                    confidenceLabel: "Low",
                    evidenceCount: 1,
                }),
            ),
        ).toBe("82% mastery · Low confidence · 1 evidence item");
        expect(topicMasteryRowTooltip(topic("gre::quant::algebra", 0, 0))).toMatch(/Not started/i);
    });
});
