<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        PerformanceAttempt,
        PerformanceScore,
    } from "@generated/anki/brainlift_pb";

    import { emptyStateContent } from "../empty-states";
    import { formatPercent } from "../score-format";
    import { rollingAccuracySeries } from "../indicator-utils";
    import { performanceHero, unmetRequirements } from "../summary-metrics";
    import GreAbstentionChecklist from "../ui/GreAbstentionChecklist.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreProgressBar from "../ui/GreProgressBar.svelte";
    import GreSparkline from "../ui/GreSparkline.svelte";
    import GreText from "../ui/GreText.svelte";

    export let performance: PerformanceScore;
    export let recentAttempts: PerformanceAttempt[] = [];

    $: sparkline = rollingAccuracySeries(recentAttempts);
    $: unlocked = performance.sufficientData && performance.value !== undefined;
</script>

<div class="gre-summary">
    {#if unlocked}
        <GreText variant="hero" tag="div" className="gre-summary-hero">
            {performanceHero(performance, formatPercent)}
        </GreText>
    {/if}

    <div class="gre-summary-indicators">
        {#if unlocked}
            <GreProgressBar
                label="Accuracy"
                value={performance.value ?? null}
                formatValue={(value) => formatPercent(value)}
            />
        {/if}
        {#if sparkline.length >= 2}
            <GreSparkline points={sparkline} label="Recent accuracy trend" />
        {/if}
        <p class="gre-summary-caption">{performance.attemptCount} attempts</p>
    </div>

    {#if unlocked}
        <GreAbstentionChecklist
            requirements={unmetRequirements(performance.abstentionRequirements)}
            compact
        />
    {:else}
        <GreEmptyState
            content={emptyStateContent("performance")}
            requirements={performance.abstentionRequirements}
        />
    {/if}
</div>

<style lang="scss">
    .gre-summary-indicators {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
    }
</style>
