// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { axisBottom, max, scaleBand, scaleLinear, select } from "d3";

import type { GraphBounds } from "../../graphs/graph-helpers";
import { setDataAvailable } from "../../graphs/graph-helpers";
import { chartEmptyLabel } from "../empty-states";
import type {
    PerformanceConfusionMatrixPresentation,
    PerformanceTopicAccuracyRow,
} from "./performance-model-presentation";

const TOPIC_LABEL_WIDTH_RATIO = 0.34;
const TOPIC_BAR_GAP = 8;

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

function topicLabelWidth(chartWidth: number): number {
    return Math.round(chartWidth * TOPIC_LABEL_WIDTH_RATIO);
}

function topicBarAreaWidth(chartWidth: number): number {
    return Math.max(
        chartWidth - topicLabelWidth(chartWidth) - 56,
        120,
    );
}

export function renderPerformanceTopicAccuracyChart(
    svg: SVGElement,
    bounds: GraphBounds,
    rows: PerformanceTopicAccuracyRow[],
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);

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

    const barX = topicLabelWidth(area.width);
    const barWidth = topicBarAreaWidth(area.width);
    const percentX = barX + barWidth;

    const y = scaleBand<string>()
        .domain(rows.map((row) => row.topicId))
        .range([0, area.height])
        .paddingInner(TOPIC_BAR_GAP / 36)
        .paddingOuter(0.05);

    const x = scaleLinear().domain([0, 100]).range([0, barWidth]).nice();

    g.selectAll("text.label")
        .data(rows)
        .join("text")
        .attr("class", "label")
        .attr("x", 0)
        .attr("y", (row) => y(row.topicId)! + y.bandwidth() / 2 + 5)
        .attr("text-anchor", "start")
        .attr("font-size", "14px")
        .text((row) => row.displayName)
        .append("title")
        .text((row) => row.tooltip);

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
        .append("title")
        .text((row) => row.tooltip);

    g.selectAll("rect.bar")
        .data(rows)
        .join("rect")
        .attr("class", "bar bar-studied")
        .attr("x", barX)
        .attr("y", (row) => y(row.topicId)!)
        .attr("width", (row) => x(row.accuracyPercent))
        .attr("height", y.bandwidth())
        .attr("rx", 8)
        .attr("fill", "var(--state-learn)")
        .append("title")
        .text((row) => row.tooltip);

    g.selectAll("text.value")
        .data(rows)
        .join("text")
        .attr("class", "value value-studied")
        .attr("x", percentX)
        .attr("y", (row) => y(row.topicId)! + y.bandwidth() / 2 + 5)
        .attr("text-anchor", "end")
        .attr("font-size", "13px")
        .text((row) => `${Math.round(row.accuracyPercent)}%`)
        .append("title")
        .text((row) => row.tooltip);
}

type ConfusionCell = {
    key: string;
    label: string;
    count: number;
    row: number;
    col: number;
};

function confusionCells(matrix: PerformanceConfusionMatrixPresentation): ConfusionCell[] {
    return [
        {
            key: "tp",
            label: "Predicted correct · Actual correct",
            count: matrix.truePositive,
            row: 0,
            col: 0,
        },
        {
            key: "fn",
            label: "Predicted incorrect · Actual correct",
            count: matrix.falseNegative,
            row: 0,
            col: 1,
        },
        {
            key: "fp",
            label: "Predicted correct · Actual incorrect",
            count: matrix.falsePositive,
            row: 1,
            col: 0,
        },
        {
            key: "tn",
            label: "Predicted incorrect · Actual incorrect",
            count: matrix.trueNegative,
            row: 1,
            col: 1,
        },
    ];
}

export function renderConfusionMatrixChart(
    svg: SVGElement,
    bounds: GraphBounds,
    matrix: PerformanceConfusionMatrixPresentation,
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);
    const cells = confusionCells(matrix);
    const hasData = matrix.total > 0;

    setDataAvailable(select(svg), hasData);

    if (!hasData) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(chartEmptyLabel("calibration"));
        return;
    }

    const labelHeight = 28;
    const gridHeight = area.height - labelHeight - 24;
    const gridWidth = Math.min(area.width, gridHeight + 120);
    const offsetX = (area.width - gridWidth) / 2;
    const cellSize = gridWidth / 2;

    const maxCount = max(cells, (cell) => cell.count) ?? 1;
    const color = scaleLinear<string>()
        .domain([0, maxCount])
        .range(["color-mix(in srgb, var(--state-learn) 18%, var(--canvas))", "var(--state-learn)"]);

    g.append("text")
        .attr("x", offsetX + gridWidth / 2)
        .attr("y", 14)
        .attr("text-anchor", "middle")
        .attr("class", "chart-caption")
        .text("Predicted outcome");

    g.append("text")
        .attr("x", 8)
        .attr("y", labelHeight + gridHeight / 2)
        .attr("text-anchor", "middle")
        .attr("transform", `rotate(-90 8 ${labelHeight + gridHeight / 2})`)
        .attr("class", "chart-caption")
        .text("Actual outcome");

    const grid = g.append("g").attr(
        "transform",
        `translate(${offsetX}, ${labelHeight})`,
    );

    grid
        .selectAll("rect.cell")
        .data(cells)
        .join("rect")
        .attr("class", "cell")
        .attr("x", (cell) => cell.col * cellSize + 1)
        .attr("y", (cell) => cell.row * cellSize + 1)
        .attr("width", cellSize - 2)
        .attr("height", cellSize - 2)
        .attr("rx", 10)
        .attr("fill", (cell) => color(cell.count))
        .append("title")
        .text((cell) => `${cell.label}: ${cell.count}`);

    grid
        .selectAll("text.count")
        .data(cells)
        .join("text")
        .attr("class", "chart-value-label")
        .attr("x", (cell) => cell.col * cellSize + cellSize / 2)
        .attr("y", (cell) => cell.row * cellSize + cellSize / 2 + 6)
        .attr("text-anchor", "middle")
        .text((cell) => String(cell.count));

    const axisY = labelHeight + gridHeight + 16;
    const axis = scaleBand<string>()
        .domain(["Correct", "Incorrect"])
        .range([0, gridWidth])
        .padding(0.08);

    grid
        .append("g")
        .attr("transform", `translate(0, ${axisY - labelHeight})`)
        .call(axisBottom(axis));
}
