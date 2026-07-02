<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let points: number[] = [];
    export let width = 88;
    export let height = 28;
    export let label: string | null = "Trend";

    $: validPoints = points.filter((point) => Number.isFinite(point));
    $: path = buildPath(validPoints, width, height);
    $: area = buildArea(validPoints, width, height);

    function buildPath(values: number[], w: number, h: number): string {
        if (values.length < 2) {
            return "";
        }
        const min = Math.min(...values);
        const max = Math.max(...values);
        const range = max - min || 1;
        const step = w / (values.length - 1);
        return values
            .map((value, index) => {
                const x = index * step;
                const y = h - ((value - min) / range) * (h - 4) - 2;
                return `${index === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`;
            })
            .join(" ");
    }

    function buildArea(values: number[], w: number, h: number): string {
        if (values.length < 2) {
            return "";
        }
        const line = buildPath(values, w, h);
        return `${line} L${w} ${h} L0 ${h} Z`;
    }
</script>

{#if validPoints.length >= 2}
    <svg
        class="gre-sparkline"
        {width}
        {height}
        viewBox="0 0 {width} {height}"
        role={label ? "img" : undefined}
        aria-hidden={label ? undefined : true}
        aria-label={label ? `${label}: ${validPoints.at(-1)}%` : undefined}
    >
        {#if area}
            <path class="gre-sparkline-fill" d={area} />
        {/if}
        {#if path}
            <path class="gre-sparkline-line" d={path} />
        {/if}
    </svg>
{/if}
