// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { EstimatedGreScore, PerformanceChartBucket, ReadinessCalibrationBin } from "@generated/anki/brainlift_pb";
import type { TopicMasteryEntry } from "@generated/anki/stats_pb";
import {
    axisBottom,
    axisLeft,
    curveMonotoneX,
    line,
    max,
    scaleBand,
    scaleLinear,
    scalePoint,
    select,
} from "d3";

import type { GraphBounds } from "../../graphs/graph-helpers";
import { setDataAvailable } from "../../graphs/graph-helpers";
import { chartEmptyLabel } from "../empty-states";
import {
    performanceChartAccuracyPercent,
    performanceChartAxisLabel,
    performanceChartHasData,
    performanceChartLineSegments,
    performanceChartTooltip,
} from "./performance-chart-presentation";
import {
    isTopicMasteryStarted,
    topicDisplayMasteryPercent,
    topicMasteryConfidenceLabel,
    topicMasteryRowTooltip,
} from "./topic-mastery-presentation";

export interface ScoreBarDatum {
    label: string;
    value?: number;
    low?: number;
    high?: number;
    abstain?: string;
    detail?: string;
    color: string;
}

function chartArea(bounds: GraphBounds) {
    return {
        width: bounds.width - bounds.marginLeft - bounds.marginRight,
        height: bounds.height - bounds.marginTop - bounds.marginBottom,
        marginLeft: bounds.marginLeft,
        marginTop: bounds.marginTop,
    };
}

function clearChart(svg: SVGElement): void {
    const root = select(svg);
    root.selectAll("*").remove();
    root.append("g").attr("class", "chart-root");
}

function rootGroup(svg: SVGElement, bounds: GraphBounds) {
    const area = chartArea(bounds);
    return select(svg).select(".chart-root").attr(
        "transform",
        `translate(${area.marginLeft}, ${area.marginTop})`,
    );
}

export function renderScoreBar(
    svg: SVGElement,
    bounds: GraphBounds,
    datum: ScoreBarDatum,
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);
    const hasValue = datum.value !== undefined;

    setDataAvailable(select(svg), hasValue);

    if (!hasValue) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(datum.abstain ?? chartEmptyLabel("score"));
        return;
    }

    const value = datum.value!;
    const y = area.height / 2 - 16;
    const x = scaleLinear().domain([0, 100]).range([0, area.width]).nice();

    g.append("g")
        .attr("transform", `translate(0, ${y})`)
        .call(axisBottom(x).ticks(5).tickFormat((d) => `${d}%`));

    g.append("rect")
        .attr("x", 0)
        .attr("y", y - 36)
        .attr("width", x(value))
        .attr("height", 28)
        .attr("rx", 8)
        .attr("fill", datum.color);

    if (datum.low !== undefined && datum.high !== undefined) {
        const lowX = x(datum.low);
        const highX = x(datum.high);
        g.append("line")
            .attr("x1", lowX)
            .attr("x2", highX)
            .attr("y1", y - 22)
            .attr("y2", y - 22)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
        g.append("line")
            .attr("x1", lowX)
            .attr("x2", lowX)
            .attr("y1", y - 28)
            .attr("y2", y - 16)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
        g.append("line")
            .attr("x1", highX)
            .attr("x2", highX)
            .attr("y1", y - 28)
            .attr("y2", y - 16)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
    }

    g.append("text")
        .attr("x", x(value) + 8)
        .attr("y", y - 18)
        .attr("class", "chart-value-label")
        .text(`${Math.round(value)}%`);
}

export function renderEstimatedGreScore(
    svg: SVGElement,
    bounds: GraphBounds,
    estimate: EstimatedGreScore,
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);
    const hasValue = estimate.combinedScore !== undefined;

    setDataAvailable(select(svg), hasValue);

    if (!hasValue) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(chartEmptyLabel("estimatedGre"));
        return;
    }

    const value = estimate.combinedScore!;
    const y = area.height / 2 - 16;
    const x = scaleLinear().domain([260, 340]).range([0, area.width]).nice();

    g.append("g")
        .attr("transform", `translate(0, ${y})`)
        .call(axisBottom(x).ticks(5).tickFormat((d) => `${d}`));

    g.append("rect")
        .attr("x", x(260))
        .attr("y", y - 36)
        .attr("width", Math.max(x(value) - x(260), 0))
        .attr("height", 28)
        .attr("rx", 8)
        .attr("fill", "var(--state-new)");

    if (estimate.combinedScoreLow !== undefined && estimate.combinedScoreHigh !== undefined) {
        const lowX = x(estimate.combinedScoreLow);
        const highX = x(estimate.combinedScoreHigh);
        g.append("line")
            .attr("x1", lowX)
            .attr("x2", highX)
            .attr("y1", y - 22)
            .attr("y2", y - 22)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
        g.append("line")
            .attr("x1", lowX)
            .attr("x2", lowX)
            .attr("y1", y - 28)
            .attr("y2", y - 16)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
        g.append("line")
            .attr("x1", highX)
            .attr("x2", highX)
            .attr("y1", y - 28)
            .attr("y2", y - 16)
            .attr("stroke", "var(--fg-subtle)")
            .attr("stroke-width", 2);
    }

    g.append("text")
        .attr("x", x(value) + 8)
        .attr("y", y - 18)
        .attr("class", "chart-value-label")
        .text(String(Math.round(value)));

    if (estimate.quantScore !== undefined && estimate.verbalScore !== undefined) {
        g.append("text")
            .attr("x", 0)
            .attr("y", y + 28)
            .attr("class", "chart-caption")
            .text(
                `Quant ${Math.round(estimate.quantScore)} · Verbal ${Math.round(estimate.verbalScore)}`,
            );
    }
}

export function renderPerformanceChart(
    svg: SVGElement,
    bounds: GraphBounds,
    buckets: PerformanceChartBucket[],
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);
    const hasData = performanceChartHasData(buckets);

    setDataAvailable(select(svg), hasData);

    if (!hasData || buckets.length === 0) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text("Not enough practice data for this period");
        return;
    }

    const labels = buckets.map((bucket) => bucket.label);
    const axisLabelByBucket = new Map(
        buckets.map((bucket) => [
            bucket.label,
            performanceChartAxisLabel(bucket, buckets.length),
        ]),
    );
    const x = scalePoint<string>()
        .domain(labels)
        .range([0, area.width])
        .padding(0.5);
    const y = scaleLinear().domain([0, 100]).range([area.height, 0]).nice();
    // Responsive tick thinning: fit roughly one label per 46px (92px for the
    // longer weekly/monthly labels), always keeping the most recent period.
    const longLabels = buckets.some(
        (bucket) => (axisLabelByBucket.get(bucket.label) ?? "").length > 6,
    );
    const maxTicks = Math.max(4, Math.floor(area.width / (longLabels ? 92 : 46)));
    const step = Math.max(1, Math.ceil(buckets.length / maxTicks));
    const tickLabels =
        step === 1
            ? labels
            : labels.filter(
                  (_label, index) => index % step === 0 || index === labels.length - 1,
              );
    const rotateLabels = longLabels;

    g.append("g")
        .attr("transform", `translate(0, ${area.height})`)
        .call(
            axisBottom(x)
                .tickValues(tickLabels)
                .tickFormat((value) => axisLabelByBucket.get(String(value)) ?? String(value)),
        )
        .selectAll("text")
        .attr("transform", rotateLabels ? "rotate(-35)" : null)
        .attr("text-anchor", rotateLabels ? "end" : "middle")
        .attr("dx", rotateLabels ? "-0.4em" : null)
        .attr("dy", rotateLabels ? "0.2em" : null);

    g.append("g").call(axisLeft(y).ticks(5).tickFormat((value) => `${value}%`));

    g.append("text")
        .attr("transform", "rotate(-90)")
        .attr("x", -area.height / 2)
        .attr("y", -48)
        .attr("text-anchor", "middle")
        .attr("class", "chart-axis-label")
        .text("Accuracy %");

    g.append("text")
        .attr("x", area.width / 2)
        .attr("y", area.height + 38)
        .attr("text-anchor", "middle")
        .attr("class", "chart-axis-label")
        .text("Practice over time");

    const trendLine = line<PerformanceChartBucket>()
        .defined(
            (bucket) => bucket.questions > 0 && bucket.accuracy !== undefined,
        )
        .x((bucket) => x(bucket.label)!)
        .y((bucket) => y(performanceChartAccuracyPercent(bucket)))
        .curve(curveMonotoneX);

    for (const segment of performanceChartLineSegments(buckets)) {
        g.append("path")
            .datum(segment)
            .attr("d", trendLine)
            .attr("fill", "none")
            .attr("stroke", "var(--fg-link)")
            .attr("stroke-width", 2.5);
    }

    const plotted = buckets.filter(
        (bucket) => bucket.questions > 0 && bucket.accuracy !== undefined,
    );

    g.selectAll("circle.point")
        .data(plotted)
        .join("circle")
        .attr("class", "point")
        .attr("cx", (bucket) => x(bucket.label)!)
        .attr("cy", (bucket) => y(performanceChartAccuracyPercent(bucket)))
        .attr("r", 4)
        .attr("fill", "var(--fg-link)")
        .append("title")
        .text((bucket) => performanceChartTooltip(bucket));

    // Show review counts with each point when they won't overlap; otherwise the
    // count stays available in the hover tooltip.
    if (plotted.length > 0 && plotted.length <= 10) {
        g.selectAll("text.point-count")
            .data(plotted)
            .join("text")
            .attr("class", "point-count")
            .attr("x", (bucket) => x(bucket.label)!)
            .attr("y", (bucket) => y(performanceChartAccuracyPercent(bucket)) - 9)
            .attr("text-anchor", "middle")
            .attr("font-size", "10px")
            .attr("fill", "var(--fg-subtle)")
            .text((bucket) => `n=${bucket.questions}`);
    }
}

export function studiedTopicEntries(topics: TopicMasteryEntry[]): TopicMasteryEntry[] {
    return catalogLeafTopics(topics).filter((topic) => isStudiedTopic(topic));
}

export function isStudiedTopic(topic: TopicMasteryEntry): boolean {
    return isTopicMasteryStarted(topic);
}

/** Catalog leaves only — parent organizer nodes are excluded from the chart. */
export function catalogLeafTopics(topics: TopicMasteryEntry[]): TopicMasteryEntry[] {
    const ids = new Set(topics.map((topic) => topic.topicId));
    return topics.filter((topic) => {
        const childPrefix = `${topic.topicId}::`;
        for (const id of ids) {
            if (id !== topic.topicId && id.startsWith(childPrefix)) {
                return false;
            }
        }
        return true;
    });
}

export function topicMasteryChartRows(topics: TopicMasteryEntry[]): TopicMasteryEntry[] {
    const leaves = catalogLeafTopics(topics);
    const studied = leaves
        .filter(isStudiedTopic)
        .sort((a, b) => {
            const left = topicDisplayMasteryPercent(a) ?? 0;
            const right = topicDisplayMasteryPercent(b) ?? 0;
            return right - left || a.displayName.localeCompare(b.displayName);
        });
    const unstudied = leaves
        .filter((topic) => !isStudiedTopic(topic))
        .sort((a, b) => a.displayName.localeCompare(b.displayName));
    return [...studied, ...unstudied];
}

const TOPIC_MASTERY_ROW_HEIGHT = 34;
const TOPIC_MASTERY_CHART_VERTICAL_PADDING = 16;
export const TOPIC_MASTERY_PERCENT_WIDTH = 52;
const TOPIC_MASTERY_ROW_GAP = 6;
const TOPIC_MASTERY_LABEL_MIN_WIDTH = 132;
const TOPIC_MASTERY_LABEL_MAX_WIDTH = 220;

export function topicMasteryLabelWidth(chartWidth: number): number {
    return Math.min(
        Math.max(Math.round(chartWidth * 0.26), TOPIC_MASTERY_LABEL_MIN_WIDTH),
        TOPIC_MASTERY_LABEL_MAX_WIDTH,
    );
}

export function topicMasteryBarAreaWidth(chartWidth: number): number {
    const labelWidth = topicMasteryLabelWidth(chartWidth);
    return Math.max(
        chartWidth - labelWidth - TOPIC_MASTERY_PERCENT_WIDTH,
        120,
    );
}

export function topicMasteryChartHeight(rowCount: number): number {
    if (rowCount === 0) {
        return 240;
    }
    return rowCount * (TOPIC_MASTERY_ROW_HEIGHT + TOPIC_MASTERY_ROW_GAP)
        + TOPIC_MASTERY_CHART_VERTICAL_PADDING;
}

export function topicMasteryChartSubtitle(
    catalogTopicCount: number,
    topics: TopicMasteryEntry[],
    topicsStudied?: number,
): string {
    const studiedCount = topicsStudied ?? studiedTopicEntries(topics).length;
    const boundedStudiedCount = Math.min(studiedCount, catalogTopicCount);
    if (boundedStudiedCount === 0) {
        return `${catalogTopicCount} GRE topics in catalog`;
    }
    return `${boundedStudiedCount} of ${catalogTopicCount} topics studied`;
}

export function renderTopicMasteryChart(
    svg: SVGElement,
    bounds: GraphBounds,
    topics: TopicMasteryEntry[],
    onTopicClick?: (topicId: string) => void,
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);

    const rows = topicMasteryChartRows(topics);

    setDataAvailable(select(svg), rows.length > 0);

    if (rows.length === 0) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(chartEmptyLabel("topicMastery"));
        return;
    }

    const barX = topicMasteryLabelWidth(area.width);
    const barWidth = topicMasteryBarAreaWidth(area.width);
    const percentX = barX + barWidth;

    const y = scaleBand<string>()
        .domain(rows.map((row) => row.topicId))
        .range([0, area.height])
        .paddingInner(TOPIC_MASTERY_ROW_GAP / TOPIC_MASTERY_ROW_HEIGHT)
        .paddingOuter(0.05);

    const x = scaleLinear().domain([0, 100]).range([0, barWidth]).nice();
    const rowTooltip = (row: TopicMasteryEntry): string => topicMasteryRowTooltip(row);

    g.selectAll("text.label")
        .data(rows)
        .join("text")
        .attr("class", "label")
        .attr("x", 0)
        .attr("y", (row) => y(row.topicId)! + y.bandwidth() / 2 + 5)
        .attr("text-anchor", "start")
        .attr("font-size", "14px")
        .text((row) => row.displayName)
        .style("cursor", onTopicClick ? "pointer" : "default")
        .on("click", (_event, row) => {
            onTopicClick?.(row.topicId);
        })
        .append("title")
        .text(rowTooltip);

    g.selectAll("rect.track")
        .data(rows)
        .join("rect")
        .attr("class", "track")
        .attr("x", barX)
        .attr("y", (row) => y(row.topicId)!)
        .attr("width", barWidth)
        .attr("height", y.bandwidth())
        .attr("rx", 8)
        .attr("fill", "var(--border)")
        .attr("opacity", (row) => (isStudiedTopic(row) ? 0 : 0.35))
        .style("cursor", onTopicClick ? "pointer" : "default")
        .on("click", (_event, row) => {
            onTopicClick?.(row.topicId);
        })
        .append("title")
        .text(rowTooltip);

    g.selectAll("rect.bar")
        .data(rows)
        .join("rect")
        .attr("class", (row) => (isStudiedTopic(row) ? "bar bar-studied" : "bar bar-unstudied"))
        .attr("x", barX)
        .attr("y", (row) => y(row.topicId)!)
        .attr(
            "width",
            (row) => {
                const percent = topicDisplayMasteryPercent(row);
                if (percent === undefined) {
                    return Math.max(y.bandwidth() * 0.35, 3);
                }
                return x(percent);
            },
        )
        .attr("height", y.bandwidth())
        .attr("rx", 8)
        .attr("fill", (row) => (isStudiedTopic(row) ? "var(--state-learn)" : "var(--fg-subtle)"))
        .attr("opacity", (row) => (isStudiedTopic(row) ? 1 : 0.55))
        .style("cursor", onTopicClick ? "pointer" : "default")
        .on("click", (_event, row) => {
            onTopicClick?.(row.topicId);
        })
        .append("title")
        .text(rowTooltip);

    g.selectAll("text.value")
        .data(rows)
        .join("text")
        .attr("class", (row) => (isStudiedTopic(row) ? "value value-studied" : "value value-unstudied"))
        .attr("x", percentX)
        .attr("y", (row) => y(row.topicId)! + y.bandwidth() / 2 + 5)
        .attr("text-anchor", "end")
        .attr("font-size", "13px")
        .text((row) => {
            const percent = topicDisplayMasteryPercent(row);
            if (percent === undefined) {
                return "Not started";
            }
            const confidence = topicMasteryConfidenceLabel(row);
            return confidence
                ? `${Math.round(percent)}% · ${confidence}`
                : `${Math.round(percent)}%`;
        })
        .append("title")
        .text(rowTooltip);
}

export function renderCalibrationCurve(
    svg: SVGElement,
    bounds: GraphBounds,
    bins: ReadinessCalibrationBin[],
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);

    const rows = bins.filter((bin) => bin.count > 0);
    setDataAvailable(select(svg), rows.length > 0);

    if (rows.length === 0) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(chartEmptyLabel("calibration"));
        return;
    }

    const x = scaleLinear().domain([0, 100]).range([0, area.width]).nice();
    const y = scaleLinear().domain([0, 100]).range([area.height, 0]).nice();

    g.append("g")
        .attr("transform", `translate(0, ${area.height})`)
        .call(axisBottom(x).ticks(5).tickFormat((d) => `${d}%`));

    g.append("g").call(axisLeft(y).ticks(5).tickFormat((d) => `${d}%`));

    const identity = line<[number, number]>()
        .x((d) => x(d[0]))
        .y((d) => y(d[1]));

    g.append("path")
        .attr(
            "d",
            identity([
                [0, 0],
                [100, 100],
            ]),
        )
        .attr("fill", "none")
        .attr("stroke", "var(--border)")
        .attr("stroke-dasharray", "4 4");

    const predictedLine = line<ReadinessCalibrationBin>()
        .x((bin) => x(bin.predictedMean))
        .y((bin) => y(bin.outcomeMean));

    g.append("path")
        .datum(rows)
        .attr("d", predictedLine)
        .attr("fill", "none")
        .attr("stroke", "var(--fg-link)")
        .attr("stroke-width", 2.5);

    g.selectAll("circle")
        .data(rows)
        .join("circle")
        .attr("cx", (bin) => x(bin.predictedMean))
        .attr("cy", (bin) => y(bin.outcomeMean))
        .attr("r", (bin) => 4 + Math.min(bin.count, 8))
        .attr("fill", "var(--state-new)")
        .attr("opacity", 0.85);

    const maxCount = max(rows, (bin) => bin.count) ?? 1;
    g.append("text")
        .attr("x", area.width - 8)
        .attr("y", 12)
        .attr("text-anchor", "end")
        .attr("class", "chart-caption")
        .text(`Up to ${maxCount} outcomes per bin`);
}
