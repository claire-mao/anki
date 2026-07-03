<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ReadinessPagePresentation } from "../readiness-page-presentation";
    import { runGreNavAction } from "../gre-navigation";
    import GreButton from "../ui/GreButton.svelte";
    import GreConfidenceIndicator from "../ui/GreConfidenceIndicator.svelte";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import GreText from "../ui/GreText.svelte";

    export let model: ReadinessPagePresentation;
</script>

<section class="readiness-estimate" aria-label="Readiness estimate">
    {#if !model.available}
        <div class="readiness-estimate-unavailable" role="status">
            <GreText variant="h2" tag="h2">{model.unavailableTitle}</GreText>
            <p class="readiness-estimate-unavailable-reason">
                <span class="readiness-estimate-label">Reason:</span>
                {model.unavailableReason}
            </p>
            {#if model.unavailableDetails.length > 0}
                <ul class="readiness-estimate-unavailable-details">
                    {#each model.unavailableDetails as detail}
                        <li>{detail}</li>
                    {/each}
                </ul>
            {/if}
        </div>
    {:else}
        <div class="readiness-estimate-hero">
            <span class="readiness-estimate-hero-label">Readiness score</span>
            <span class="readiness-estimate-hero-value">{model.readinessScore}</span>
        </div>
    {/if}

    <div class="readiness-estimate-metrics">
        <GreMetricRow label="Estimated GRE" value={model.estimatedGre} />
        {#if model.estimatedGreDetail}
            <p class="readiness-estimate-detail">{model.estimatedGreDetail}</p>
        {/if}
        <GreMetricRow label="Confidence interval" value={model.confidenceInterval} />
        <div class="readiness-estimate-confidence-row">
            <span class="gre-ds-metric-label">Confidence level</span>
            <GreConfidenceIndicator confidence={model.confidenceLevel} />
        </div>
        <GreMetricRow label="Coverage" value={model.coverage} />
        <GreMetricRow label="Memory" value={model.memory} />
        <GreMetricRow label="Performance" value={model.performance} />
        <GreMetricRow label="Calibration quality" value={model.calibrationQuality} />
        <GreMetricRow label="Last updated" value={model.lastUpdated} />
    </div>

    <div class="readiness-estimate-evidence">
        <section
            class="readiness-estimate-evidence-block"
            aria-labelledby="readiness-evidence-used-heading"
        >
            <h3
                class="readiness-estimate-section-title"
                id="readiness-evidence-used-heading"
            >
                Evidence used
            </h3>
            {#if model.evidenceUsed.length > 0}
                <ul class="readiness-estimate-evidence-list">
                    {#each model.evidenceUsed as line}
                        <li>
                            <span class="readiness-estimate-evidence-label">
                                {line.label}
                            </span>
                            <span class="readiness-estimate-evidence-detail">
                                {line.detail}
                            </span>
                        </li>
                    {/each}
                </ul>
            {:else}
                <p class="readiness-estimate-empty">No evidence recorded yet.</p>
            {/if}
        </section>

        <section
            class="readiness-estimate-evidence-block"
            aria-labelledby="readiness-evidence-missing-heading"
        >
            <h3
                class="readiness-estimate-section-title"
                id="readiness-evidence-missing-heading"
            >
                Evidence missing
            </h3>
            {#if model.evidenceMissing.length > 0}
                <ul
                    class="readiness-estimate-evidence-list readiness-estimate-evidence-list-missing"
                >
                    {#each model.evidenceMissing as line}
                        <li>
                            <span class="readiness-estimate-evidence-label">
                                {line.label}
                            </span>
                            <span class="readiness-estimate-evidence-detail">
                                {line.detail}
                            </span>
                        </li>
                    {/each}
                </ul>
            {:else}
                <p class="readiness-estimate-empty">
                    All required evidence is present.
                </p>
            {/if}
        </section>
    </div>

    <div class="readiness-estimate-action">
        <span class="readiness-estimate-label">Next best action</span>
        {#if model.nextAction.href || model.nextAction.bridge}
            <GreButton
                variant="primary"
                size="md"
                on:click={(event) => runGreNavAction(model.nextAction, event)}
            >
                {model.nextAction.label}
            </GreButton>
        {:else}
            <GreButton variant="secondary" size="md" disabled>
                {model.nextAction.label}
            </GreButton>
        {/if}
    </div>
</section>

<style lang="scss">
    .readiness-estimate {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-5);
        padding: var(--gre-card-padding);
        border-radius: var(--gre-radius-lg);
        background: var(--gre-surface-bg);
        box-shadow: var(--gre-shadow-md);
    }

    .readiness-estimate-unavailable {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        padding: var(--gre-space-4);
        border-radius: var(--gre-radius-md);
        border: 1px solid color-mix(in srgb, var(--state-review) 28%, var(--border));
        background: color-mix(in srgb, var(--state-review) 8%, var(--canvas));
    }

    .readiness-estimate-unavailable :global(.gre-text-h2) {
        margin: 0;
    }

    .readiness-estimate-unavailable-reason {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .readiness-estimate-unavailable-details {
        margin: 0;
        padding-left: 1.25rem;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-estimate-hero {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .readiness-estimate-hero-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.05em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .readiness-estimate-hero-value {
        font-size: var(--gre-font-display);
        font-weight: var(--gre-weight-hero);
        line-height: var(--gre-lh-display);
        letter-spacing: var(--gre-tracking-display);
        color: var(--fg);
    }

    .readiness-estimate-metrics {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        padding-top: var(--gre-space-2);
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
    }

    .readiness-estimate-detail {
        margin: calc(-1 * var(--gre-space-1)) 0 0;
        padding-left: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-estimate-confidence-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--gre-space-3);
        padding: var(--gre-space-1) 0;
    }

    .readiness-estimate-evidence {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: var(--gre-space-4);
    }

    .readiness-estimate-evidence-block {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        min-width: 0;
    }

    .readiness-estimate-section-title {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .readiness-estimate-evidence-list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .readiness-estimate-evidence-list li {
        display: flex;
        flex-direction: column;
        gap: 0.125rem;
    }

    .readiness-estimate-evidence-list-missing li {
        padding-left: 0.75rem;
        border-left: 2px solid
            color-mix(in srgb, var(--state-review) 45%, var(--border));
    }

    .readiness-estimate-evidence-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .readiness-estimate-evidence-detail {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-estimate-empty {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .readiness-estimate-action {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        justify-content: space-between;
        gap: var(--gre-space-3);
        padding-top: var(--gre-space-3);
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
    }

    .readiness-estimate-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    @media (max-width: 767px) {
        .readiness-estimate-evidence {
            grid-template-columns: 1fr;
        }
    }
</style>
