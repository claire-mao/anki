<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";

    import DailyStudyPlan from "../DailyStudyPlan.svelte";
    import { commitMetricSnapshot } from "../metric-change";
    import { presentOnboarding } from "../onboarding-presentation";
    import GreOnboardingPanel from "../ui/GreOnboardingPanel.svelte";
    import GreCoverageSummary from "../ui/GreCoverageSummary.svelte";
    import GrePageHeader from "../GrePageHeader.svelte";
    import DashboardHero from "./DashboardHero.svelte";
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
    const dailyPlan = plan.dailyPlan!;
    const dueTotal = status.newCount + status.learnCount + status.reviewCount;

    $: onboarding = presentOnboarding({
        deckExists: status.deckExists,
        deckName: status.deckName,
        memory,
        performance,
        readiness,
        estimatedGre,
        calibration: readinessCalibration.calibration,
        dueTotal,
        weakestTopicId: dashboard.weakTopics[0]?.topicId,
        weakestTopicName: dashboard.weakTopics[0]?.displayName,
        context: "home",
    });

    onMount(() => {
        if (!browser) {
            return;
        }
        commitMetricSnapshot({
            memory,
            performance,
            readiness,
            estimatedGre,
            topicInsights: [...dashboard.weakTopics, ...dashboard.recommendedTopics],
            recentActivity: dashboard.recentActivity,
        });
    });

    function dashboardSubtitle(onboardingActive: boolean, due: number): string {
        if (onboardingActive) {
            return "Complete a few study sessions to unlock your first score estimate.";
        }
        if (due <= 0) {
            return "Start with today's mission.";
        }
        const noun = due === 1 ? "card" : "cards";
        return `${due} ${noun} ready — start below.`;
    }
</script>

<div class="home-dashboard">
    <GrePageHeader
        title="Dashboard"
        icon="dashboard"
        subtitle={dashboardSubtitle(onboarding.active, dueTotal)}
    />

    {#if onboarding.active}
        <GreOnboardingPanel model={onboarding} />
    {:else}
        <DashboardHero
            estimate={estimatedGre}
            {readiness}
            {memory}
            {performance}
            {coverage}
        />

        <section class="home-mission" aria-label="Today's mission">
            <DailyStudyPlan
                plan={dailyPlan}
                studyStatus={status}
                recentAttempts={dashboard.recentActivity}
                primary
            />
        </section>

        <section class="home-coverage" aria-label="Topic coverage">
            <GreCoverageSummary {coverage} showReadinessGate />
        </section>
    {/if}
</div>
