<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        GreStudyStatusResponse,
        StudyPlanDailyTask,
    } from "@generated/anki/brainlift_pb";

    import {
        missionAction,
        missionDescription,
        missionIcon,
        missionProgress,
        missionTitle,
        runMissionAction,
    } from "./daily-mission";
    import { runGreNavAction } from "./gre-navigation";
    import { presentDailyFocusTask } from "./recommendation-presentation";
    import GreIcon from "./GreIcon.svelte";
    import GreButton from "./ui/GreButton.svelte";
    import GreStudyRecommendationCard from "./ui/GreStudyRecommendationCard.svelte";
    import GreText from "./ui/GreText.svelte";

    export let task: StudyPlanDailyTask;
    export let studyStatus: GreStudyStatusResponse | undefined = undefined;

    $: focusRecommendation = presentDailyFocusTask(task);
    $: icon = missionIcon(task);
    $: title = missionTitle(task);
    $: description = missionDescription(task);
    $: progress = missionProgress(task, studyStatus);
    $: action = missionAction(task);

    function onAction(): void {
        runMissionAction(action);
    }
</script>

{#if focusRecommendation}
    <GreStudyRecommendationCard recommendation={focusRecommendation} compact />
{:else}
    <article class="gre-ds-mission-card daily-mission-card">
        <div class="daily-mission-card-head">
            <div class="gre-ds-icon-box daily-mission-icon" aria-hidden="true">
                <GreIcon name={icon} size="lg" />
            </div>
            <div class="daily-mission-copy">
                <GreText variant="h3" tag="h3" className="daily-mission-title">
                    {title}
                </GreText>
                <GreText variant="caption" muted className="daily-mission-description">
                    {description}
                </GreText>
            </div>
        </div>

        <div class="daily-mission-progress">
            {#if progress.showBar !== false}
                <div
                    class="gre-ds-progress-track daily-mission-progress-track"
                    role="progressbar"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow={progress.value}
                    aria-label="{title} progress"
                >
                    <div
                        class="gre-ds-progress-fill daily-mission-progress-fill"
                        style:width="{progress.value}%"
                    ></div>
                </div>
            {/if}
            <div class="daily-mission-progress-meta">
                <span class="daily-mission-progress-label">{progress.label}</span>
                {#if progress.detail}
                    <span class="daily-mission-progress-detail">{progress.detail}</span>
                {/if}
            </div>
        </div>

        {#if action.bridge}
            <GreButton
                variant="primary"
                size="sm"
                className="daily-mission-action"
                on:click={onAction}
            >
                {action.label}
            </GreButton>
        {:else if action.href}
            <GreButton
                variant="primary"
                size="sm"
                className="daily-mission-action"
                href={action.href}
                on:click={(event) => runGreNavAction(action, event)}
            >
                {action.label}
            </GreButton>
        {/if}
    </article>
{/if}

<style lang="scss">
    .daily-mission-card-head {
        display: flex;
        gap: var(--gre-space-3);
        align-items: flex-start;
    }

    .daily-mission-copy {
        min-width: 0;
    }

    .daily-mission-card :global(.daily-mission-title) {
        margin: 0;
    }

    .daily-mission-card :global(.daily-mission-description) {
        margin: var(--gre-space-1) 0 0;
    }

    .daily-mission-progress {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        margin-top: auto;
    }

    .daily-mission-progress-meta {
        display: flex;
        flex-wrap: wrap;
        gap: var(--gre-space-1) var(--gre-space-2);
        align-items: baseline;
        justify-content: space-between;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
    }

    .daily-mission-progress-label {
        font-weight: var(--gre-weight-label);
        color: var(--fg);
    }

    .daily-mission-progress-detail {
        color: var(--fg-subtle);
    }

    .daily-mission-card :global(.daily-mission-action) {
        align-self: flex-start;
    }
</style>
