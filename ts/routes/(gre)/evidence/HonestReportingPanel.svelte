<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreIcon from "../GreIcon.svelte";
    import {
        HONEST_REPORTING_ALL_CLEAR,
        HONEST_REPORTING_LEAD,
        HONEST_REPORTING_TITLE,
        type HonestReportingPresentation,
    } from "./honest-reporting-presentation";

    export let model: HonestReportingPresentation;
</script>

<section class="evidence-section evidence-honest-reporting" aria-labelledby="evidence-honest-reporting-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-honest-reporting-heading">
            {HONEST_REPORTING_TITLE}
        </h2>
        <p class="evidence-section-lead">{HONEST_REPORTING_LEAD}</p>
    </header>

    {#if model.allClear}
        <div class="evidence-honest-clear" role="status">
            <GreIcon name="check" size="sm" />
            <p>{HONEST_REPORTING_ALL_CLEAR}</p>
        </div>
    {:else}
        <div class="evidence-honest-cards">
            {#each model.limitations as limitation (limitation.id)}
                <article class="evidence-honest-card" aria-labelledby={`limitation-${limitation.id}`}>
                    <h3 class="evidence-honest-card-title" id={`limitation-${limitation.id}`}>
                        <GreIcon name="alert" size="sm" />
                        {limitation.title}
                    </h3>
                    <dl class="evidence-honest-details">
                        <div class="evidence-honest-detail">
                            <dt>Why it happened</dt>
                            <dd>{limitation.why}</dd>
                        </div>
                        <div class="evidence-honest-detail">
                            <dt>Missing evidence</dt>
                            <dd>{limitation.missingEvidence}</dd>
                        </div>
                        <div class="evidence-honest-detail">
                            <dt>How to improve</dt>
                            <dd>{limitation.howToImprove}</dd>
                        </div>
                    </dl>
                </article>
            {/each}
        </div>
    {/if}
</section>

<style lang="scss">
    .evidence-honest-reporting {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
    }

    .evidence-honest-clear {
        display: flex;
        align-items: flex-start;
        gap: var(--gre-space-2);
        padding: var(--gre-space-3) var(--gre-space-4);
        border-radius: var(--gre-radius-md);
        border: 1px solid color-mix(in srgb, var(--gre-success) 28%, var(--border));
        background: color-mix(in srgb, var(--gre-success) 8%, var(--canvas));
        color: var(--fg-subtle);
    }

    .evidence-honest-clear p {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
    }

    .evidence-honest-clear :global(.gre-icon) {
        flex-shrink: 0;
        margin-top: 0.15rem;
        color: var(--gre-success);
    }

    .evidence-honest-cards {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
    }

    .evidence-honest-card {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        padding: var(--gre-space-3) var(--gre-space-4);
        border-radius: var(--gre-radius-md);
        border: 1px solid color-mix(in srgb, var(--state-review) 28%, var(--border));
        background: color-mix(in srgb, var(--state-review) 8%, var(--canvas));
    }

    .evidence-honest-card-title {
        display: flex;
        align-items: center;
        gap: var(--gre-space-2);
        margin: 0;
        font-size: var(--gre-font-h3);
        font-weight: var(--gre-weight-h3);
        line-height: var(--gre-lh-h3);
        color: var(--fg);
    }

    .evidence-honest-card-title :global(.gre-icon) {
        flex-shrink: 0;
        color: var(--state-review);
    }

    .evidence-honest-details {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        margin: 0;
    }

    .evidence-honest-detail {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .evidence-honest-detail dt {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .evidence-honest-detail dd {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }
</style>
