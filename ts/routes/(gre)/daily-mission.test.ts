// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    GreStudyStatusResponse,
    PerformanceAttempt,
    StudyPlanDailyTask,
} from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    attemptMatchesTopic,
    countAttemptsSince,
    missionProgress,
    missionProgressCounts,
    parseReviewBaselineDue,
    startOfLocalDaySecs,
} from "./daily-mission";

function task(partial: Partial<StudyPlanDailyTask> & Pick<StudyPlanDailyTask, "id" | "title">): StudyPlanDailyTask {
    return {
        detail: "",
        targetCount: 5,
        ...partial,
    };
}

function studyStatus(newCount: number, learnCount: number, reviewCount: number): GreStudyStatusResponse {
    return {
        deckName: "GRE Atlas",
        deckExists: true,
        newCount,
        learnCount,
        reviewCount,
    };
}

describe("parseReviewBaselineDue", () => {
    test("parses due breakdown from plan detail", () => {
        expect(
            parseReviewBaselineDue(
                "Clear up to 20 due cards (4 new, 6 learning, 10 review)",
            ),
        ).toBe(20);
    });
});

describe("attemptMatchesTopic", () => {
    test("matches exact topic ids and descendants", () => {
        expect(attemptMatchesTopic("gre::awa::argument", "gre::awa::argument")).toBe(true);
        expect(attemptMatchesTopic("gre::awa::argument::tone", "gre::awa::argument")).toBe(true);
        expect(attemptMatchesTopic("gre::quant::algebra", "gre::awa::argument")).toBe(false);
    });
});

describe("missionProgressCounts", () => {
    test("tracks review progress from due-count delta", () => {
        const counts = missionProgressCounts(
            task({
                id: "review_cards",
                title: "Review GRE flashcards",
                targetCount: 10,
                detail: "Clear up to 10 due cards (2 new, 3 learning, 5 review)",
            }),
            {
                studyStatus: studyStatus(1, 2, 2),
            },
        );

        expect(counts).toEqual({
            current: 5,
            target: 10,
            unit: "cards",
        });
    });

    test("counts practice questions answered today", () => {
        const dayStart = startOfLocalDaySecs(new Date("2026-07-03T00:00:00"));
        const attempts: PerformanceAttempt[] = [
            {
                questionId: "q1",
                topic: "gre::quant::algebra",
                answeredAtSecs: dayStart + 60,
                answer: "A",
                correct: true,
                responseTimeMs: 1000,
            },
            {
                questionId: "q2",
                topic: "gre::verbal::vocab",
                answeredAtSecs: dayStart + 120,
                answer: "B",
                correct: false,
                responseTimeMs: 900,
            },
            {
                questionId: "q3",
                topic: "gre::verbal::vocab",
                answeredAtSecs: dayStart - 60,
                answer: "C",
                correct: true,
                responseTimeMs: 800,
            },
        ];

        expect(
            countAttemptsSince(attempts, dayStart),
        ).toBe(2);

        expect(
            missionProgressCounts(
                task({
                    id: "practice_questions",
                    title: "GRE practice questions",
                    targetCount: 5,
                }),
                { recentAttempts: attempts, dayStartSecs: dayStart },
            ),
        ).toEqual({
            current: 2,
            target: 5,
            unit: "questions",
        });
    });

    test("filters topic practice attempts by topic id", () => {
        const dayStart = startOfLocalDaySecs(new Date("2026-07-03T12:00:00"));
        const attempts: PerformanceAttempt[] = [
            {
                questionId: "q1",
                topic: "gre::quant::data_interpretation",
                answeredAtSecs: dayStart + 10,
                answer: "A",
                correct: true,
                responseTimeMs: 1000,
            },
            {
                questionId: "q2",
                topic: "gre::verbal::vocab",
                answeredAtSecs: dayStart + 20,
                answer: "B",
                correct: true,
                responseTimeMs: 1000,
            },
        ];

        expect(
            missionProgressCounts(
                task({
                    id: "focus_topic",
                    title: "Practice Data interpretation",
                    targetCount: 5,
                    topicId: "gre::quant::data_interpretation",
                }),
                { recentAttempts: attempts, dayStartSecs: dayStart },
            ),
        ).toEqual({
            current: 1,
            target: 5,
            unit: "questions",
        });
    });

    test("counts cover task progress from topic practice attempts", () => {
        const dayStart = startOfLocalDaySecs(new Date("2026-07-03T12:00:00"));
        const attempts: PerformanceAttempt[] = [
            {
                questionId: "q1",
                topic: "gre::awa::argument",
                answeredAtSecs: dayStart + 10,
                answer: "A",
                correct: true,
                responseTimeMs: 1000,
            },
            {
                questionId: "q2",
                topic: "gre::quant::algebra",
                answeredAtSecs: dayStart + 20,
                answer: "B",
                correct: true,
                responseTimeMs: 1000,
            },
        ];

        expect(
            missionProgressCounts(
                task({
                    id: "focus_topic",
                    title: "Cover Analyze an Argument",
                    targetCount: 5,
                    topicId: "gre::awa::argument",
                }),
                { recentAttempts: attempts, dayStartSecs: dayStart },
            ),
        ).toEqual({
            current: 1,
            target: 5,
            unit: "questions",
        });
    });
});

describe("missionProgress", () => {
    test("caps review progress at target and renders percent", () => {
        const progress = missionProgress(
            task({
                id: "review_cards",
                title: "Review GRE flashcards",
                targetCount: 5,
                detail: "Clear up to 5 due cards (1 new, 2 learning, 2 review)",
            }),
            {
                studyStatus: studyStatus(0, 0, 0),
            },
        );

        expect(progress.value).toBe(100);
        expect(progress.current).toBe(5);
        expect(progress.showBar).toBe(true);
    });

    test("shows full bar when review target is zero", () => {
        const progress = missionProgress(
            task({
                id: "review_cards",
                title: "Review GRE flashcards",
                targetCount: 0,
            }),
        );

        expect(progress.value).toBe(100);
        expect(progress.label).toBe("All caught up");
    });
});
