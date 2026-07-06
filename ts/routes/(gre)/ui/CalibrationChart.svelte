<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { GraphBounds } from "../../graphs/graph-helpers";
    import { defaultGraphBounds } from "../../graphs/graph-helpers";
    import type { CalibrationChartPoint } from "../evidence/memory-model-presentation";
    import ProgressChart from "../progress/ProgressChart.svelte";
    import { renderCalibrationChart } from "./calibration-chart";

    export let title = "Calibration chart";
    export let subtitle =
        "Predicted recall compared with observed recall on held-out reviews.";
    export let points: CalibrationChartPoint[] = [];
    export let emptyMessage: string | undefined = undefined;
    export let bounds: GraphBounds = {
        ...defaultGraphBounds(),
        width: 520,
        height: 360,
        marginLeft: 58,
        marginBottom: 56,
    };
    export let wide = true;
    export let tall = true;

    function renderChart(svg: SVGElement, chartBounds: GraphBounds): void {
        renderCalibrationChart(svg, chartBounds, points, {
            predictedLabel: "Predicted recall",
            actualLabel: "Actual recall",
            emptyMessage,
        });
    }
</script>

<ProgressChart
    {title}
    {subtitle}
    ariaLabel="Memory calibration chart"
    {renderChart}
    {bounds}
    {wide}
    {tall}
/>
