<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import CalibrationChart from "../ui/CalibrationChart.svelte";
    import type { ReadinessModelPresentation } from "./eval-report-presentation";

    export let model: ReadinessModelPresentation;
</script>

<section class="evidence-section" aria-labelledby="evidence-readiness-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-readiness-heading">Readiness Model</h2>
        <p class="evidence-section-lead">
            Held-out readiness predictions evaluate whether projected scores match later
            observed outcomes.
        </p>
    </header>

    {#if !model.available}
        <p class="evidence-section-empty">{model.emptyMessage}</p>
    {:else}
        <div class="evidence-metrics">
            <GreMetricRow label="Held-out predictions" value={model.heldOutPredictions} />
            <GreMetricRow label="Brier score" value={model.brierScore} />
            <GreMetricRow label="Mean absolute error" value={model.meanAbsoluteError} />
            <GreMetricRow label="Well calibrated" value={model.wellCalibrated} />
        </div>

        {#if model.assessment}
            <p class="evidence-section-assessment">{model.assessment}</p>
        {/if}

        <CalibrationChart
            title="Readiness calibration"
            subtitle="Predicted readiness compared with observed outcomes on held-out predictions."
            points={model.calibrationCurve}
            emptyMessage={model.calibrationCurve.length === 0 ? model.assessment : undefined}
        />
    {/if}
</section>
