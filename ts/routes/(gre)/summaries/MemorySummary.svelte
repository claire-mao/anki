<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { MemoryScore } from "@generated/anki/brainlift_pb";

    import { emptyStateContent } from "../empty-states";
    import { formatPercent } from "../score-format";
    import { ratioToPercent } from "../indicator-utils";
    import { fsrsStatus, memoryHero, unmetRequirements } from "../summary-metrics";
    import GreAbstentionChecklist from "../ui/GreAbstentionChecklist.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreProgressBar from "../ui/GreProgressBar.svelte";
    import GreText from "../ui/GreText.svelte";

    export let memory: MemoryScore;

    $: fsrsMet = memory.abstentionRequirements.find((req) => req.id === "fsrs_enabled")?.met ?? false;
</script>

<div class="gre-summary">
    {#if memory.sufficientData && memory.value !== undefined}
        <GreText variant="hero" tag="div" className="gre-summary-hero">
            {memoryHero(memory, formatPercent)}
        </GreText>
    {/if}

    <div class="gre-summary-indicators">
        <GreProgressBar
            label="Coverage"
            value={ratioToPercent(memory.coverageRatio)}
            formatValue={() => formatPercent(memory.coverageRatio * 100)}
        />
        <GreProgressBar
            label="FSRS"
            value={fsrsMet ? 100 : 0}
            formatValue={() => fsrsStatus(memory.abstentionRequirements)}
        />
        <p class="gre-summary-caption">{memory.studiedCards} studied cards</p>
    </div>

    {#if memory.sufficientData && memory.value !== undefined}
        <GreAbstentionChecklist
            requirements={unmetRequirements(memory.abstentionRequirements)}
            compact
        />
    {:else}
        <GreEmptyState
            content={emptyStateContent("memory")}
            requirements={memory.abstentionRequirements}
        />
    {/if}
</div>

<style lang="scss">
    .gre-summary-indicators {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
    }
</style>
