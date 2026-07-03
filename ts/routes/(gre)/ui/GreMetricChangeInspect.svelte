<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { MetricChangePresentation } from "../metric-change-presentation";

    export let change: MetricChangePresentation | null | undefined = null;
    export let variant: "default" | "compact" = "default";
</script>

{#if change}
    <details
        class="gre-metric-change"
        class:gre-metric-change-compact={variant === "compact"}
        aria-label="{change.headline} explanation"
    >
        <summary class="gre-metric-change-summary">
            <span class="gre-metric-change-headline">{change.headline}</span>
            {#if change.deltaLabel}
                <span
                    class="gre-metric-change-delta"
                    class:gre-metric-change-delta-up={change.direction === "increased"}
                    class:gre-metric-change-delta-down={change.direction ===
                        "decreased"}
                >
                    {change.deltaLabel}
                </span>
            {/if}
        </summary>
        {#if change.evidence.length > 0}
            <div class="gre-metric-change-body">
                <p class="gre-metric-change-because">because</p>
                <ul class="gre-metric-change-evidence">
                    {#each change.evidence as item (item.id)}
                        <li
                            class:gre-metric-change-positive={item.positive}
                            class:gre-metric-change-negative={!item.positive}
                        >
                            <span class="gre-metric-change-mark" aria-hidden="true">
                                {item.positive ? "✓" : "•"}
                            </span>
                            <span>{item.label}</span>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}
    </details>
{/if}

<style lang="scss">
    .gre-metric-change {
        border-top: 1px solid color-mix(in srgb, var(--border) 35%, transparent);
        padding-top: var(--gre-space-2);
    }

    .gre-metric-change-compact {
        padding-top: var(--gre-space-1);
    }

    .gre-metric-change-summary {
        display: flex;
        flex-wrap: wrap;
        align-items: baseline;
        gap: var(--gre-space-2);
        cursor: pointer;
        list-style: none;
    }

    .gre-metric-change-summary::-webkit-details-marker {
        display: none;
    }

    .gre-metric-change-headline {
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        color: var(--fg-link);
    }

    .gre-metric-change-delta {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-metric-change-delta-up {
        color: var(--state-new);
    }

    .gre-metric-change-delta-down {
        color: var(--state-review);
    }

    .gre-metric-change-body {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        margin-top: var(--gre-space-2);
    }

    .gre-metric-change-because {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-metric-change-evidence {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-metric-change-evidence li {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: var(--gre-space-2);
        align-items: start;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg);
    }

    .gre-metric-change-mark {
        font-weight: var(--gre-weight-h1);
    }

    .gre-metric-change-positive .gre-metric-change-mark {
        color: var(--state-new);
    }

    .gre-metric-change-negative .gre-metric-change-mark {
        color: var(--state-review);
    }
</style>
