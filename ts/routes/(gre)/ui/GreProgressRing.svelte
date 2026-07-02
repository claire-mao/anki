<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { clampPercent } from "../indicator-utils";

    export let value: number | null = null;
    export let size: "sm" | "md" | "lg" = "md";
    export let label: string | null = null;
    export let color = "var(--fg-link)";

    const sizes = { sm: 36, md: 52, lg: 72 } as const;
    const stroke = { sm: 4, md: 5, lg: 6 } as const;

    $: dimension = sizes[size];
    $: strokeWidth = stroke[size];
    $: radius = (dimension - strokeWidth) / 2;
    $: circumference = 2 * Math.PI * radius;
    $: percent = value === null ? 0 : clampPercent(value);
    $: offset = circumference - (percent / 100) * circumference;
    $: displayValue = value === null ? "—" : `${percent}%`;
</script>

<div
    class="gre-progress-ring gre-progress-ring-{size}"
    role="img"
    aria-label={label ? `${label}: ${displayValue}` : displayValue}
>
    <svg width={dimension} height={dimension} viewBox="0 0 {dimension} {dimension}">
        <circle
            class="gre-progress-ring-bg"
            cx={dimension / 2}
            cy={dimension / 2}
            r={radius}
            stroke-width={strokeWidth}
        />
        <circle
            class="gre-progress-ring-fill"
            cx={dimension / 2}
            cy={dimension / 2}
            r={radius}
            stroke-width={strokeWidth}
            stroke={color}
            stroke-dasharray={circumference}
            stroke-dashoffset={offset}
        />
    </svg>
    <span class="gre-progress-ring-value">{displayValue}</span>
</div>
