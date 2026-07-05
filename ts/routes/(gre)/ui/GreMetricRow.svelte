<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { GreMetricStructuredValue } from "./metric-value";

    export let label: string;
    export let value: string | undefined = undefined;
    export let structuredValue: GreMetricStructuredValue | undefined = undefined;
    export let hint: string | undefined = undefined;

    $: detailLayout = structuredValue?.detailLayout ?? "stack";
</script>

<div
    class="gre-ds-metric-row"
    class:gre-ds-metric-row-has-hint={hint}
    class:gre-ds-metric-row-has-stack={structuredValue}
>
    <div class="gre-ds-metric-main">
        <span class="gre-ds-metric-label">{label}</span>
        {#if hint}
            <p class="gre-ds-metric-hint">{hint}</p>
        {/if}
    </div>
    <div class="gre-ds-metric-value">
        {#if $$slots.default}
            <slot />
        {:else if structuredValue}
            <div class="gre-ds-metric-value-stack">
                <span class="gre-ds-metric-value-headline">
                    {structuredValue.headline}
                </span>
                {#if structuredValue.details && structuredValue.details.length > 0}
                    {#if detailLayout === "chips"}
                        <ul
                            class="gre-ds-metric-value-chips"
                            aria-label="{label} breakdown"
                        >
                            {#each structuredValue.details as detail}
                                <li class="gre-ds-metric-value-chip">{detail}</li>
                            {/each}
                        </ul>
                    {:else}
                        <ul
                            class="gre-ds-metric-value-details"
                            aria-label="{label} details"
                        >
                            {#each structuredValue.details as detail}
                                <li class="gre-ds-metric-value-detail">{detail}</li>
                            {/each}
                        </ul>
                    {/if}
                {/if}
            </div>
        {:else if value !== undefined}
            <span class="gre-ds-metric-value-text">{value}</span>
        {/if}
    </div>
</div>
