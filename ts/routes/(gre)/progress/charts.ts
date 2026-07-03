// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { EstimatedGreScore, ReadinessCalibrationBin } from "@generated/anki/brainlift_pb";
import type { TopicMasteryEntry } from "@generated/anki/stats_pb";
import { axisBottom, axisLeft, line, max, scaleBand, scaleLinear, select } from "d3";

import type { GraphBounds } from "../../graphs/graph-helpers";
import { setDataAvailable } from "../../graphs/graph-helpers";
import { chartEmptyLabel } from "../empty-states";

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

export function renderTopicMasteryChart(
    svg: SVGElement,
    bounds: GraphBounds,
    topics: TopicMasteryEntry[],
    onTopicClick?: (topicId: string) => void,
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);

    const rows = topics
        .filter((topic) => topic.studiedCards > 0)
        .sort((a, b) => b.avgRetrievability - a.avgRetrievability)
        .slice(0, 10);

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

    const y = scaleBand<string>()
        .domain(rows.map((row) => row.displayName))
        .range([0, area.height - 10])
        .padding(0.25);

    const x = scaleLinear()
        .domain([0, 100])
        .range([0, area.width - 120])
        .nice();

    g.append("g").call(axisLeft(y).tickSize(0)).selectAll("text").attr("font-size", "11px");

    g.append("g")
        .attr("transform", `translate(0, ${area.height})`)
        .call(axisBottom(x).ticks(5).tickFormat((d) => `${d}%`));

    g.selectAll("rect.bar")
        .data(rows)
        .join("rect")
        .attr("class", "bar")
        .attr("x", 0)
        .attr("y", (row) => y(row.displayName)!)
        .attr("width", (row) => x(row.avgRetrievability * 100))
        .attr("height", y.bandwidth())
        .attr("rx", 6)
        .attr("fill", "var(--state-learn)")
        .style("cursor", onTopicClick ? "pointer" : "default")
        .on("click", (_event, row) => {
            onTopicClick?.(row.topicId);
        });

    g.selectAll("text.value")
        .data(rows)
        .join("text")
        .attr("class", "value")
        .attr("x", (row) => x(row.avgRetrievability * 100) + 6)
        .attr("y", (row) => y(row.displayName)! + y.bandwidth() / 2 + 4)
        .text((row) => `${Math.round(row.avgRetrievability * 100)}%`);
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
