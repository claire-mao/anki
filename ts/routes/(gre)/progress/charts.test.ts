// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceChartBucket } from "@generated/anki/brainlift_pb";
import type { TopicMasteryEntry } from "@generated/anki/stats_pb";
import { describe, expect, test } from "vitest";

import {
    catalogLeafTopics,
    studiedTopicEntries,
    topicMasteryBarAreaWidth,
    topicMasteryChartHeight,
    topicMasteryChartRows,
    topicMasteryChartSubtitle,
    topicMasteryLabelWidth,
    TOPIC_MASTERY_PERCENT_WIDTH,
} from "./charts";
import {
    performanceChartHasData,
    performanceChartLineSegments,
} from "./performance-chart-presentation";

function bucket(
    partial: Partial<PerformanceChartBucket> & Pick<PerformanceChartBucket, "label">,
): PerformanceChartBucket {
    return {
        startSecs: 0n,
        endSecs: 0n,
        questions: 0,
        correct: 0,
        incorrect: 0,
        ...partial,
    } as PerformanceChartBucket;
}

function topic(
    id: string,
    name: string,
    studiedCards: number,
    avgRetrievability: number,
    extra: Partial<TopicMasteryEntry> = {},
): TopicMasteryEntry {
    return {
        topicId: id,
        displayName: name,
        studiedCards,
        avgRetrievability,
        // Chart mastery now reads displayMastery; default it to the recall arg
        // so these fixtures exercise the blended value the backend supplies.
        displayMastery: avgRetrievability,
        ...extra,
    } as TopicMasteryEntry;
}

describe("performance chart rendering data", () => {
    test("uses backend bucket counts without client-side regrouping", () => {
        const buckets = [
            bucket({ label: "Jul 1", questions: 0 }),
            bucket({ label: "Jul 2", questions: 6, correct: 5, incorrect: 1, accuracy: 5 / 6 }),
            bucket({ label: "Jul 3", questions: 4, correct: 2, incorrect: 2, accuracy: 0.5 }),
        ];

        expect(performanceChartHasData(buckets)).toBe(true);
        expect(performanceChartLineSegments(buckets).flat().map((row) => row.label)).toEqual([
            "Jul 2",
            "Jul 3",
        ]);
    });
});

describe("topic mastery chart data", () => {
    test("subtitle reports studied topics against catalog size", () => {
        const topics = [
            topic("gre::quant::algebra::linear", "Linear equations", 2, 0.95),
            topic("gre::verbal::reading::inference", "Inference", 1, 0.88),
            topic("gre::quant::geometry::triangles", "Triangles", 0, 0),
        ];

        expect(topicMasteryChartSubtitle(21, topics)).toBe("2 of 21 topics studied");
    });

    test("subtitle prefers backend topicsStudied count", () => {
        const topics = [
            topic("gre::quant::algebra::linear", "Linear equations", 2, 0.95),
        ];

        expect(topicMasteryChartSubtitle(21, topics, 2)).toBe("2 of 21 topics studied");
    });

    test("subtitle never exceeds catalog size", () => {
        const topics = [
            topic("gre::quant::algebra::linear", "Linear equations", 2, 0.95),
            topic("gre::verbal::reading::inference", "Inference", 1, 0.88),
        ];

        expect(topicMasteryChartSubtitle(21, topics, 28)).toBe("21 of 21 topics studied");
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

    test("studiedTopicEntries ignores practice-only parent topics", () => {
        const topics = [
            topic("gre::quant", "Quantitative Reasoning", 0, 0, { practiceAttempts: 5 }),
            topic("gre::quant::algebra::linear", "Linear equations", 1, 0.9),
        ];

        expect(studiedTopicEntries(topics)).toHaveLength(1);
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

    test("chart rows sort by FSRS recall", () => {
        const topics = [
            topic("a", "A", 1, 0.7),
            topic("b", "B", 1, 0.95),
        ];

        expect(topicMasteryChartRows(topics).map((row) => row.displayName)).toEqual([
            "B",
            "A",
        ]);
    });

    test("chart height scales with row count", () => {
        expect(topicMasteryChartHeight(0)).toBe(240);
        expect(topicMasteryChartHeight(21)).toBe(21 * (34 + 6) + 16);
    });

    test("bar area uses proportional label and percent gutters", () => {
        const chartWidth = 960;
        const labelWidth = topicMasteryLabelWidth(chartWidth);

        expect(TOPIC_MASTERY_PERCENT_WIDTH).toBeGreaterThan(0);
        expect(labelWidth).toBeGreaterThanOrEqual(132);
        expect(labelWidth).toBeLessThanOrEqual(220);
        expect(topicMasteryBarAreaWidth(chartWidth)).toBe(
            chartWidth - labelWidth - TOPIC_MASTERY_PERCENT_WIDTH,
        );
    });

    test("label width scales down for narrow charts", () => {
        expect(topicMasteryLabelWidth(400)).toBe(132);
        expect(topicMasteryBarAreaWidth(400)).toBeGreaterThanOrEqual(120);
    });
});
