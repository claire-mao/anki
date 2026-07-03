<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import type { PerformanceAttempt } from "@generated/anki/brainlift_pb";
    import {
        greNavAction,
        greNavItem,
        runGreNavAction,
        settingsNavAction,
    } from "../gre-navigation";

    import DailyStudyPlan from "../DailyStudyPlan.svelte";
    import { commitMetricSnapshot, type MetricChanges } from "../metric-change";
    import { presentOnboarding } from "../onboarding-presentation";
    import GreOnboardingPanel from "../ui/GreOnboardingPanel.svelte";
    import { milestoneRequirementsForEstimatedGre } from "../summary-metrics";
    import GreButton from "../ui/GreButton.svelte";
    import { emptyStateContent } from "../empty-states";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import { formatRatio } from "../score-format";
    import { rollingAccuracySeries } from "../indicator-utils";
    import GreTopicMasteryBar from "../ui/GreTopicMasteryBar.svelte";
    import GreSparkline from "../ui/GreSparkline.svelte";
    import { topicDetailsPath } from "../topic-link";
    import GreCoverageSummary from "../ui/GreCoverageSummary.svelte";
    import DashboardHero from "./DashboardHero.svelte";
    import GrePageHeader from "../GrePageHeader.svelte";
    import type { PageData } from "./$types";

    import "./home.scss";

    export let data: PageData;

    const dashboard = data.dashboard;
    const plan = data.plan;
    const status = data.status;
    const readinessCalibration = data.readinessCalibration;
    const memory = dashboard.memory!;
    const performance = dashboard.performance!;
    const readiness = dashboard.readiness!;
    const estimatedGre = dashboard.estimatedGre!;
    const coverage = dashboard.coverage!;
    const weakestTopic = data.dashboard.weakTopics[0];
    const dailyPlan = plan.dailyPlan!;
    const dueTotal = status.newCount + status.learnCount + status.reviewCount;
    const estimatedGreChecklist = milestoneRequirementsForEstimatedGre(
        memory,
        performance,
        readiness,
    );

    $: onboarding = presentOnboarding({
        deckExists: status.deckExists,
        deckName: status.deckName,
        memory,
        performance,
        readiness,
        estimatedGre,
        calibration: readinessCalibration.calibration,
        dueTotal,
        weakestTopicId: weakestTopic?.topicId,
        weakestTopicName: weakestTopic?.displayName,
        context: "home",
    });

    function continueStudying(): void {
        if (!status.deckExists) {
            runGreNavAction(settingsNavAction());
            return;
        }
        runGreNavAction(greNavAction(greNavItem("study")));
    }

    function continueStudyingLabel(): string {
        if (!status.deckExists) {
            return "Set up deck";
        }
        if (dueTotal === 0) {
            return "Continue studying";
        }
        return `Start review (${dueTotal} due)`;
    }

    function studyBandHeading(): string {
        if (!status.deckExists) {
            return "Get started";
        }
        if (dueTotal > 0) {
            return "Review due cards";
        }
        return "Continue studying";
    }

    function studyBandMeta(): string {
        if (!status.deckExists) {
            return `Create deck "${status.deckName}" to begin`;
        }
        if (dueTotal > 0) {
            return `${dueTotal} cards due now`;
        }
        return emptyStateContent("noCardsDue").explanation;
    }

    function weakestTopicContext(): string | null {
        if (!weakestTopic) {
            return null;
        }
        return [weakestTopic.section, formatRatio(weakestTopic.examWeight)].join(" · ");
    }

    $: recentTrend = rollingAccuracySeries(dashboard.recentActivity);

    let metricChanges: MetricChanges = {};

    onMount(() => {
        if (!browser) {
            return;
        }
        metricChanges = commitMetricSnapshot({
            memory,
            performance,
            readiness,
            estimatedGre,
            topicInsights: [...dashboard.weakTopics, ...dashboard.recommendedTopics],
            recentActivity: dashboard.recentActivity,
        });
    });

    function attemptSummary(attempt: PerformanceAttempt): string {
        return [
            attempt.correct ? "Correct" : "Incorrect",
            `${(attempt.responseTimeMs / 1000).toFixed(1)}s`,
        ].join(" · ");
    }
</script>

<div class="home-dashboard">
    <GrePageHeader
        title="Dashboard"
        icon="dashboard"
        subtitle={onboarding.active
            ? "Complete study and practice to unlock your first estimated GRE score."
            : "How close you are to your GRE goal"}
    />

    {#if onboarding.active}
        <GreCoverageSummary {coverage} compact showReadinessGate={false} />
        <GreOnboardingPanel model={onboarding} />
    {:else}
        <GreCoverageSummary {coverage} />
        <DashboardHero
            estimate={estimatedGre}
            {readiness}
            {memory}
            {performance}
            {coverage}
            weakTopics={dashboard.weakTopics}
            checklistRequirements={estimatedGreChecklist}
            {metricChanges}
        />
    {/if}

    <section class="home-section" aria-labelledby="daily-mission-heading">
        <DailyStudyPlan plan={dailyPlan} studyStatus={status} compact />
    </section>

    <section class="home-section home-study-band" aria-labelledby="home-study-heading">
        <div class="home-study-copy">
            <h2 class="gre-section-title" id="home-study-heading">
                {studyBandHeading()}
            </h2>
            <p class="home-study-meta">{studyBandMeta()}</p>
        </div>
        <GreButton
            variant="primary"
            size="lg"
            className="home-study-button"
            on:click={continueStudying}
        >
            {continueStudyingLabel()}
        </GreButton>
    </section>

    <div class="home-secondary-grid gre-stagger">
        {#if !onboarding.active}
            <section class="home-panel" aria-labelledby="home-weakest-heading">
                <h2 class="gre-section-title" id="home-weakest-heading">
                    Weakest topic
                </h2>
                {#if weakestTopic}
                    <a
                        class="home-panel-lead"
                        href={topicDetailsPath(weakestTopic.topicId)}
                    >
                        {weakestTopic.displayName}
                    </a>
                    {#if weakestTopicContext()}
                        <p class="home-panel-meta">{weakestTopicContext()}</p>
                    {/if}
                    <div class="home-topic-bars">
                        {#if weakestTopic.memoryScore !== undefined}
                            <GreTopicMasteryBar
                                label="Memory"
                                value={weakestTopic.memoryScore}
                            />
                        {/if}
                        {#if weakestTopic.practiceAccuracy !== undefined}
                            <GreTopicMasteryBar
                                label="Practice"
                                value={weakestTopic.practiceAccuracy}
                            />
                        {/if}
                    </div>
                {:else}
                    <GreEmptyState
                        content={emptyStateContent("homeWeakTopics")}
                        compact
                    />
                {/if}
            </section>

            <section class="home-panel" aria-labelledby="home-recent-heading">
                <h2 class="gre-section-title" id="home-recent-heading">
                    Recent practice
                </h2>
                {#if dashboard.recentActivity.length > 0}
                    {#if recentTrend.length >= 2}
                        <GreSparkline
                            points={recentTrend}
                            label="Recent accuracy trend"
                        />
                    {/if}
                    <ul class="home-recent-list">
                        {#each dashboard.recentActivity as attempt}
                            <li>
                                <span class="home-recent-topic">{attempt.topic}</span>
                                <span class="home-recent-meta">
                                    {attemptSummary(attempt)}
                                </span>
                            </li>
                        {/each}
                    </ul>
                {:else}
                    <GreEmptyState
                        content={emptyStateContent("homeRecentPractice")}
                        compact
                    />
                {/if}
            </section>
        {/if}
    </div>
</div>
