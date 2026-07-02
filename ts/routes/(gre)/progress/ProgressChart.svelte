<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { defaultGraphBounds, type GraphBounds } from "../../graphs/graph-helpers";

    export let title: string;
    export let subtitle: string | null = null;
    export let renderChart: ((svg: SVGElement, bounds: GraphBounds) => void) | null = null;
    export let wide = false;
    export let tall = false;

    const bounds = defaultGraphBounds();
    let svg: SVGElement | null = null;

    $: chartId = title.toLowerCase().replace(/[^a-z0-9]+/g, "-");

    $: if (svg && renderChart) {
        renderChart(svg, bounds);
    }
</script>

<article
    class="gre-ds-chart-card progress-chart-card"
    class:progress-chart-wide={wide}
    class:progress-chart-tall={tall}
>
    <header class="progress-chart-header">
        <h3 class="progress-chart-title" id="chart-title-{chartId}">{title}</h3>
        {#if subtitle}
            <p class="progress-chart-subtitle" id="chart-subtitle-{chartId}">{subtitle}</p>
        {/if}
    </header>
    <div
        class="progress-chart-body"
        role="img"
        aria-labelledby="chart-title-{chartId}"
        aria-describedby={subtitle ? `chart-subtitle-${chartId}` : undefined}
    >
        <svg bind:this={svg} class="progress-chart" viewBox="0 0 {bounds.width} {bounds.height}">
            <g class="chart-root" />
        </svg>
    </div>
</article>

<style lang="scss">
    .progress-chart-header {
        margin-bottom: var(--gre-space-2);
    }

    .progress-chart-title {
        margin: 0;
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h2);
        line-height: var(--gre-lh-h2);
        letter-spacing: -0.01em;
    }

    .progress-chart-subtitle {
        margin: var(--gre-space-1) 0 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .progress-chart-body {
        display: flex;
        flex: 1;
        align-items: center;
        min-height: 0;
    }

    .progress-chart {
        width: 100%;
        height: auto;
        max-height: 12.5rem;

        :global(.chart-empty-label) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-empty);
        }

        :global(.chart-empty-title) {
            fill: var(--fg);
            font-size: var(--gre-chart-font-title);
            font-weight: var(--gre-weight-h1);
        }

        :global(.chart-value-label) {
            fill: var(--fg);
            font-size: var(--gre-chart-font-label);
            font-weight: var(--gre-weight-h1);
        }

        :global(.chart-caption) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-caption);
        }

        :global(.tick text) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-tick);
        }
    }

    .progress-chart-tall .progress-chart {
        max-height: 18rem;
    }
</style>
