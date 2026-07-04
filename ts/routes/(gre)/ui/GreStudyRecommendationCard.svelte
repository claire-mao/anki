<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { StudyRecommendationPresentation } from "../recommendation-presentation";
    import { runGreNavAction } from "../gre-navigation";

    import GreButton from "./GreButton.svelte";

    export let recommendation: StudyRecommendationPresentation;
    export let compact = false;

    function onAction(): void {
        runGreNavAction(recommendation.action);
    }
</script>

<article
    class="gre-study-recommendation"
    class:gre-study-recommendation-compact={compact}
>
    <h3 class="gre-study-recommendation-title">
        {#if recommendation.topicId && recommendation.action.href}
            <a href={recommendation.action.href}>{recommendation.title}</a>
        {:else}
            {recommendation.title}
        {/if}
    </h3>

    <dl class="gre-study-recommendation-rows">
        <div class="gre-study-recommendation-row">
            <dt>Reason</dt>
            <dd>{recommendation.reason}</dd>
        </div>
        <div class="gre-study-recommendation-row">
            <dt>Expected impact</dt>
            <dd>{recommendation.expectedImpact}</dd>
        </div>
    </dl>

    {#if recommendation.progress}
        <div class="gre-study-recommendation-progress">
            <span class="gre-study-recommendation-progress-label">
                {recommendation.progress.current}/{recommendation.progress.target}
                {recommendation.progress.unit}
            </span>
            <div
                class="gre-ds-progress-track gre-study-recommendation-progress-track"
                role="progressbar"
                aria-valuemin="0"
                aria-valuemax={recommendation.progress.target}
                aria-valuenow={recommendation.progress.current}
                aria-label="{recommendation.title} progress"
            >
                <div
                    class="gre-ds-progress-fill gre-study-recommendation-progress-fill"
                    style:width="{recommendation.progress.percent}%"
                ></div>
            </div>
        </div>
    {/if}

    {#if recommendation.action.bridge}
        <GreButton
            variant="primary"
            size="sm"
            className="gre-study-recommendation-action"
            on:click={onAction}
        >
            {recommendation.action.label}
        </GreButton>
    {:else if recommendation.action.href}
        <GreButton
            variant="primary"
            size="sm"
            className="gre-study-recommendation-action"
            href={recommendation.action.href}
            on:click={(event) => runGreNavAction(recommendation.action, event)}
        >
            {recommendation.action.label}
        </GreButton>
    {/if}
</article>

<style lang="scss">
    .gre-study-recommendation {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        min-width: 0;
        padding: var(--gre-card-padding-compact);
        border: none;
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg-muted);
        box-shadow: var(--gre-shadow-sm);
    }

    .gre-study-recommendation-compact {
        gap: var(--gre-space-2);
    }

    .gre-study-recommendation-title {
        margin: 0;
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h1);
        line-height: var(--gre-lh-h2);
        letter-spacing: -0.01em;
    }

    .gre-study-recommendation-title a {
        color: var(--fg);
        text-decoration: none;
    }

    .gre-study-recommendation-title a:hover {
        color: var(--fg-link);
        text-decoration: underline;
    }

    .gre-study-recommendation-rows {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        margin: 0;
    }

    .gre-study-recommendation-row {
        display: grid;
        grid-template-columns: minmax(5.5rem, auto) minmax(0, 1fr);
        gap: var(--gre-space-2) var(--gre-space-3);
        align-items: baseline;
    }

    .gre-study-recommendation-row dt {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-study-recommendation-row dd {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .gre-study-recommendation-progress {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .gre-study-recommendation-progress-label {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
        font-variant-numeric: tabular-nums;
    }

    .gre-study-recommendation :global(.gre-study-recommendation-action) {
        align-self: flex-start;
    }
</style>
