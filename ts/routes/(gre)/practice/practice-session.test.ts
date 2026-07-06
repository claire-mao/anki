// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { Question } from "@generated/anki/brainlift_pb";
import { describe, expect, test, vi } from "vitest";

import {
    AWA_TOPIC_PRACTICE_SESSION_SIZE,
    buildQuestionQueue,
    buildTopicPracticeQueue,
    countQuestionsInSection,
    PRACTICE_BANK_SECTION_TOTALS,
    PRACTICE_QUESTION_LIST_LIMIT,
    practiceSectionQuestionTotal,
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
    } as Question;
}

describe("practice session queue", () => {
    test("practice list limit matches offline bank total", () => {
        expect(PRACTICE_QUESTION_LIST_LIMIT).toBe(260);
        expect(PRACTICE_BANK_SECTION_TOTALS).toEqual({
            quant: 100,
            verbal: 100,
            awa: 60,
            all: 260,
        });
    });

    test("section totals match canonical bank targets", () => {
        expect(practiceSectionQuestionTotal("quant")).toBe(100);
        expect(practiceSectionQuestionTotal("verbal")).toBe(100);
        expect(practiceSectionQuestionTotal("awa")).toBe(60);
        expect(practiceSectionQuestionTotal("all")).toBe(260);
    });

    test("filtered bank counts partition the full practice list", () => {
        const bank = [
            ...Array.from({ length: 100 }, (_, index) =>
                question(`q-${index}`, "gre::quant::algebra::linear", "quant"),
            ),
            ...Array.from({ length: 100 }, (_, index) =>
                question(`v-${index}`, "gre::verbal::reading::main", "verbal"),
            ),
            ...Array.from({ length: 60 }, (_, index) =>
                question(`a-${index}`, "gre::awa::argument", "awa"),
            ),
        ];

        expect(countQuestionsInSection(bank, "quant")).toBe(100);
        expect(countQuestionsInSection(bank, "verbal")).toBe(100);
        expect(countQuestionsInSection(bank, "awa")).toBe(60);
        expect(countQuestionsInSection(bank, "all")).toBe(260);
        expect(
            countQuestionsInSection(bank, "quant")
                + countQuestionsInSection(bank, "verbal")
                + countQuestionsInSection(bank, "awa"),
        ).toBe(countQuestionsInSection(bank, "all"));
    });

    test("topic sessions prefer eight questions when the bank is large enough", () => {
        const bank = Array.from({ length: 10 }, (_, index) => question(`q-${index}`, "gre::quant::algebra::linear"));

        expect(buildTopicPracticeQueue(bank)).toHaveLength(TOPIC_PRACTICE_SESSION_SIZE);
        expect(buildTopicPracticeQueue(bank).map((item) => item.id)).toEqual([
            "q-0",
            "q-1",
            "q-2",
            "q-3",
            "q-4",
            "q-5",
            "q-6",
            "q-7",
        ]);
    });

    test("topic sessions use every available question when the bank is short", () => {
        const bank = [
            question("arg-1", "gre::awa::argument", "awa"),
            question("arg-2", "gre::awa::argument", "awa"),
        ];

        expect(buildTopicPracticeQueue(bank)).toHaveLength(2);
        expect(buildTopicPracticeQueue(bank).map((item) => item.id)).toEqual([
            "arg-1",
            "arg-2",
        ]);
    });

    test("general practice keeps the full filtered queue", () => {
        const bank = [
            ...Array.from({ length: 100 }, (_, index) =>
                question(`q-${index}`, "gre::quant::algebra::linear", "quant"),
            ),
            ...Array.from({ length: 100 }, (_, index) =>
                question(`v-${index}`, "gre::verbal::reading::main", "verbal"),
            ),
            ...Array.from({ length: 60 }, (_, index) =>
                question(`a-${index}`, "gre::awa::argument", "awa"),
            ),
        ];

        expect(buildQuestionQueue(bank, "quant")).toHaveLength(100);
        expect(buildQuestionQueue(bank, "verbal")).toHaveLength(100);
        expect(buildQuestionQueue(bank, "awa")).toHaveLength(60);
        expect(buildQuestionQueue(bank, "all")).toHaveLength(260);
    });

    test("all-section practice shuffles mixed sections", () => {
        const bank = [
            ...Array.from({ length: 4 }, (_, index) =>
                question(`q-${index}`, "gre::quant::algebra::linear", "quant"),
            ),
            ...Array.from({ length: 4 }, (_, index) =>
                question(`v-${index}`, "gre::verbal::reading::main", "verbal"),
            ),
        ];
        const random = vi.spyOn(Math, "random").mockReturnValue(0);

        expect(buildQuestionQueue(bank, "all").map((item) => item.id)).toEqual([
            "q-1",
            "q-2",
            "q-3",
            "v-0",
            "v-1",
            "v-2",
            "v-3",
            "q-0",
        ]);

        random.mockRestore();
    });

    test("section-specific practice preserves backend order", () => {
        const bank = [
            question("q-2", "gre::quant::algebra::linear", "quant"),
            question("q-0", "gre::quant::algebra::linear", "quant"),
            question("q-1", "gre::verbal::reading::main", "verbal"),
        ];

        expect(buildQuestionQueue(bank, "quant").map((item) => item.id)).toEqual([
            "q-2",
            "q-0",
        ]);
    });

    test("topic filter uses larger AWA sessions", () => {
        const bank = Array.from({ length: 12 }, (_, index) =>
            question(`arg-${index}`, "gre::awa::argument", "awa"),
        );

        expect(
            buildQuestionQueue(bank, "all", {
                topicFilter: "gre::awa::argument",
            }),
        ).toHaveLength(AWA_TOPIC_PRACTICE_SESSION_SIZE);
    });

    test("topic filter limits non-AWA practice to eight topic questions", () => {
        const bank = [
            ...Array.from({ length: 10 }, (_, index) => question(`lin-${index}`, "gre::quant::algebra::linear", "quant")),
        ];

        expect(
            buildQuestionQueue(bank, "all", {
                topicFilter: "gre::quant::algebra::linear",
            }),
        ).toHaveLength(TOPIC_PRACTICE_SESSION_SIZE);
    });
});
