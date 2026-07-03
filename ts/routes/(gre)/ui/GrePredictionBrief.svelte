<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { AbstentionRequirement } from "@generated/anki/brainlift_pb";

    import type {
        PredictionAction,
        PredictionDetailRow,
    } from "../prediction-presentation";
    import type { PredictionExplainability } from "../prediction-explainability";
    import type { MetricChangePresentation } from "../metric-change-presentation";
    import { runGreNavAction } from "../gre-navigation";
    import GreAbstentionChecklist from "./GreAbstentionChecklist.svelte";
    import GreButton from "./GreButton.svelte";
    import GreConfidenceIndicator from "./GreConfidenceIndicator.svelte";
    import GreMetricRow from "./GreMetricRow.svelte";
    import GreMetricChangeInspect from "./GreMetricChangeInspect.svelte";
    import GrePredictionExplain from "./GrePredictionExplain.svelte";
    import GreText from "./GreText.svelte";

    export let title: string;
    export let score: string;
    export let scoreRange: string | null = null;
    export let unlocked = true;
    export let confidence: string;
    export let why: string;
    export let evidence: string;
    export let nextAction: PredictionAction;
    export let explainability: PredictionExplainability | null = null;
    export let detailRows: PredictionDetailRow[] = [];
    export let requirements: AbstentionRequirement[] = [];
    export let variant: "card" | "compact" | "inline" = "card";
    export let showScoreHeader = true;
    export let expandLabel = "Inspect evidence";
    export let confidenceAsText = false;
    export let metricChange: MetricChangePresentation | null = null;
    export let confidenceChange: MetricChangePresentation | null = null;

    $: explainCompact = variant === "compact";
</script>

<div
    class="gre-prediction-brief"
    class:gre-prediction-brief-compact={variant === "compact"}
    class:gre-prediction-brief-inline={variant === "inline"}
    aria-label="{title} prediction"
>
    {#if showScoreHeader && variant !== "inline"}
        <div class="gre-prediction-score-header">
            <GreText variant="hero" tag="div" className="gre-prediction-score">
                {score}
            </GreText>
            {#if scoreRange}
                <span class="gre-prediction-range">{scoreRange}</span>
            {/if}
            {#if !unlocked}
                <span class="gre-prediction-locked-label">Locked</span>
            {/if}
        </div>
    {/if}

    <GreMetricChangeInspect
        change={metricChange}
        variant={explainCompact ? "compact" : "default"}
    />

    {#if explainability}
        <GrePredictionExplain {explainability} compact={explainCompact} />
    {/if}

    <div class="gre-prediction-footer">
        <div class="gre-prediction-confidence-row">
            <span class="gre-ds-metric-label">Confidence</span>
            {#if confidenceAsText}
                <span class="gre-ds-metric-value">{confidence}</span>
            {:else}
                <GreConfidenceIndicator {confidence} />
            {/if}
        </div>
        <GreMetricChangeInspect change={confidenceChange} variant="compact" />
        <div class="gre-prediction-action-row">
            <span class="gre-ds-metric-label">Next</span>
            {#if nextAction.href || nextAction.bridge}
                <GreButton
                    variant="primary"
                    size="sm"
                    className="gre-prediction-action"
                    on:click={(event) => runGreNavAction(nextAction, event)}
                >
                    {nextAction.label}
                </GreButton>
            {:else}
                <GreButton
                    variant="secondary"
                    size="sm"
                    className="gre-prediction-action"
                >
                    {nextAction.label}
                </GreButton>
            {/if}
        </div>
    </div>

    {#if detailRows.length > 0 || requirements.some((req) => !req.met) || evidence || why}
        <details class="gre-prediction-details">
            <summary>{expandLabel}</summary>
            <div class="gre-prediction-details-body">
                {#if why}
                    <GreMetricRow label="Why" value={why} />
                {/if}
                {#if evidence}
                    <div class="gre-prediction-evidence-block">
                        <span class="gre-ds-metric-label">Evidence summary</span>
                        <p class="gre-prediction-evidence-text">{evidence}</p>
                    </div>
                {/if}
                {#each detailRows as row}
                    <GreMetricRow label={row.label} value={row.value} />
                {/each}
                <GreAbstentionChecklist {requirements} compact showProgress />
            </div>
        </details>
    {/if}
</div>

<style lang="scss">
    .gre-prediction-brief {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        min-width: 0;
    }

    .gre-prediction-brief-compact {
        gap: var(--gre-space-2);
    }

    .gre-prediction-brief-inline {
        gap: var(--gre-space-2);
    }

    .gre-prediction-score-header {
        display: flex;
        flex-wrap: wrap;
        align-items: baseline;
        gap: var(--gre-space-2);
    }

    .gre-prediction-brief-compact :global(.gre-prediction-score) {
        font-size: var(--gre-font-kpi);
        line-height: var(--gre-lh-h1);
    }

    .gre-prediction-range {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-prediction-locked-label {
        font-size: var(--gre-font-h3);
        font-weight: var(--gre-weight-h3);
        line-height: var(--gre-lh-h3);
        letter-spacing: 0.05em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .gre-prediction-footer {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        padding-top: var(--gre-space-1);
        border-top: 1px solid color-mix(in srgb, var(--border) 35%, transparent);
    }

    .gre-prediction-confidence-row,
    .gre-prediction-action-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--gre-space-3);
        padding: var(--gre-space-1) 0;
        min-width: 0;
    }

    .gre-prediction-confidence-row :global(.gre-confidence) {
        flex-shrink: 0;
    }

    .gre-prediction-action-row :global(.gre-prediction-action) {
        flex-shrink: 0;
    }

    .gre-prediction-details {
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
        padding-top: var(--gre-space-2);
    }

    .gre-prediction-details summary {
        cursor: pointer;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg-link);
        list-style: none;
    }

    .gre-prediction-details summary::-webkit-details-marker {
        display: none;
    }

    .gre-prediction-details-body {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        margin-top: var(--gre-space-2);
    }

    .gre-prediction-evidence-block {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        padding: var(--gre-space-1) 0;
    }

    .gre-prediction-evidence-text {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .gre-prediction-details-body :global(.gre-ds-metric-value) {
        text-align: right;
        max-width: 16rem;
    }
</style>
