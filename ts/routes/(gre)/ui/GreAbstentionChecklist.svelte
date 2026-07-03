<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { AbstentionRequirement } from "@generated/anki/brainlift_pb";

    export let requirements: AbstentionRequirement[] = [];
    export let compact = false;
    export let showProgress = false;

    $: visible = showProgress
        ? requirements
        : requirements.filter((req) => !req.met || !compact);
</script>

{#if visible.length > 0}
    <ul
        class="gre-checklist gre-abstention-checklist"
        class:gre-checklist-compact={compact}
        aria-label="Unlock requirements"
    >
        {#each visible as req}
            <li class:gre-checklist-met={req.met}>
                <span class="gre-checklist-mark" aria-hidden="true">
                    {req.met ? "✓" : "○"}
                </span>
                <span class="sr-only">{req.met ? "Complete:" : "Incomplete:"}</span>
                <span class="gre-checklist-label">{req.label}</span>
                {#if !showProgress}
                    <span class="gre-checklist-detail">{req.status}</span>
                {/if}
            </li>
        {/each}
    </ul>
{/if}
