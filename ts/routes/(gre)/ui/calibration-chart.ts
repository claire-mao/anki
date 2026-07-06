// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { axisBottom, axisLeft, line, max, scaleLinear, select } from "d3";

import type { GraphBounds } from "../../graphs/graph-helpers";
import { setDataAvailable } from "../../graphs/graph-helpers";
import { chartEmptyLabel } from "../empty-states";
import type { CalibrationChartPoint } from "../evidence/memory-model-presentation";

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

function percent(value: number): number {
    return value * 100;
}

export function renderCalibrationChart(
    svg: SVGElement,
    bounds: GraphBounds,
    points: CalibrationChartPoint[],
    options?: {
        predictedLabel?: string;
        actualLabel?: string;
        emptyMessage?: string;
    },
): void {
    clearChart(svg);
    const area = chartArea(bounds);
    const g = rootGroup(svg, bounds);
    const rows = points.filter((point) => point.count > 0);

    setDataAvailable(select(svg), rows.length > 0);

    if (rows.length === 0) {
        g.append("text")
            .attr("x", area.width / 2)
            .attr("y", area.height / 2)
            .attr("text-anchor", "middle")
            .attr("class", "chart-empty-label")
            .text(options?.emptyMessage ?? chartEmptyLabel("calibration"));
        return;
    }

    const x = scaleLinear().domain([0, 100]).range([0, area.width]).nice();
    const y = scaleLinear().domain([0, 100]).range([area.height, 0]).nice();

    g.append("g")
        .attr("transform", `translate(0, ${area.height})`)
        .call(axisBottom(x).ticks(5).tickFormat((d) => `${d}%`));

    g.append("g").call(axisLeft(y).ticks(5).tickFormat((d) => `${d}%`));

    g.append("text")
        .attr("x", area.width / 2)
        .attr("y", area.height + 40)
        .attr("text-anchor", "middle")
        .attr("class", "chart-axis-label")
        .text(options?.predictedLabel ?? "Predicted recall");

    g.append("text")
        .attr("transform", "rotate(-90)")
        .attr("x", -area.height / 2)
        .attr("y", -42)
        .attr("text-anchor", "middle")
        .attr("class", "chart-axis-label")
        .text(options?.actualLabel ?? "Actual recall");

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

    const predictedLine = line<CalibrationChartPoint>()
        .x((point) => x(percent(point.predictedMean)))
        .y((point) => y(percent(point.outcomeMean)));

    g.append("path")
        .datum(rows)
        .attr("d", predictedLine)
        .attr("fill", "none")
        .attr("stroke", "var(--fg-link)")
        .attr("stroke-width", 2.5);

    g.selectAll("circle.predicted")
        .data(rows)
        .join("circle")
        .attr("class", "predicted")
        .attr("cx", (point) => x(percent(point.predictedMean)))
        .attr("cy", (point) => y(percent(point.outcomeMean)))
        .attr("r", (point) => 4 + Math.min(point.count, 8))
        .attr("fill", "var(--state-new)")
        .attr("opacity", 0.85)
        .append("title")
        .text(
            (point) =>
                `Predicted ${percent(point.predictedMean).toFixed(1)}% · Actual ${percent(point.outcomeMean).toFixed(1)}% · ${point.count} reviews`,
        );

    const maxCount = max(rows, (point) => point.count) ?? 1;
    g.append("text")
        .attr("x", area.width - 8)
        .attr("y", 12)
        .attr("text-anchor", "end")
        .attr("class", "chart-caption")
        .text(`Up to ${maxCount} reviews per bin`);
}
