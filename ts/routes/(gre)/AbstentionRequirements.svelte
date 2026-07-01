<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { AbstentionRequirement } from "@generated/anki/brainlift_pb";

    export let requirements: AbstentionRequirement[] = [];
    export let heading = "Missing requirements";
</script>

{#if requirements.some((req) => !req.met)}
    <div class="abstention-panel">
        <h3>{heading}</h3>
        <ul class="abstention-list">
            {#each requirements.filter((req) => !req.met) as req}
                <li>
                    <strong>{req.label}</strong>
                    <span class="muted">{req.status}</span>
                    <p class="abstention-next">{req.nextStep}</p>
                </li>
            {/each}
        </ul>
    </div>
{/if}
