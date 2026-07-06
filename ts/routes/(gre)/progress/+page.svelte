<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import { goto } from "$app/navigation";
    import { getPerformanceChart } from "@generated/backend";
    import type { PerformanceChartBucket } from "@generated/anki/brainlift_pb";
    import type { GraphBounds } from "../../graphs/graph-helpers";
    import { defaultGraphBounds } from "../../graphs/graph-helpers";
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import GreChip from "../ui/GreChip.svelte";
    import GreOnboardingPanel from "../ui/GreOnboardingPanel.svelte";
    import GreMetricChangeInspect from "../ui/GreMetricChangeInspect.svelte";
    import ProgressChart from "./ProgressChart.svelte";
    import { commitMetricSnapshot, type MetricChanges } from "../metric-change";
    import { presentOnboarding } from "../onboarding-presentation";
    import { type AccuracyTrendHorizon } from "../indicator-utils";
    import {
        greNavAction,
        greNavItem,
        GRE_CTA_PRACTICE,
        GRE_CTA_REVIEW,
    } from "../gre-navigation";
    import { topicDetailsPath } from "../topic-link";
    import {
        renderPerformanceChart,
        renderTopicMasteryChart,
        topicMasteryChartHeight,
        topicMasteryChartRows,
        topicMasteryChartSubtitle,
    } from "./charts";
    import {
        performanceChartHasData,
        performanceChartHorizonProto,
    } from "./performance-chart-presentation";
    import {
        topicMasteryClusterNote,
        topicMasterySectionExplanation,
    } from "./topic-mastery-presentation";
    import type { PageData } from "./$types";

    import "../gre.scss";
    import "./progress.scss";

    export let data: PageData;

    const scores = data.scores;
    const dashboard = data.dashboard;
    const mastery = data.mastery;
    const status = data.status;
    const readinessCalibration = data.readinessCalibration;
    const memory = scores.memory!;
    const performance = scores.performance!;
    const readiness = scores.readiness!;
    const estimatedGre = scores.estimatedGre!;
    const masterySummary = mastery.summary;

    const performanceHorizons: { id: AccuracyTrendHorizon; label: string }[] = [
        { id: "1d", label: "Last 1 day" },
        { id: "3d", label: "Last 3 days" },
        { id: "7d", label: "Last 7 days" },
        { id: "30d", label: "Last 30 days" },
        { id: "all", label: "All time" },
    ];
    const performanceChartBounds: GraphBounds = {
        ...defaultGraphBounds(),
        height: 360,
        marginLeft: 58,
        marginBottom: 48,
    };
    $: topicMasteryRows = topicMasteryChartRows(mastery.topics);
    $: topicMasteryBounds = {
        ...defaultGraphBounds(),
        width: 960,
        height: topicMasteryChartHeight(topicMasteryRows.length),
        marginLeft: 8,
        marginRight: 8,
        marginTop: 8,
        marginBottom: 8,
    };

    let performanceHorizon: AccuracyTrendHorizon = "30d";
    let performanceChartBuckets: PerformanceChartBucket[] = data.performanceChart.buckets;
    let hasPerformanceAttempts = data.performanceChart.hasAnyAttempts;
    let loadedPerformanceHorizon: AccuracyTrendHorizon = performanceHorizon;

    async function refreshPerformanceChart(horizon: AccuracyTrendHorizon): Promise<void> {
        const response = await getPerformanceChart({
            horizon: performanceChartHorizonProto(horizon),
            topicPrefix: "",
        });
        performanceChartBuckets = response.buckets;
        hasPerformanceAttempts = response.hasAnyAttempts;
        loadedPerformanceHorizon = horizon;
    }

    $: if (browser && performanceHorizon !== loadedPerformanceHorizon) {
        void refreshPerformanceChart(performanceHorizon);
    }

    $: performanceChartHasPlottedData = performanceChartHasData(performanceChartBuckets);
    $: renderPerformanceChartView = (svg: SVGElement, bounds: GraphBounds) =>
        renderPerformanceChart(svg, bounds, performanceChartBuckets);

    $: onboarding = presentOnboarding({
        deckExists: status.deckExists,
        deckName: status.deckName,
        memory,
        performance,
        readiness,
        estimatedGre,
        calibration: readinessCalibration.calibration,
        dueTotal: status.newCount + status.learnCount + status.reviewCount,
        weakestTopicId: dashboard.weakTopics[0]?.topicId,
        weakestTopicName: dashboard.weakTopics[0]?.displayName,
        context: "progress",
    });

    let changes: MetricChanges = {};

    onMount(() => {
        if (!browser) {
            return;
        }
        changes = commitMetricSnapshot({
            memory,
            performance,
            readiness,
            estimatedGre,
            topicMasterySummary: masterySummary,
            topicMasteryTopics: mastery.topics,
            topicInsights: [...dashboard.weakTopics, ...dashboard.recommendedTopics],
            recentActivity: dashboard.recentActivity,
        });
    });

    $: topicMasterySubtitle = topicMasteryChartSubtitle(
        masterySummary?.topicCount ?? 0,
        mastery.topics,
        masterySummary?.topicsStudied,
    );

    $: topicMasteryClusterNoteText = topicMasteryClusterNote(mastery.topics);

    const renderMasteryChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderTopicMasteryChart(svg, bounds, mastery.topics, (topicId) => {
            void goto(topicDetailsPath(topicId));
        });
</script>

<GrePageHeader
    title="Progress"
    icon="progress"
    subtitle="See how far you've come, then keep going."
/>

<GreSection>
    {#if onboarding.active}
        <GreOnboardingPanel model={onboarding} />
    {:else}
        <section class="progress-next-action">
            <div class="progress-next-action-copy">
                <h2 class="gre-section-title">Keep improving</h2>
                <p class="progress-next-action-hint">
                    A few minutes of review or practice moves your score forward.
                </p>
            </div>
            <GreButtonRow className="progress-next-action-buttons">
                <GreButton
                    variant="primary"
                    navAction={greNavAction(greNavItem("practice"))}
                >
                    {GRE_CTA_PRACTICE}
                </GreButton>
                <GreButton navAction={greNavAction(greNavItem("study"))}>
                    {GRE_CTA_REVIEW}
                </GreButton>
            </GreButtonRow>
        </section>

        <div class="progress-history">
            <section class="progress-group" aria-labelledby="progress-memory">
                <h2 class="gre-section-title" id="progress-memory">Memory</h2>
                {#if changes.topicMastery}
                    <GreMetricChangeInspect change={changes.topicMastery} />
                {:else}
                    <p class="progress-group-empty">
                        Your memory retention trend builds as you review cards across
                        multiple days.
                    </p>
                {/if}
            </section>

            <section class="progress-group" aria-labelledby="progress-performance">
                <h2 class="gre-section-title" id="progress-performance">Performance</h2>
                {#if hasPerformanceAttempts}
                    <div
                        class="progress-horizons"
                        role="group"
                        aria-label="Performance time range"
                    >
                        {#each performanceHorizons as horizon (horizon.id)}
                            <GreChip
                                active={performanceHorizon === horizon.id}
                                on:click={() => (performanceHorizon = horizon.id)}
                            >
                                {horizon.label}
                            </GreChip>
                        {/each}
                    </div>
                {/if}
                {#if performanceChartHasPlottedData}
                    <div class="progress-performance-chart">
                        <ProgressChart
                            ariaLabel="Accuracy trend"
                            renderChart={renderPerformanceChartView}
                            bounds={performanceChartBounds}
                            wide
                            tall
                        />
                    </div>
                {:else}
                    <p class="progress-group-empty">
                        {#if !hasPerformanceAttempts}
                            Answer a few more practice questions to see your accuracy
                            trend.
                        {:else}
                            Answer a few more practice questions in this period to see
                            your accuracy trend.
                        {/if}
                    </p>
                {/if}
            </section>

            <section class="progress-group" aria-labelledby="progress-readiness">
                <h2 class="gre-section-title" id="progress-readiness">Readiness</h2>
                {#if changes.readiness || changes.estimatedGre}
                    {#if changes.readiness}
                        <GreMetricChangeInspect change={changes.readiness} />
                    {/if}
                    {#if changes.estimatedGre}
                        <GreMetricChangeInspect change={changes.estimatedGre} />
                    {/if}
                {:else}
                    <p class="progress-group-empty">
                        Your readiness and estimated score changes appear here after
                        your next study sessions.
                    </p>
                {/if}
            </section>

            <section class="progress-group" aria-labelledby="progress-coverage">
                <h2 class="gre-section-title" id="progress-coverage">Coverage</h2>
                <p class="progress-group-empty">
                    Coverage growth shows here as you study cards across new GRE topics.
                </p>
            </section>

            <section class="progress-group progress-topic-mastery" aria-labelledby="progress-topic-mastery">
                <h2 class="gre-section-title" id="progress-topic-mastery">Topic mastery</h2>
                <p class="progress-group-explanation">
                    {topicMasterySectionExplanation()}
                </p>
                <p class="progress-group-caption">{topicMasterySubtitle}</p>
                {#if topicMasteryClusterNoteText}
                    <p class="progress-group-cluster-note">{topicMasteryClusterNoteText}</p>
                {/if}
                <div class="progress-charts progress-charts-mastery gre-stagger">
                    <ProgressChart
                        ariaLabel="Topic mastery"
                        renderChart={renderMasteryChart}
                        bounds={topicMasteryBounds}
                        wide
                        tall
                        extraTall
                        scrollable
                        fluid
                    />
                </div>
            </section>
        </div>
    {/if}
</GreSection>
