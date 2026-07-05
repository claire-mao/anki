// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceAttempt, StudyPlanDailyTask } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import { startOfLocalDaySecs } from "./daily-mission";
import { GRE_CTA_PRACTICE, GRE_CTA_PRACTICE_TOPIC, GRE_CTA_STUDY_TOPIC } from "./gre-navigation";
import { presentDailyFocusTask } from "./recommendation-presentation";
import { practicePathForTopic, topicDetailsPath } from "./topic-link";

function focusTask(partial: Partial<StudyPlanDailyTask> & Pick<StudyPlanDailyTask, "title">): StudyPlanDailyTask {
    return {
        id: "focus_topic",
        detail: "",
        targetCount: 5,
        topicId: "gre::awa::argument",
        topicDisplayName: "Analyze an Argument",
        ...partial,
    } as StudyPlanDailyTask;
}

describe("presentDailyFocusTask", () => {
    test("coverage gap tasks open topic-filtered practice", () => {
        const presentation = presentDailyFocusTask(
            focusTask({ title: "Cover Analyze an Argument" }),
        );

        expect(presentation).not.toBeNull();
        expect(presentation?.title).toBe("Analyze an Argument");
        expect(presentation?.reason).toBe("Coverage gap detected");
        expect(presentation?.action).toEqual({
            label: GRE_CTA_PRACTICE_TOPIC,
            href: practicePathForTopic("gre::awa::argument"),
        });
    });

    test("strengthen tasks still open topic details for flashcard review", () => {
        const presentation = presentDailyFocusTask(
            focusTask({
                title: "Strengthen Function of a sentence",
                topicId: "gre::verbal::reading::function",
                topicDisplayName: "Function of a sentence",
            }),
        );

        expect(presentation?.action).toEqual({
            label: GRE_CTA_STUDY_TOPIC,
            href: topicDetailsPath("gre::verbal::reading::function"),
        });
    });

    test("practice tasks open topic-filtered practice", () => {
        const presentation = presentDailyFocusTask(
            focusTask({
                title: "Practice Data interpretation",
                topicId: "gre::quant::data_interpretation",
                topicDisplayName: "Data interpretation",
            }),
        );

        expect(presentation?.action).toEqual({
            label: GRE_CTA_PRACTICE,
            href: practicePathForTopic("gre::quant::data_interpretation"),
        });
    });

    test("includes progress toward target for focus tasks", () => {
        const dayStart = startOfLocalDaySecs(new Date("2026-07-03T08:00:00"));
        const presentation = presentDailyFocusTask(
            focusTask({
                title: "Cover Analyze an Argument",
                targetCount: 5,
            }),
            {
                recentAttempts: [],
                dayStartSecs: dayStart,
            },
        );

        expect(presentation?.progress).toEqual({
            current: 0,
            target: 5,
            unit: "questions",
            percent: 0,
        });
    });

    test("counts cover task progress from topic practice attempts", () => {
        const dayStart = startOfLocalDaySecs(new Date("2026-07-03T08:00:00"));
        const presentation = presentDailyFocusTask(
            focusTask({
                title: "Cover Analyze an Argument",
                targetCount: 5,
            }),
            {
                recentAttempts: [
                    {
                        questionId: "q1",
                        topic: "gre::awa::argument",
                        answeredAtSecs: BigInt(dayStart + 30),
                        answer: "A",
                        correct: true,
                        responseTimeMs: 1000,
                    } as unknown as PerformanceAttempt,
                ],
                dayStartSecs: dayStart,
            },
        );

        expect(presentation?.progress).toEqual({
            current: 1,
            target: 5,
            unit: "questions",
            percent: 20,
        });
    });
});
