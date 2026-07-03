<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";
    import { fade } from "svelte/transition";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { runGreNavAction, settingsNavAction } from "../gre-navigation";
    import GreButton from "../ui/GreButton.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreText from "../ui/GreText.svelte";
    import type { PageData } from "./$types";

    import { onMount } from "svelte";

    export let data: PageData;

    const status = data.status;
    let launched = false;

    function startReview(): void {
        if (bridgeCommandsAvailable()) {
            bridgeCommand("greStartReview");
        }
    }

    onMount(() => {
        if (bridgeCommandsAvailable()) {
            launched = true;
            startReview();
        }
    });
</script>

<GrePageHeader
    title="Study"
    icon="study"
    subtitle="Flashcard review for the GRE Atlas deck."
/>

<GrePanel>
    {#if launched}
        <div
            class="gre-loading"
            role="status"
            aria-live="polite"
            transition:fade={{ duration: 180 }}
        >
            <div class="gre-loading-spinner" aria-hidden="true"></div>
            <GreText variant="caption" muted className="gre-loading-caption">
                Opening your GRE review session…
            </GreText>
        </div>
    {:else if !status.deckExists}
        <GreText variant="body" muted>
            Create a deck named "{status.deckName}" and add GRE flashcards tagged with
            <code>gre::</code>
            topics.
        </GreText>
        <GreButton
            variant="primary"
            className="gre-ds-btn-spaced"
            on:click={() => runGreNavAction(settingsNavAction())}
        >
            Set up deck
        </GreButton>
    {:else}
        <GreText variant="body">Due now across your GRE deck:</GreText>
        <div class="study-due-grid gre-stagger" aria-label="Due card counts">
            <div class="study-due-stat">
                <strong>{status.newCount}</strong>
                <span>New</span>
            </div>
            <div class="study-due-stat">
                <strong>{status.learnCount}</strong>
                <span>Learning</span>
            </div>
            <div class="study-due-stat">
                <strong>{status.reviewCount}</strong>
                <span>Review</span>
            </div>
        </div>
        <GreButton
            variant="primary"
            className="gre-ds-btn-spaced"
            on:click={startReview}
        >
            Start review
        </GreButton>
    {/if}
</GrePanel>
