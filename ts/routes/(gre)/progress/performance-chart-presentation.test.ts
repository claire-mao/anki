// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceChartBucket } from "@generated/anki/brainlift_pb";
import { describe, expect, test } from "vitest";

import {
    performanceChartAxisLabel,
    performanceChartHasData,
    performanceChartHorizonProto,
    performanceChartLineSegments,
    performanceChartTooltip,
    performanceChartTooltipLines,
} from "./performance-chart-presentation";
import { PerformanceChartHorizon } from "@generated/anki/brainlift_pb";

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

describe("performance chart presentation", () => {
    test("maps UI horizons to backend proto values", () => {
        expect(performanceChartHorizonProto("1d")).toBe(
            PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_1D,
        );
        expect(performanceChartHorizonProto("3d")).toBe(
            PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_3D,
        );
        expect(performanceChartHorizonProto("7d")).toBe(
            PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_7D,
        );
        expect(performanceChartHorizonProto("30d")).toBe(
            PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_30D,
        );
        expect(performanceChartHorizonProto("all")).toBe(
            PerformanceChartHorizon.PERFORMANCE_CHART_HORIZON_ALL,
        );
    });

    test("detects plotted data from backend bucket counts", () => {
        expect(performanceChartHasData([])).toBe(false);
        expect(
            performanceChartHasData([
                bucket({ label: "Jul 1", questions: 0 }),
                bucket({ label: "Jul 2", questions: 0 }),
            ]),
        ).toBe(false);
        expect(
            performanceChartHasData([
                bucket({ label: "Jul 1", questions: 0 }),
                bucket({ label: "Jul 2", questions: 4, correct: 3, incorrect: 1, accuracy: 0.75 }),
            ]),
        ).toBe(true);
    });

    test("formats tooltip content from backend bucket fields", () => {
        expect(
            performanceChartTooltipLines(
                bucket({
                    label: "Jul 4",
                    questions: 12,
                    correct: 10,
                    incorrect: 2,
                    accuracy: 0.833,
                }),
            ),
        ).toEqual([
            "Jul 4",
            "Accuracy: 83%",
            "Correct: 10",
            "Incorrect: 2",
            "Questions: 12",
        ]);
        expect(
            performanceChartTooltip(
                bucket({
                    label: "Jul 4",
                    questions: 12,
                    correct: 10,
                    incorrect: 2,
                    accuracy: 0.833,
                }),
            ),
        ).toBe("Jul 4\nAccuracy: 83%\nCorrect: 10\nIncorrect: 2\nQuestions: 12");
    });

    test("formats hourly x-axis labels as local hour integers", () => {
        const midnight = 1_700_000_000n;
        const noon = midnight + 12n * 3600n;
        const elevenPm = midnight + 23n * 3600n;

        expect(
            performanceChartAxisLabel(
                bucket({ label: "legacy", startSecs: midnight }),
                24,
            ),
        ).toBe(`${new Date(Number(midnight) * 1000).getHours()}`);
        expect(
            performanceChartAxisLabel(
                bucket({ label: "12", startSecs: noon }),
                24,
            ),
        ).toBe(`${new Date(Number(noon) * 1000).getHours()}`);
        expect(
            performanceChartAxisLabel(
                bucket({ label: "23", startSecs: elevenPm }),
                24,
            ),
        ).toBe(`${new Date(Number(elevenPm) * 1000).getHours()}`);
    });

    test("keeps backend labels for non-hourly horizons", () => {
        expect(
            performanceChartAxisLabel(
                bucket({ label: "Jul 4", startSecs: 1_700_000_000n }),
                30,
            ),
        ).toBe("Jul 4");
    });

    test("splits line segments around empty buckets", () => {
        const segments = performanceChartLineSegments([
            bucket({ label: "1", questions: 2, correct: 1, incorrect: 1, accuracy: 0.5 }),
            bucket({ label: "2", questions: 0 }),
            bucket({ label: "3", questions: 1, correct: 1, incorrect: 0, accuracy: 1 }),
            bucket({ label: "4", questions: 2, correct: 2, incorrect: 0, accuracy: 1 }),
        ]);

        expect(segments).toHaveLength(2);
        expect(segments[0]!.map((row) => row.label)).toEqual(["1"]);
        expect(segments[1]!.map((row) => row.label)).toEqual(["3", "4"]);
    });
});
