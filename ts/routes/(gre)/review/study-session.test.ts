// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import {
    formatEstimatedTime,
    formatRecallProbability,
    gradeButtonsFromLabels,
    queueBadge,
    studyDueTotal,
    studyMetrics,
    studyProgressLabel,
    studyProgressPercent,
} from "./study-session";

describe("gradeButtonsFromLabels", () => {
    test("maps scheduler labels to Anki ratings with names and shortcuts", () => {
        const buttons = gradeButtonsFromLabels(["1m", "6m", "10m", "4d"]);
        expect(buttons).toHaveLength(4);
        expect(buttons.map((b) => b.rating)).toEqual([0, 1, 2, 3]);
        expect(buttons.map((b) => b.name)).toEqual([
            "Again",
            "Hard",
            "Good",
            "Easy",
        ]);
        expect(buttons.map((b) => b.shortcut)).toEqual(["1", "2", "3", "4"]);
        expect(buttons.map((b) => b.variant)).toEqual([
            "again",
            "hard",
            "good",
            "easy",
        ]);
        // Scheduler interval stays available as a secondary hint.
        expect(buttons[3].label).toBe("4d");
    });

    test("handles a shorter label list without throwing", () => {
        const buttons = gradeButtonsFromLabels(["1m", "10m"]);
        expect(buttons).toHaveLength(2);
        expect(buttons[1].variant).toBe("hard");
    });
});

describe("study metrics", () => {
    const counts = { dueNew: 5, dueLearn: 3, dueReview: 12 };

    test("dueTotal sums the three queues", () => {
        expect(studyDueTotal(counts)).toBe(20);
    });

    test("metrics row exposes remaining, per-queue counts, and time", () => {
        const metrics = studyMetrics(counts);
        expect(metrics.map((m) => m.label)).toEqual([
            "Cards remaining",
            "New",
            "Learning",
            "Review",
            "Estimated time",
        ]);
        expect(metrics[0].value).toBe("20");
        expect(metrics[4].value).toBe("3 min");
    });

    test("estimated time is at least one minute when cards are due", () => {
        expect(formatEstimatedTime(1)).toBe("1 min");
        expect(formatEstimatedTime(0)).toBe("0 min");
    });
});

describe("queueBadge", () => {
    test("maps known queue kinds", () => {
        expect(queueBadge("new").label).toBe("New");
        expect(queueBadge("learning").label).toBe("Learning");
        expect(queueBadge("review").label).toBe("Review");
    });

    test("falls back to review for unknown values", () => {
        expect(queueBadge("mystery").label).toBe("Review");
    });
});

describe("study progress", () => {
    test("percent reflects reviewed share of the session", () => {
        expect(studyProgressPercent(0, 20)).toBe(0);
        expect(studyProgressPercent(5, 15)).toBe(25);
        expect(studyProgressPercent(20, 0)).toBe(100);
    });

    test("percent is 100 when nothing is due", () => {
        expect(studyProgressPercent(0, 0)).toBe(100);
    });

    test("label describes remaining work", () => {
        expect(studyProgressLabel(0, 3)).toBe("3 cards to go");
        expect(studyProgressLabel(2, 1)).toBe("1 card to go");
        expect(studyProgressLabel(5, 0)).toBe("All caught up");
        expect(studyProgressLabel(0, 0)).toBe("No cards due");
    });
});

describe("formatRecallProbability", () => {
    test("formats a fraction as a percentage", () => {
        expect(formatRecallProbability(0.83)).toBe("83%");
        expect(formatRecallProbability(1.4)).toBe("100%");
        expect(formatRecallProbability(-0.2)).toBe("0%");
    });

    test("returns null when unknown", () => {
        expect(formatRecallProbability(null)).toBeNull();
        expect(formatRecallProbability(undefined)).toBeNull();
        expect(formatRecallProbability(NaN)).toBeNull();
    });
});
