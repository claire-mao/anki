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
    import { missionIntro } from "./daily-mission";
    import { runGreNavAction } from "./gre-navigation";
    import GreIcon from "./GreIcon.svelte";
    import GreText from "./ui/GreText.svelte";

    export let plan: DailyStudyPlan;
    export let studyStatus: GreStudyStatusResponse | undefined = undefined;
    export let recentAttempts: PerformanceAttempt[] | undefined = undefined;
    export let compact = false;
    export let primary = false;
    export let hideHeader = false;
</script>

<section class="daily-mission" class:daily-mission-compact={compact} class:daily-mission-primary={primary}>
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

    {#if compact && !primary}
        <p class="daily-mission-more daily-mission-more-compact">
            <a
                href="/study-plan"
                on:click={(event) =>
                    runGreNavAction(
                        {
                            label: "Study plan",
                            bridge: "greOpenStudyPlan",
                            href: "/study-plan",
                        },
                        event,
                    )}
            >
                View full study plan
            </a>
        </p>
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

    .daily-mission-more {
        margin: var(--gre-space-4) 0 0;
    }

    .daily-mission-more-compact {
        margin-top: var(--gre-space-3);
        margin-bottom: 0;
    }

    .daily-mission-more a {
        color: var(--fg-link);
        font-weight: var(--gre-weight-label);
        text-decoration: none;
    }

    .daily-mission-more a:hover {
        text-decoration: underline;
    }

    .daily-mission-primary .daily-mission-grid {
        grid-template-columns: 1fr;
    }

    .daily-mission-primary :global(.daily-mission-card),
    .daily-mission-primary :global(.gre-ds-recommendation-card) {
        border-color: color-mix(in srgb, var(--gre-accent) 18%, var(--border));
        background: color-mix(in srgb, var(--gre-accent-soft) 65%, var(--canvas));
    }
</style>
