<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";

    import type { PageData } from "./$types";

    export let data: PageData;

    const status = data.status;

    function startReview(): void {
        if (bridgeCommandsAvailable()) {
            bridgeCommand("brainliftStartReview");
        }
    }
</script>

<h1>GRE memory review</h1>

<div class="brainlift-panel">
    <p>
        BrainLift always reviews the <strong>{status.deckName}</strong>
         deck using Anki's FSRS scheduler. Your flashcard ratings stay separate from GRE practice
        questions.
    </p>

    {#if !status.deckExists}
        <p class="muted">
            Create a deck named "{status.deckName}" and add GRE flashcards tagged with
            <code>gre::</code>
             topics.
        </p>
    {:else}
        <p>
            Due now: {status.newCount} new · {status.learnCount} learning ·
            {status.reviewCount} review
        </p>
    {/if}

    <button class="btn btn-primary primary-button" on:click={startReview}>
        Start GRE review
    </button>
</div>
