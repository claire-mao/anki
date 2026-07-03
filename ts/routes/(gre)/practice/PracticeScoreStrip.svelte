<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { MemoryScore, PerformanceScore } from "@generated/anki/brainlift_pb";

    import { formatPercent } from "../score-format";
    import { memoryHero, performanceHero } from "../summary-metrics";

    export let memory: MemoryScore;
    export let performance: PerformanceScore;
</script>

<div class="gre-ds-stat-strip" aria-label="Memory and performance scores">
    <div class="gre-ds-stat-strip-item">
        <span class="gre-ds-stat-strip-label">Memory</span>
        <span class="gre-ds-stat-strip-value">{memoryHero(memory, formatPercent)}</span>
    </div>
    <div class="gre-ds-stat-strip-item">
        <span class="gre-ds-stat-strip-label">Performance</span>
        <span class="gre-ds-stat-strip-value">
            {performanceHero(performance, formatPercent)}
        </span>
    </div>
</div>
