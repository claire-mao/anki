<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { confidenceVisual } from "../indicator-utils";

    export let confidence: string;
    export let showLabel = true;

    $: visual = confidenceVisual(confidence);

    function activeSegmentCount(level: typeof visual): number {
        if (level === "high") {
            return 3;
        }
        if (level === "medium") {
            return 2;
        }
        if (level === "low" || level === "preliminary") {
            return 1;
        }
        return 0;
    }

    $: activeSegments = activeSegmentCount(visual);
</script>

<div class="gre-confidence" aria-label="Confidence: {confidence}">
    <div class="gre-confidence-segments" aria-hidden="true">
        {#each [0, 1, 2] as index}
            <span
                class="gre-confidence-segment"
                class:gre-confidence-segment-active={index < activeSegments &&
                    visual !== "preliminary"}
                class:gre-confidence-segment-preliminary={index < activeSegments &&
                    visual === "preliminary"}
            ></span>
        {/each}
    </div>
    {#if showLabel}
        <span class="gre-confidence-label">{confidence}</span>
    {/if}
</div>
