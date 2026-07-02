<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { AbstentionRequirement } from "@generated/anki/brainlift_pb";

    import type { EmptyStateContent } from "../empty-states";
    import {
        requirementUnlockLabel,
        sortRequirementsForProgress,
    } from "../empty-states";
    import { runGreNavAction } from "../gre-navigation";
    import GreButton from "./GreButton.svelte";

    export let content: EmptyStateContent;
    export let requirements: AbstentionRequirement[] = [];
    export let compact = false;
    export let inline = false;
    export let showChecklist = true;
    export let showAction = true;

    const dispatch = createEventDispatcher<{ action: void }>();

    $: sortedRequirements = sortRequirementsForProgress(requirements);
    $: hasChecklist = showChecklist && sortedRequirements.length > 0;

    function handleAction(): void {
        if (content.action.href || content.action.bridge) {
            runGreNavAction(content.action);
            return;
        }
        dispatch("action");
    }
</script>

<div
    class="gre-empty-state"
    class:gre-empty-state-compact={compact}
    class:gre-empty-state-inline={inline}
    aria-labelledby="gre-empty-title"
>
    {#if content.kicker}
        <p class="gre-empty-kicker">{content.kicker}</p>
    {/if}
    <h3 class="gre-empty-title" id="gre-empty-title">{content.title}</h3>
    <p class="gre-empty-explanation">{content.explanation}</p>

    {#if hasChecklist}
        <div class="gre-empty-checklist-block">
            <p class="gre-checklist-heading">Unlock milestones</p>
            <ul class="gre-checklist gre-empty-checklist" aria-label="Unlock milestones">
                {#each sortedRequirements as req (req.id)}
                    <li class:gre-checklist-met={req.met}>
                        <span class="gre-checklist-mark" aria-hidden="true">
                            {req.met ? "✓" : "○"}
                        </span>
                        <span class="sr-only">{req.met ? "Complete:" : "Incomplete:"}</span>
                        <span class="gre-checklist-label">{requirementUnlockLabel(req)}</span>
                    </li>
                {/each}
            </ul>
            <p class="gre-empty-unlock-goal">{content.unlockGoal}</p>
        </div>
    {:else if content.unlockGoal}
        <p class="gre-empty-unlock-goal">{content.unlockGoal}</p>
    {/if}

    {#if showAction}
        {#if content.action.href || content.action.bridge}
            <GreButton
                variant="secondary"
                size={compact ? "sm" : "md"}
                className="gre-empty-action"
                on:click={handleAction}
            >
                {content.action.label}
            </GreButton>
        {:else}
            <GreButton
                variant="secondary"
                size={compact ? "sm" : "md"}
                className="gre-empty-action"
                on:click={handleAction}
            >
                {content.action.label}
            </GreButton>
        {/if}
    {/if}
</div>

<style lang="scss">
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }
</style>
