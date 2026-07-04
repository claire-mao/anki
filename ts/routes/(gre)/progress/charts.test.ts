// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { TopicMasteryEntry } from "@generated/anki/stats_pb";
import { describe, expect, test } from "vitest";

import {
    catalogLeafTopics,
    studiedTopicEntries,
    topicMasteryChartHeight,
    topicMasteryChartRows,
    topicMasteryChartSubtitle,
} from "./charts";

function topic(
    id: string,
    name: string,
    studiedCards: number,
    avgRetrievability: number,
): TopicMasteryEntry {
    return {
        topicId: id,
        displayName: name,
        studiedCards,
        avgRetrievability,
    } as TopicMasteryEntry;
}

describe("topic mastery chart data", () => {
    test("subtitle reports studied topics against catalog size", () => {
        const topics = [
            topic("gre::quant::algebra::linear", "Linear equations", 2, 0.95),
            topic("gre::verbal::reading::inference", "Inference", 1, 0.88),
            topic("gre::quant::geometry::triangles", "Triangles", 0, 0),
        ];

        expect(topicMasteryChartSubtitle(21, topics)).toBe("2 of 21 topics studied");
    });

    test("subtitle falls back to catalog size when nothing is studied", () => {
        const topics = [topic("gre::quant::algebra::linear", "Linear equations", 0, 0)];

        expect(topicMasteryChartSubtitle(21, topics)).toBe("21 GRE topics in catalog");
    });

    test("catalogLeafTopics excludes parent organizer nodes", () => {
        const topics = [
            topic("gre::quant", "Quantitative Reasoning", 0, 0),
            topic("gre::quant::algebra", "Algebra", 0, 0),
            topic("gre::quant::algebra::linear", "Linear equations", 1, 0.9),
        ];

        expect(catalogLeafTopics(topics).map((row) => row.topicId)).toEqual([
            "gre::quant::algebra::linear",
        ]);
    });

    test("chart rows include studied and unstudied leaves", () => {
        const topics = [
            topic("a", "A", 1, 0.7),
            topic("b", "B", 2, 0.95),
            topic("c", "C", 0, 0),
            topic("d", "D", 1, 0.8),
            topic("e", "Echo", 0, 0),
        ];

        expect(studiedTopicEntries(topics)).toHaveLength(3);
        expect(topicMasteryChartRows(topics).map((row) => row.displayName)).toEqual([
            "B",
            "D",
            "A",
            "C",
            "Echo",
        ]);
    });

    test("chart height scales with row count", () => {
        expect(topicMasteryChartHeight(0)).toBe(200);
        expect(topicMasteryChartHeight(21)).toBe(21 * 22 + 56);
    });
});
