<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { defaultGraphBounds, type GraphBounds } from "../../graphs/graph-helpers";

    export let title: string | null = null;
    export let ariaLabel: string | null = null;
    export let subtitle: string | null = null;
    export let renderChart: ((svg: SVGElement, bounds: GraphBounds) => void) | null =
        null;
    export let bounds: GraphBounds = defaultGraphBounds();
    export let wide = false;
    export let tall = false;
    export let extraTall = false;
    export let scrollable = false;
    /** When true, chart width tracks the card body (viewBox width updates on resize). */
    export let fluid = false;
    let svg: SVGElement | null = null;
    let body: HTMLDivElement | null = null;
    let measuredWidth = bounds.width;

    $: chartId = (title ?? ariaLabel ?? "chart")
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-");
    $: accessibleLabel = title ?? ariaLabel ?? "Chart";
    $: effectiveBounds = fluid
        ? { ...bounds, width: measuredWidth }
        : bounds;

    onMount(() => {
        if (!fluid || !body) {
            return;
        }
        const updateWidth = (): void => {
            measuredWidth = Math.max(body!.clientWidth, 320);
        };
        updateWidth();
        const observer = new ResizeObserver(updateWidth);
        observer.observe(body);
        return () => observer.disconnect();
    });

    $: if (svg && renderChart) {
        renderChart(svg, effectiveBounds);
    }
</script>

<article
    class="gre-ds-chart-card progress-chart-card"
    class:progress-chart-wide={wide}
    class:progress-chart-tall={tall}
    class:progress-chart-extra-tall={extraTall}
    class:progress-chart-scrollable={scrollable}
>
    {#if title}
        <header class="progress-chart-header">
            <h3 class="progress-chart-title" id="chart-title-{chartId}">{title}</h3>
            {#if subtitle}
                <p class="progress-chart-subtitle" id="chart-subtitle-{chartId}">
                    {subtitle}
                </p>
            {/if}
        </header>
    {/if}
    <div
        bind:this={body}
        class="progress-chart-body"
        class:progress-chart-body-fluid={fluid}
        role="img"
        aria-label={title ? undefined : accessibleLabel}
        aria-labelledby={title ? `chart-title-${chartId}` : undefined}
        aria-describedby={title && subtitle ? `chart-subtitle-${chartId}` : undefined}
    >
        <svg
            bind:this={svg}
            class="progress-chart"
            class:progress-chart-fluid={fluid}
            viewBox="0 0 {effectiveBounds.width} {effectiveBounds.height}"
        >
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

    .progress-chart-body-fluid {
        width: 100%;
    }

    .progress-chart-wide {
        width: 100%;
    }

    .progress-chart-fluid {
        display: block;
        width: 100%;
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

        :global(.chart-axis-label) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-caption);
        }

        :global(.tick text) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-tick);
        }

        :global(.value-unstudied) {
            fill: var(--fg-subtle);
            font-size: var(--gre-chart-font-caption);
        }
    }

    .progress-chart-tall .progress-chart {
        max-height: 18rem;
    }

    .progress-chart-extra-tall .progress-chart {
        max-height: 30rem;
        min-height: 22rem;
    }

    .progress-chart-scrollable .progress-chart-body {
        max-height: 28rem;
        overflow-y: auto;
        align-items: flex-start;
    }

    .progress-chart-scrollable .progress-chart {
        max-height: none;
        min-height: 0;
    }
</style>
