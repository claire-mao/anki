<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { MemoryEvalResponse } from "@generated/anki/brainlift_pb";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import CalibrationChart from "../ui/CalibrationChart.svelte";
    import { presentMemoryModel } from "./memory-model-presentation";

    export let response: MemoryEvalResponse | null = undefined;

    $: model = presentMemoryModel(response);
</script>

<section class="evidence-section" aria-labelledby="evidence-memory-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-memory-heading">Memory Model</h2>
        <p class="evidence-section-lead">
            Held-out GRE flashcard reviews evaluate whether FSRS predicted recall
            matches observed recall outcomes.
        </p>
    </header>

    {#if !model.available}
        <p class="evidence-section-empty">{model.emptyMessage}</p>
    {:else}
        <div class="evidence-metrics">
            <GreMetricRow label="Model name" value={model.modelName} />
            <GreMetricRow label="FSRS" value={model.fsrs} />
            <GreMetricRow label="Held-out reviews" value={model.heldOutReviews} />
            <GreMetricRow label="Calibration metric" value={model.calibrationMetric} />
            <GreMetricRow label="Brier score" value={model.brierScore} />
            <GreMetricRow label="Log loss" value={model.logLoss} />
            <GreMetricRow label="Calibration status" value={model.calibrationStatus} />
        </div>

        <CalibrationChart points={model.calibrationCurve} />
    {/if}
</section>
