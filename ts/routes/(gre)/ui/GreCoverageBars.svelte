<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { formatPercent } from "../score-format";
    import { ratioToPercent } from "../indicator-utils";
    import GreProgressBar from "./GreProgressBar.svelte";

    export let weightedRatio: number;
    export let unweightedRatio: number;
    export let coveredLeafCount: number;
    export let catalogLeafCount: number;
</script>

<div class="gre-coverage-bars">
    <GreProgressBar
        label="Weighted coverage"
        value={ratioToPercent(weightedRatio)}
        formatValue={() => formatPercent(weightedRatio * 100)}
    />
    <GreProgressBar
        label="Unweighted coverage"
        value={ratioToPercent(unweightedRatio)}
        formatValue={() => formatPercent(unweightedRatio * 100)}
    />
    <GreProgressBar
        label="Catalog leaves covered"
        value={coveredLeafCount}
        max={Math.max(catalogLeafCount, 1)}
        formatValue={(value) => `${value} / ${catalogLeafCount}`}
    />
</div>

<style lang="scss">
    .gre-coverage-bars {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
    }
</style>
