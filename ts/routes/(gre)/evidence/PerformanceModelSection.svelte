<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { PerformanceEvalResponse } from "@generated/anki/brainlift_pb";
    import type { GraphBounds } from "../../graphs/graph-helpers";
    import { defaultGraphBounds } from "../../graphs/graph-helpers";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import ProgressChart from "../progress/ProgressChart.svelte";
    import {
        performanceTopicAccuracyChartHeight,
        presentPerformanceModel,
    } from "./performance-model-presentation";
    import {
        renderConfusionMatrixChart,
        renderPerformanceTopicAccuracyChart,
    } from "./charts";

    export let response: PerformanceEvalResponse | null = undefined;

    $: model = presentPerformanceModel(response);
    $: topicChartBounds = {
        ...defaultGraphBounds(),
        width: 960,
        height: model.available
            ? performanceTopicAccuracyChartHeight(model.topicRows.length)
            : 180,
        marginLeft: 8,
        marginRight: 8,
        marginTop: 8,
        marginBottom: 8,
    };
    $: confusionBounds = {
        ...defaultGraphBounds(),
        width: 420,
        height: 360,
        marginLeft: 24,
        marginRight: 24,
        marginTop: 16,
        marginBottom: 32,
    };

    function renderTopicChart(svg: SVGElement, bounds: GraphBounds): void {
        if (!model.available) {
            return;
        }
        renderPerformanceTopicAccuracyChart(svg, bounds, model.topicRows);
    }

    function renderConfusionChart(svg: SVGElement, bounds: GraphBounds): void {
        if (!model.available) {
            return;
        }
        renderConfusionMatrixChart(svg, bounds, model.confusion);
    }
</script>

<section class="evidence-section" aria-labelledby="evidence-performance-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-performance-heading">Performance Model</h2>
        <p class="evidence-section-lead">
            Held-out GRE practice questions evaluate whether topic-stratified accuracy
            predictions match observed outcomes.
        </p>
    </header>

    {#if !model.available}
        <p class="evidence-section-empty">{model.emptyMessage}</p>
    {:else}
        <div class="evidence-metrics">
            <GreMetricRow label="Held-out GRE questions" value={String(model.heldOutQuestions)} />
            <GreMetricRow label="Accuracy" value={model.accuracy} />
            <GreMetricRow label="Confidence interval" value={model.confidenceInterval} />
            <GreMetricRow
                label="Questions evaluated"
                value={String(model.questionsEvaluated)}
            />
        </div>

        {#if model.assessment}
            <p class="evidence-section-assessment">{model.assessment}</p>
        {/if}

        <div class="evidence-charts">
            <ProgressChart
                title="Accuracy by topic"
                subtitle="Held-out practice accuracy grouped by GRE topic."
                ariaLabel="Held-out accuracy by topic"
                renderChart={renderTopicChart}
                bounds={topicChartBounds}
                wide
                tall
                extraTall={model.topicRows.length > 6}
                scrollable={model.topicRows.length > 8}
                fluid
            />

            <ProgressChart
                title="Confusion matrix"
                subtitle="Positive class = correct answer."
                ariaLabel="Held-out prediction confusion matrix"
                renderChart={renderConfusionChart}
                bounds={confusionBounds}
                wide
                tall
            />
        </div>
    {/if}
</section>
