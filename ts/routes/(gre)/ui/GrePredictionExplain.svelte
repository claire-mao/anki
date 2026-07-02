<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { PredictionExplainability } from "../prediction-explainability";

    export let explainability: PredictionExplainability;
    export let compact = false;
</script>

<div class="gre-prediction-explain" class:gre-prediction-explain-compact={compact}>
    <section class="gre-prediction-explain-section" aria-label="Evidence used for this prediction">
        <h4 class="gre-prediction-explain-heading">Based on</h4>
        <ul class="gre-prediction-based-on">
            {#each explainability.basedOn as pillar (pillar.id)}
                <li class:gre-prediction-pillar-met={pillar.met}>
                    <span class="gre-prediction-pillar-mark" aria-hidden="true">
                        {pillar.met ? "✓" : "○"}
                    </span>
                    <span class="sr-only">{pillar.met ? "Met:" : "Not met:"}</span>
                    <span class="gre-prediction-pillar-label">{pillar.label}</span>
                </li>
            {/each}
        </ul>
    </section>

    {#if explainability.needsImprovement.length > 0}
        <section class="gre-prediction-explain-section" aria-label="Areas to improve this prediction">
            <h4 class="gre-prediction-explain-heading">Needs improvement</h4>
            <ul class="gre-prediction-improvements">
                {#each explainability.needsImprovement as item (item.id)}
                    <li>
                        {#if item.href}
                            <a class="gre-prediction-improvement-link" href={item.href}>{item.label}</a>
                        {:else}
                            <span class="gre-prediction-improvement-label">{item.label}</span>
                        {/if}
                        {#if item.detail && !compact}
                            <span class="gre-prediction-improvement-detail">{item.detail}</span>
                        {/if}
                    </li>
                {/each}
            </ul>
        </section>
    {/if}
</div>

<style lang="scss">
    .gre-prediction-explain {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        min-width: 0;
    }

    .gre-prediction-explain-compact {
        gap: var(--gre-space-2);
    }

    .gre-prediction-explain-section {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        min-width: 0;
    }

    .gre-prediction-explain-heading {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .gre-prediction-based-on,
    .gre-prediction-improvements {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-prediction-based-on li {
        display: flex;
        align-items: baseline;
        gap: var(--gre-space-2);
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-prediction-pillar-met {
        color: var(--fg);
    }

    .gre-prediction-pillar-mark {
        flex-shrink: 0;
        width: 1rem;
        font-weight: var(--gre-weight-h1);
        text-align: center;
    }

    .gre-prediction-pillar-met .gre-prediction-pillar-mark {
        color: var(--state-new);
    }

    .gre-prediction-pillar-label {
        min-width: 0;
    }

    .gre-prediction-improvements li {
        display: flex;
        flex-direction: column;
        gap: 0.125rem;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
    }

    .gre-prediction-improvement-link {
        color: var(--fg-link);
        font-weight: var(--gre-weight-label);
        text-decoration: none;
    }

    .gre-prediction-improvement-link:hover {
        text-decoration: underline;
    }

    .gre-prediction-improvement-label {
        color: var(--fg);
        font-weight: var(--gre-weight-label);
    }

    .gre-prediction-improvement-label::before {
        content: "• ";
        color: var(--fg-subtle);
    }

    .gre-prediction-improvement-link::before {
        content: "• ";
        color: var(--fg-subtle);
        font-weight: var(--gre-weight-body);
    }

    .gre-prediction-improvement-detail {
        padding-left: calc(1rem + var(--gre-space-2));
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
</style>
