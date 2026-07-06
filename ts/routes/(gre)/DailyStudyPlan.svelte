<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        DailyStudyPlan,
        GreStudyStatusResponse,
        PerformanceAttempt,
    } from "@generated/anki/brainlift_pb";

    import DailyMissionTask from "./DailyMissionTask.svelte";
    import {
        dailyMissionComplete,
        missionIntro,
        type DailyMissionProgressContext,
        startOfLocalDaySecs,
    } from "./daily-mission";
    import { runGreNavAction, studyPlanNavAction } from "./gre-navigation";
    import GreButton from "./ui/GreButton.svelte";
    import GreIcon from "./GreIcon.svelte";
    import GreText from "./ui/GreText.svelte";

    export let plan: DailyStudyPlan;
    export let studyStatus: GreStudyStatusResponse | undefined = undefined;
    export let recentAttempts: PerformanceAttempt[] | undefined = undefined;
    export let compact = false;
    export let primary = false;
    export let hideHeader = false;

    $: progressContext = {
        studyStatus,
        recentAttempts,
        dayStartSecs: startOfLocalDaySecs(),
    } satisfies DailyMissionProgressContext;
    $: missionComplete = dailyMissionComplete(plan, progressContext);
</script>

<section
    class="daily-mission"
    class:daily-mission-compact={compact}
    class:daily-mission-primary={primary}
>
    {#if !hideHeader}
        <header class="daily-mission-header">
            <GreText
                variant="h3"
                tag="h2"
                className="daily-mission-heading gre-text-label-row"
                id="daily-mission-heading"
            >
                <GreIcon name="calendar" size="sm" />
                Today's mission
            </GreText>
            <p class="daily-mission-intro">{missionIntro(plan.tasks.length)}</p>
        </header>
    {/if}

    <div class="daily-mission-grid gre-stagger">
        {#each plan.tasks as task (task.id + (task.topicId ?? "") + task.title)}
            <DailyMissionTask {task} {studyStatus} {recentAttempts} />
        {/each}
    </div>

    {#if missionComplete}
        <div class="daily-mission-complete" aria-live="polite">
            <p class="daily-mission-complete-title">Today's mission complete</p>
            <p class="daily-mission-complete-copy">
                You finished every focus action for today. Come back tomorrow for a fresh set.
            </p>
            {#if !primary}
                <GreButton
                    variant="secondary"
                    size="sm"
                    className="daily-mission-complete-action"
                    on:click={(event) => runGreNavAction(studyPlanNavAction("Back to dashboard"), event)}
                >
                    Back to dashboard
                </GreButton>
            {/if}
        </div>
    {/if}
</section>

<style lang="scss">
    .daily-mission-header {
        margin-bottom: var(--gre-space-4);
    }

    .daily-mission-compact .daily-mission-header {
        margin-bottom: var(--gre-space-3);
    }

    .daily-mission-intro {
        margin: var(--gre-space-2) 0 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .daily-mission-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(min(100%, 16rem), 1fr));
        gap: var(--gre-space-3);
    }

    .daily-mission-compact .daily-mission-grid {
        gap: var(--gre-space-2);
    }

    .daily-mission-primary .daily-mission-grid {
        grid-template-columns: 1fr;
    }

    .daily-mission-primary :global(.daily-mission-card),
    .daily-mission-primary :global(.gre-ds-recommendation-card) {
        border-color: color-mix(in srgb, var(--gre-accent) 18%, var(--border));
        background: color-mix(in srgb, var(--gre-accent-soft) 65%, var(--canvas));
    }

    .daily-mission-complete {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        margin-top: var(--gre-space-3);
        padding: var(--gre-space-3);
        border-radius: var(--gre-radius-md);
        border: 1px solid color-mix(in srgb, var(--gre-success) 24%, var(--border));
        background: color-mix(in srgb, var(--gre-success) 8%, var(--canvas));
    }

    .daily-mission-complete-title {
        margin: 0;
        font-size: var(--gre-font-h3);
        font-weight: var(--gre-weight-h3);
        line-height: var(--gre-lh-h3);
        color: var(--fg);
    }

    .daily-mission-complete-copy {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .daily-mission-complete :global(.daily-mission-complete-action) {
        align-self: flex-start;
    }
</style>
