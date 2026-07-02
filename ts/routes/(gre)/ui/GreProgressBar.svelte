<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { clampPercent } from "../indicator-utils";

    export let label: string | null = null;
    export let value: number | null = null;
    export let max = 100;
    export let showValue = true;
    export let compact = false;
    export let formatValue: ((value: number) => string) | null = null;

    $: percent =
        value === null || max <= 0 ? 0 : clampPercent((value / max) * 100);

    function formatDisplayValue(): string {
        if (value === null) {
            return "—";
        }
        if (formatValue) {
            return formatValue(value);
        }
        if (max === 100) {
            return `${percent}%`;
        }
        return `${value} / ${max}`;
    }

    $: displayValue = formatDisplayValue();
</script>

<div
    class="gre-progress-bar"
    class:gre-progress-bar-compact={compact}
    aria-label={label ? `${label}: ${displayValue}` : displayValue}
>
    <div class="gre-progress-bar-header">
        {#if label}
            <span class="gre-progress-bar-label">{label}</span>
        {/if}
        {#if showValue}
            <span class="gre-progress-bar-value">{displayValue}</span>
        {/if}
    </div>
    <div class="gre-progress-bar-track" aria-hidden="true">
        <div class="gre-progress-bar-fill" style:width="{percent}%"></div>
    </div>
</div>
