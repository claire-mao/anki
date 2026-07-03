<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { MetricChangePresentation } from "../metric-change-presentation";
    import GreConfidenceIndicator from "../ui/GreConfidenceIndicator.svelte";
    import GreMetricChangeInspect from "../ui/GreMetricChangeInspect.svelte";
    import GreProgressBar from "../ui/GreProgressBar.svelte";
    import GreProgressRing from "../ui/GreProgressRing.svelte";
    import GreSparkline from "../ui/GreSparkline.svelte";

    export let label: string;
    export let value: string;
    export let detail: string | null = null;
    export let ringValue: number | null = null;
    export let barValue: number | null = null;
    export let barMax = 100;
    export let sparklinePoints: number[] = [];
    export let confidence: string | null = null;
    export let ringColor = "var(--fg-link)";
    export let metricChange: MetricChangePresentation | null = null;
</script>

<article class="progress-kpi-card">
    <div class="progress-kpi-visual">
        {#if ringValue !== null}
            <GreProgressRing
                value={ringValue}
                size="sm"
                label={null}
                color={ringColor}
            />
        {:else if barValue !== null}
            <GreProgressBar
                label={null}
                value={barValue}
                max={barMax}
                compact
                showValue={false}
            />
        {:else if sparklinePoints.length >= 2}
            <GreSparkline
                points={sparklinePoints}
                label={null}
                width={72}
                height={24}
            />
        {:else if confidence}
            <GreConfidenceIndicator {confidence} showLabel={false} />
        {/if}
    </div>
    <span class="progress-kpi-label">{label}</span>
    <span class="progress-kpi-value">{value}</span>
    {#if detail}
        <span class="progress-kpi-detail">{detail}</span>
    {/if}
    {#if confidence && (ringValue !== null || barValue !== null)}
        <GreConfidenceIndicator {confidence} />
    {/if}
    <GreMetricChangeInspect change={metricChange} variant="compact" />
</article>
