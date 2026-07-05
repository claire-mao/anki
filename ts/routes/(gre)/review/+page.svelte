<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";
    import { fade } from "svelte/transition";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { GRE_CTA_REVIEW, GRE_CTA_STUDY_PLAN } from "../gre-navigation";
    import { buildStudyCaughtUpSummary } from "../session-completion";
    import { greMotionDuration } from "../motion";
    import GreButton from "../ui/GreButton.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreSessionCompletePanel from "../ui/GreSessionCompletePanel.svelte";
    import GreText from "../ui/GreText.svelte";
    import { shouldAutoLaunchReview } from "./study-launch";
    import type { PageData } from "./$types";

    import { onMount } from "svelte";
    import "../ui/session-complete.scss";
    import "./study.scss";

    export let data: PageData;

    const status = data.status;
    const dashboard = data.dashboard;
    const dueTotal = status.newCount + status.learnCount + status.reviewCount;
    let launched = false;

    $: studySummary = buildStudyCaughtUpSummary({
        weakTopics: dashboard.weakTopics,
        recommendedTopics: dashboard.recommendedTopics,
        dueTotal,
        deckName: status.deckName,
        studiedCards: dashboard.memory?.studiedCards ?? 0,
        coveredLeafCount: dashboard.coverage?.coveredLeafCount ?? 0,
    });

    function startReview(): void {
        if (bridgeCommandsAvailable()) {
            bridgeCommand("greStartReview");
        }
    }

    onMount(() => {
        if (bridgeCommandsAvailable() && shouldAutoLaunchReview(status)) {
            launched = true;
            startReview();
        }
    });
</script>

<GrePageHeader title="Study" icon="study" subtitle="Built-in GRE flashcards." />

{#if launched}
    <GrePanel>
        <div
            class="study-loading"
            role="status"
            aria-live="polite"
            transition:fade={{ duration: greMotionDuration(160) }}
        >
            <div class="gre-loading-spinner" aria-hidden="true"></div>
            <GreText variant="caption" muted>Opening your review session…</GreText>
        </div>
    </GrePanel>
{:else if !status.deckExists}
    <GrePanel className="study-guide">
        <h2 class="study-guide-title">Couldn't load your GRE flashcards</h2>
        <p class="study-guide-body">
            GRE Atlas includes starter flashcards. Reload this page to try again.
        </p>
        <GreButton variant="primary" on:click={() => location.reload()}>
            Try again
        </GreButton>
    </GrePanel>
{:else if dueTotal > 0}
    <GrePanel className="study-guide study-guide-active">
        <h2 class="study-guide-title">Ready to review</h2>
        <p class="study-guide-body">
            You have {dueTotal} card{dueTotal === 1 ? "" : "s"} ready. A few minutes now keeps
            this material fresh for test day.
        </p>
        <GreButton variant="primary" size="lg" on:click={startReview}>
            {GRE_CTA_REVIEW}
        </GreButton>
    </GrePanel>
{:else}
    <GrePanel className="study-guide study-caught-up">
        <GreSessionCompletePanel
            summary={studySummary}
            secondaryLabel={studySummary.secondaryAction?.label ?? GRE_CTA_STUDY_PLAN}
        />
    </GrePanel>
{/if}
