// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Question } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    buildQuestionQueue,
    buildTopicPracticeQueue,
    TOPIC_PRACTICE_SESSION_SIZE,
} from "./practice-session";

function question(id: string, topic: string, section = "quant"): Question {
    return {
        id,
        topic,
        section,
        format: "mcq",
        stem: id,
        choices: ["A", "B"],
    };
}

describe("practice session queue", () => {
    test("topic sessions prefer five questions when the bank is large enough", () => {
        const bank = Array.from({ length: 8 }, (_, index) =>
            question(`q-${index}`, "gre::quant::algebra::linear"),
        );

        expect(buildTopicPracticeQueue(bank)).toHaveLength(TOPIC_PRACTICE_SESSION_SIZE);
        expect(buildTopicPracticeQueue(bank).map((item) => item.id)).toEqual([
            "q-0",
            "q-1",
            "q-2",
            "q-3",
            "q-4",
        ]);
    });

    test("topic sessions cycle short banks up to the session target", () => {
        const bank = [
            question("arg-1", "gre::awa::argument", "awa"),
            question("arg-2", "gre::awa::argument", "awa"),
        ];

        expect(buildTopicPracticeQueue(bank)).toHaveLength(TOPIC_PRACTICE_SESSION_SIZE);
        expect(buildTopicPracticeQueue(bank).map((item) => item.id)).toEqual([
            "arg-1",
            "arg-2",
            "arg-1",
            "arg-2",
            "arg-1",
        ]);
    });

    test("general practice keeps the full filtered queue", () => {
        const bank = Array.from({ length: 12 }, (_, index) =>
            question(`q-${index}`, "gre::quant::algebra::linear"),
        );

        expect(buildQuestionQueue(bank, "all")).toHaveLength(12);
    });

    test("topic filter limits section practice to five topic questions", () => {
        const bank = [
            ...Array.from({ length: 6 }, (_, index) =>
                question(`arg-${index}`, "gre::awa::argument", "awa"),
            ),
            question("lin-1", "gre::quant::algebra::linear", "quant"),
        ];

        expect(
            buildQuestionQueue(bank, "all", {
                topicFilter: "gre::awa::argument",
            }),
        ).toHaveLength(TOPIC_PRACTICE_SESSION_SIZE);
    });
});
