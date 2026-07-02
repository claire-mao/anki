<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ReadinessCalibrationStats, ReadinessScore } from "@generated/anki/brainlift_pb";

    import { presentCalibration } from "../calibration-presentation";
    import GreConfidenceIndicator from "./GreConfidenceIndicator.svelte";
    import GreEmptyState from "./GreEmptyState.svelte";
    import GreSparkline from "./GreSparkline.svelte";
    import { emptyStateContent } from "../empty-states";

    export let readiness: ReadinessScore;
    export let calibration: ReadinessCalibrationStats;
    export let variant: "full" | "compact" = "full";
    export let showImprovements = true;

    $: model = presentCalibration(readiness, calibration);
</script>

<section
    class="gre-calibration"
    class:gre-calibration-compact={variant === "compact"}
    aria-label="Prediction calibration"
>
    {#if variant === "full"}
        <header class="gre-calibration-header">
            <h2 class="gre-calibration-title">Prediction calibration</h2>
            <p class="gre-calibration-lead">
                How accurately BrainLift predicts your readiness, and what shapes confidence.
            </p>
        </header>
    {/if}

    <div class="gre-calibration-grid">
        <article class="gre-calibration-metric">
            <h3 class="gre-calibration-metric-label">Current confidence</h3>
            <GreConfidenceIndicator confidence={model.currentConfidence} />
            <p class="gre-calibration-metric-detail">{model.confidenceCaption}</p>
        </article>

        <article class="gre-calibration-metric">
            <h3 class="gre-calibration-metric-label">Historical accuracy</h3>
            <p class="gre-calibration-metric-value">{model.historicalAccuracy}</p>
            <p class="gre-calibration-metric-detail">{model.historicalAccuracyDetail}</p>
        </article>

        <article class="gre-calibration-metric">
            <h3 class="gre-calibration-metric-label">Prediction quality</h3>
            <p class="gre-calibration-metric-value">{model.predictionQuality}</p>
            <p class="gre-calibration-metric-detail">{model.predictionQualityDetail}</p>
        </article>

        <article class="gre-calibration-metric gre-calibration-metric-trend">
            <h3 class="gre-calibration-metric-label">Calibration trend</h3>
            {#if model.trendAvailable}
                <GreSparkline
                    points={model.trendPoints}
                    label="Calibration trend across predicted bins"
                    width={variant === "compact" ? 96 : 120}
                    height={32}
                />
            {:else}
                <p class="gre-calibration-metric-value gre-calibration-metric-muted">Building</p>
            {/if}
            <p class="gre-calibration-metric-detail">{model.trendCaption}</p>
        </article>
    </div>

    {#if model.confidenceChangeNotes.length > 0}
        <section class="gre-calibration-section" aria-label="How confidence changes">
            <h3 class="gre-calibration-section-title">How confidence changes</h3>
            <ul class="gre-calibration-notes">
                {#each model.confidenceChangeNotes as note}
                    <li>{note}</li>
                {/each}
            </ul>
        </section>
    {/if}

    {#if showImprovements && model.improvementItems.length > 0}
        <section class="gre-calibration-section" aria-label="What improves confidence">
            <h3 class="gre-calibration-section-title">What improves confidence</h3>
            <ul class="gre-calibration-improvements">
                {#each model.improvementItems as item (item.id)}
                    <li class:gre-calibration-improvement-met={item.met}>
                        <span class="gre-calibration-improvement-mark" aria-hidden="true">
                            {item.met ? "✓" : "○"}
                        </span>
                        <span class="gre-calibration-improvement-copy">
                            <span class="gre-calibration-improvement-label">{item.label}</span>
                            <span class="gre-calibration-improvement-detail">{item.detail}</span>
                        </span>
                    </li>
                {/each}
            </ul>
        </section>
    {/if}

    {#if variant === "full"}
        <details class="gre-calibration-details">
            <summary>Calibration bins</summary>
            {#if calibration.calibrationCurve.length === 0}
                <GreEmptyState content={emptyStateContent("calibrationTable")} compact />
            {:else}
                <div class="gre-calibration-table-wrap">
                    <table class="calibration-table">
                        <caption class="sr-only">
                            Predicted score bins compared with observed outcomes
                        </caption>
                        <thead>
                            <tr>
                                <th scope="col">Predicted bin</th>
                                <th scope="col">Predicted</th>
                                <th scope="col">Observed</th>
                                <th scope="col">Count</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each calibration.calibrationCurve as bin}
                                {#if bin.count > 0}
                                    <tr>
                                        <td>{Math.round(bin.binLow)}–{Math.round(bin.binHigh)}%</td>
                                        <td>{Math.round(bin.predictedMean)}%</td>
                                        <td>{Math.round(bin.outcomeMean)}%</td>
                                        <td>{bin.count}</td>
                                    </tr>
                                {/if}
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
            {#if model.assessment}
                <p class="gre-calibration-assessment">{model.assessment}</p>
            {/if}
        </details>
    {:else if model.assessment}
        <p class="gre-calibration-assessment">{model.assessment}</p>
    {/if}
</section>

<style lang="scss">
    .gre-calibration {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
        padding: var(--gre-card-padding);
        border: none;
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg);
        box-shadow: var(--gre-shadow-md);
    }

    .gre-calibration-compact {
        gap: var(--gre-space-3);
        padding: var(--gre-card-padding-compact);
        box-shadow: var(--gre-shadow-sm);
        background: var(--gre-surface-bg-muted);
    }

    .gre-calibration-header {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-calibration-title {
        margin: 0;
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h2);
        line-height: var(--gre-lh-h2);
        letter-spacing: -0.01em;
    }

    .gre-calibration-lead {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-calibration-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: var(--gre-space-3);
    }

    .gre-calibration-compact .gre-calibration-grid {
        gap: var(--gre-space-2);
    }

    .gre-calibration-metric {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        min-width: 0;
        padding: var(--gre-card-padding-compact);
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg-muted);
        box-shadow: var(--gre-shadow-sm);
    }

    .gre-calibration-compact .gre-calibration-metric {
        padding: var(--gre-space-3);
    }

    .gre-calibration-metric-label {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .gre-calibration-metric-value {
        margin: 0;
        font-size: var(--gre-font-kpi);
        font-weight: var(--gre-weight-h1);
        line-height: var(--gre-lh-h1);
        letter-spacing: -0.02em;
        color: var(--fg);
    }

    .gre-calibration-metric-muted {
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg-subtle);
    }

    .gre-calibration-metric-detail {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-calibration-metric-trend :global(.gre-sparkline) {
        max-width: 100%;
    }

    .gre-calibration-section {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .gre-calibration-section-title {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .gre-calibration-notes,
    .gre-calibration-improvements {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .gre-calibration-notes li {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .gre-calibration-improvements li {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: var(--gre-space-2);
        align-items: start;
    }

    .gre-calibration-improvement-mark {
        color: var(--fg-subtle);
        font-weight: var(--gre-weight-h1);
    }

    .gre-calibration-improvement-met .gre-calibration-improvement-mark {
        color: var(--state-new);
    }

    .gre-calibration-improvement-copy {
        display: flex;
        flex-direction: column;
        gap: 0.125rem;
        min-width: 0;
    }

    .gre-calibration-improvement-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .gre-calibration-improvement-detail {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-calibration-details {
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
        padding-top: var(--gre-space-2);
    }

    .gre-calibration-details summary {
        cursor: pointer;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg-link);
        list-style: none;
    }

    .gre-calibration-details summary::-webkit-details-marker {
        display: none;
    }

    .gre-calibration-table-wrap {
        overflow-x: auto;
        margin-top: var(--gre-space-3);
    }

    .gre-calibration-assessment {
        margin: var(--gre-space-3) 0 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }

    @media (max-width: 767px) {
        .gre-calibration-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
