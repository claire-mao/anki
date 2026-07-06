<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ReadinessModelExplanation } from "../readiness-model-presentation";
    import GreConfidenceIndicator from "../ui/GreConfidenceIndicator.svelte";
    import GreMetricRow from "../ui/GreMetricRow.svelte";

    export let model: ReadinessModelExplanation;
</script>

<section class="readiness-model-explain" aria-label="Readiness model explanation">
    <div class="readiness-model-explain-summary">
        {#if model.score}
            <div class="readiness-model-explain-hero">
                <span class="readiness-model-explain-label">Current readiness score</span>
                <span class="readiness-model-explain-score">{model.score}</span>
            </div>
        {/if}

        <div class="readiness-model-explain-metrics">
            <GreMetricRow
                label="Prediction interval"
                value={model.predictionInterval}
                hint="Combined uncertainty from memory and performance intervals."
            />
            <GreMetricRow label="Confidence" hint="Evidence quantity and interval tightness.">
                <GreConfidenceIndicator confidence={model.confidence} />
            </GreMetricRow>
            {#if model.confidenceFactors.length > 0}
                <ul class="readiness-model-explain-factors">
                    {#each model.confidenceFactors as factor}
                        <li>{factor}</li>
                    {/each}
                </ul>
            {/if}
        </div>

        <section
            class="readiness-model-explain-evidence"
            aria-labelledby="readiness-model-evidence-heading"
        >
            <h3 class="readiness-model-explain-section-title" id="readiness-model-evidence-heading">
                Evidence used
            </h3>
            {#if model.evidenceUsed.length > 0}
                <ul class="readiness-model-explain-evidence-list">
                    {#each model.evidenceUsed as line}
                        <li>
                            <span class="readiness-model-explain-evidence-label">{line.label}</span>
                            <span class="readiness-model-explain-evidence-detail">{line.detail}</span>
                        </li>
                    {/each}
                </ul>
            {:else}
                <p class="readiness-model-explain-empty">No evidence recorded yet.</p>
            {/if}
        </section>
    </div>

    <details class="readiness-model-explain-computation">
        <summary class="readiness-model-explain-computation-summary">
            How this score is computed.
        </summary>
        <ol class="readiness-model-explain-pipeline">
            {#each model.steps as step, index}
                <li class="readiness-model-explain-step">
                    <div class="readiness-model-explain-step-header">
                        <span class="readiness-model-explain-step-label">{step.label}</span>
                        {#if step.value}
                            <span class="readiness-model-explain-step-value">{step.value}</span>
                        {/if}
                    </div>
                    {#if step.interval}
                        <p class="readiness-model-explain-step-interval">{step.interval}</p>
                    {/if}
                    <p class="readiness-model-explain-step-detail">{step.detail}</p>
                </li>
                {#if index < model.steps.length - 1}
                    <li class="readiness-model-explain-arrow" aria-hidden="true">↓</li>
                {/if}
            {/each}
        </ol>
    </details>
</section>

<style lang="scss">
    .readiness-model-explain {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
        padding-top: var(--gre-space-3);
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
    }

    .readiness-model-explain-summary {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
    }

    .readiness-model-explain-hero {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .readiness-model-explain-label,
    .readiness-model-explain-section-title {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .readiness-model-explain-score {
        font-size: var(--gre-font-display);
        font-weight: var(--gre-weight-hero);
        line-height: var(--gre-lh-display);
        letter-spacing: var(--gre-tracking-display);
        color: var(--fg);
    }

    .readiness-model-explain-metrics {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .readiness-model-explain-factors {
        margin: 0;
        padding-left: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-model-explain-evidence {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .readiness-model-explain-evidence-list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .readiness-model-explain-evidence-list li {
        display: flex;
        flex-direction: column;
        gap: 0.125rem;
    }

    .readiness-model-explain-evidence-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .readiness-model-explain-evidence-detail {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-model-explain-empty {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-model-explain-computation {
        border-radius: var(--gre-radius-md);
        background: color-mix(in srgb, var(--canvas) 55%, var(--gre-surface-bg-muted));
        padding: var(--gre-space-3) var(--gre-space-4);
    }

    .readiness-model-explain-computation-summary {
        cursor: pointer;
        font-size: var(--gre-font-body);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-body);
        color: var(--fg-link);
        list-style: none;
    }

    .readiness-model-explain-computation-summary::-webkit-details-marker {
        display: none;
    }

    .readiness-model-explain-pipeline {
        list-style: none;
        margin: var(--gre-space-3) 0 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .readiness-model-explain-step {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        padding: var(--gre-space-3);
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg);
        box-shadow: var(--gre-shadow-sm);
    }

    .readiness-model-explain-step-header {
        display: flex;
        flex-wrap: wrap;
        align-items: baseline;
        justify-content: space-between;
        gap: var(--gre-space-2);
    }

    .readiness-model-explain-step-label {
        font-size: var(--gre-font-h3);
        font-weight: var(--gre-weight-h3);
        line-height: var(--gre-lh-h3);
        color: var(--fg);
    }

    .readiness-model-explain-step-value {
        font-size: var(--gre-font-body);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .readiness-model-explain-step-interval {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-model-explain-step-detail {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-model-explain-arrow {
        align-self: center;
        font-size: var(--gre-font-h2);
        line-height: 1;
        color: var(--fg-subtle);
    }
</style>
