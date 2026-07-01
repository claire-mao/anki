<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ReadinessCalibrationBin } from "@generated/anki/brainlift_pb";

    import AbstentionRequirements from "../AbstentionRequirements.svelte";
    import { formatPercent, formatRange } from "../score-format";
    import type { PageData } from "./$types";

    export let data: PageData;

    const response = data.response;
    const readiness = response.readiness!;
    const calibration = response.calibration!;

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString();
    }

    function binLabel(bin: ReadinessCalibrationBin): string {
        return `${Math.round(bin.binLow)}–${Math.round(bin.binHigh)}%`;
    }
</script>

<h1>Readiness</h1>

<p class="muted study-plan-summary">{calibration.assessment}</p>
<p class="muted dashboard-updated">
    Last updated {formatTimestampMillis(response.computedAtMillis)}
</p>

<div class="score-grid">
    <div class="score-card">
        <h2>Readiness</h2>
        {#if readiness.sufficientData && readiness.projectedScore !== undefined}
            <div class="score-value">{formatPercent(readiness.projectedScore)}</div>
            {#if formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                <p class="score-range">
                    {formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                </p>
            {/if}
            <p class="muted">{readiness.evidenceSummary}</p>
            {#if readiness.confidenceLevel}
                <p class="muted">
                    Model confidence: {readiness.confidenceLevel}
                    {#if readiness.calibrationSufficientData}
                        · calibration {readiness.calibrationWellCalibrated
                            ? "verified"
                            : "poor"} on held-out data
                    {/if}
                </p>
            {/if}
        {:else}
            <p class="score-abstain">{readiness.abstainReason}</p>
            <AbstentionRequirements
                requirements={readiness.abstentionRequirements}
                heading="Readiness requires more evidence"
            />
            <p class="muted">{readiness.evidenceSummary}</p>
        {/if}
        {#if readiness.calibrationNote}
            <p class="calibration-note">{readiness.calibrationNote}</p>
        {/if}
    </div>
</div>

<div class="gre-panel">
    <h2>Calibration statistics</h2>
    <dl class="coverage-stats">
        <div>
            <dt>Predictions recorded</dt>
            <dd>{calibration.totalPredictions}</dd>
        </div>
        <div>
            <dt>Resolved outcomes</dt>
            <dd>{calibration.resolvedOutcomes}</dd>
        </div>
        <div>
            <dt>Held-out pairs</dt>
            <dd>{calibration.heldOutCount}</dd>
        </div>
        {#if calibration.brierScore !== undefined}
            <div>
                <dt>Brier score</dt>
                <dd>{calibration.brierScore.toFixed(3)}</dd>
            </div>
        {/if}
        {#if calibration.meanAbsoluteError !== undefined}
            <div>
                <dt>Mean absolute error</dt>
                <dd>{calibration.meanAbsoluteError.toFixed(1)} pts</dd>
            </div>
        {/if}
        <div>
            <dt>Well calibrated</dt>
            <dd>{calibration.wellCalibrated ? "yes" : "no"}</dd>
        </div>
    </dl>
</div>

<div class="gre-panel">
    <h2>Calibration curve</h2>
    {#if calibration.calibrationCurve.length === 0}
        <p class="muted">{calibration.assessment}</p>
    {:else}
        <table class="calibration-table">
            <thead>
                <tr>
                    <th>Predicted bin</th>
                    <th>Mean predicted</th>
                    <th>Mean outcome</th>
                    <th>Count</th>
                </tr>
            </thead>
            <tbody>
                {#each calibration.calibrationCurve as bin}
                    <tr>
                        <td>{binLabel(bin)}</td>
                        <td>{formatPercent(bin.predictedMean)}</td>
                        <td>{formatPercent(bin.outcomeMean)}</td>
                        <td>{bin.count}</td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>

<div class="gre-panel study-plan-actions">
    <a class="btn btn-primary" href="/dashboard">Dashboard</a>
    <a class="btn" href="/practice">Practice</a>
    <a class="btn" href="/study-plan">Study plan</a>
</div>
