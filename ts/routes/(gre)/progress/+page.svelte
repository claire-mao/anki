<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import { goto } from "$app/navigation";
    import type { GraphBounds } from "../../graphs/graph-helpers";
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import { commitMetricSnapshot, type MetricChanges } from "../metric-change";
    import { presentOnboarding } from "../onboarding-presentation";
    import { formatPercent, formatRatio, formatRange } from "../score-format";
    import { ratioToPercent, rollingAccuracySeries } from "../indicator-utils";
    import {
        checklistRequirementsForEstimatedGre,
        estimatedGreChartContext,
        estimatedGreConfidence,
        estimatedGreHero,
        memoryChartContext,
        performanceChartContext,
        readinessChartContext,
        readinessHero,
    } from "../summary-metrics";
    import EstimatedGreSummary from "../summaries/EstimatedGreSummary.svelte";
    import ReadinessSummary from "../summaries/ReadinessSummary.svelte";
    import GreCalibrationPanel from "../ui/GreCalibrationPanel.svelte";
    import GreButton from "../ui/GreButton.svelte";
    import GreMetricChangeInspect from "../ui/GreMetricChangeInspect.svelte";
    import GreOnboardingPanel from "../ui/GreOnboardingPanel.svelte";
    import { chartEmptyLabel, emptyStateTitle } from "../empty-states";
    import { topicDetailsPath } from "../topic-link";
    import ProgressChart from "./ProgressChart.svelte";
    import ProgressKpiCard from "./ProgressKpiCard.svelte";
    import {
        renderEstimatedGreScore,
        renderScoreBar,
        renderTopicMasteryChart,
        type ScoreBarDatum,
    } from "./charts";
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
    const calibrationStats = readinessCalibration.calibration;
    const coverage = dashboard.coverage!;
    const masterySummary = mastery.summary;

    $: onboarding = presentOnboarding({
        deckExists: status.deckExists,
        deckName: status.deckName,
        memory,
        performance,
        readiness,
        estimatedGre,
        calibration: calibrationStats,
        dueTotal: status.newCount + status.learnCount + status.reviewCount,
        weakestTopicId: dashboard.weakTopics[0]?.topicId,
        weakestTopicName: dashboard.weakTopics[0]?.displayName,
        context: "progress",
    });

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
            topicMasterySummary: masterySummary,
            topicMasteryTopics: mastery.topics,
            topicInsights: [...dashboard.weakTopics, ...dashboard.recommendedTopics],
            recentActivity: dashboard.recentActivity,
        });
    });

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }

    function memoryDatum(): ScoreBarDatum {
        if (memory.sufficientData && memory.value !== undefined) {
            return {
                label: "Memory",
                value: memory.value,
                low: memory.valueLow,
                high: memory.valueHigh,
                detail: memoryChartContext(memory, formatRatio),
                color: "var(--state-learn)",
            };
        }
        return {
            label: "Memory",
            abstain: chartEmptyLabel("score"),
            detail: memoryChartContext(memory, formatRatio),
            color: "var(--state-learn)",
        };
    }

    function performanceDatum(): ScoreBarDatum {
        if (performance.sufficientData && performance.value !== undefined) {
            return {
                label: "Performance",
                value: performance.value,
                low: performance.valueLow,
                high: performance.valueHigh,
                detail: performanceChartContext(performance),
                color: "var(--fg-link)",
            };
        }
        return {
            label: "Performance",
            abstain: chartEmptyLabel("score"),
            detail: performanceChartContext(performance),
            color: "var(--fg-link)",
        };
    }

    function readinessDatum(): ScoreBarDatum {
        if (readiness.sufficientData && readiness.projectedScore !== undefined) {
            return {
                label: "Readiness score",
                value: readiness.projectedScore,
                low: readiness.projectedScoreLow,
                high: readiness.projectedScoreHigh,
                detail: readinessChartContext(readiness, formatRatio),
                color: "var(--state-review)",
            };
        }
        return {
            label: "Readiness score",
            abstain: chartEmptyLabel("score"),
            detail: readinessChartContext(readiness, formatRatio),
            color: "var(--state-review)",
        };
    }

    $: practiceTrend = rollingAccuracySeries(dashboard.recentActivity);
    $: estimatedGreConfidenceLabel = estimatedGreConfidence(estimatedGre, readiness);
    $: estimatedGreChecklist = checklistRequirementsForEstimatedGre(
        memory,
        performance,
        readiness,
    );

    function estimatedGreKpiDetail(): string {
        if (estimatedGre.combinedScore === undefined) {
            return emptyStateTitle("estimatedGre");
        }
        return (
            formatRange(
                estimatedGre.combinedScoreLow,
                estimatedGre.combinedScoreHigh,
            ) ?? ""
        );
    }

    function readinessKpiDetail(): string {
        const range = formatRange(
            readiness.projectedScoreLow,
            readiness.projectedScoreHigh,
        );
        if (range) {
            return range;
        }
        return readinessChartContext(readiness, formatRatio);
    }

    function coverageKpiDetail(): string {
        return `${formatRatio(coverage.unweightedRatio)} unweighted · ${coverage.coveredLeafCount}/${coverage.catalogLeafCount} leaves`;
    }

    function studiedCardsKpiDetail(): string {
        const topics = masterySummary?.topicCount ?? 0;
        return `${topics} topics tracked`;
    }

    function topicMasterySubtitle(): string {
        const topics = masterySummary?.topicCount ?? 0;
        return `${topics} topics · average retrievability by topic`;
    }

    const renderMemoryChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderScoreBar(svg, bounds, memoryDatum());
    const renderPerformanceChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderScoreBar(svg, bounds, performanceDatum());
    const renderReadinessChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderScoreBar(svg, bounds, readinessDatum());
    const renderEstimatedGreChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderEstimatedGreScore(svg, bounds, estimatedGre);
    const renderMasteryChart = (svg: SVGElement, bounds: GraphBounds) =>
        renderTopicMasteryChart(svg, bounds, mastery.topics, (topicId) => {
            void goto(topicDetailsPath(topicId));
        });
</script>

<GrePageHeader
    title="Progress"
    icon="progress"
    subtitle="Track your learning and prediction quality."
    meta="Last updated {formatTimestampMillis(dashboard.computedAtMillis)}"
/>

<GreSection>
    {#if onboarding.active}
        <GreOnboardingPanel model={onboarding} />
    {:else}
        <div class="progress-dashboard">
            <section class="progress-kpi-row gre-stagger" aria-label="Key metrics">
                <ProgressKpiCard
                    label="Estimated GRE"
                    value={estimatedGreHero(estimatedGre)}
                    detail={estimatedGreKpiDetail() || null}
                    confidence={estimatedGreConfidenceLabel}
                    metricChange={metricChanges.estimatedGre ?? null}
                />
                <ProgressKpiCard
                    label="Readiness score"
                    value={readinessHero(readiness, formatPercent)}
                    detail={readinessKpiDetail()}
                    ringValue={readiness.sufficientData &&
                    readiness.projectedScore !== undefined
                        ? readiness.projectedScore
                        : null}
                    ringColor="var(--state-review)"
                    metricChange={metricChanges.readiness ?? null}
                />
                <ProgressKpiCard
                    label="Coverage"
                    value={formatRatio(coverage.weightedRatio)}
                    detail={coverageKpiDetail()}
                    barValue={ratioToPercent(coverage.weightedRatio)}
                />
                <ProgressKpiCard
                    label="Practice attempts"
                    value={String(performance.attemptCount)}
                    sparklinePoints={practiceTrend}
                />
                <ProgressKpiCard
                    label="Studied cards"
                    value={String(masterySummary?.studiedCards ?? 0)}
                    detail={studiedCardsKpiDetail()}
                />
            </section>

            <h2 class="progress-section-label">Predictions</h2>

            <div class="progress-predictions gre-stagger">
                <div class="progress-prediction-panel">
                    <EstimatedGreSummary
                        estimate={estimatedGre}
                        {readiness}
                        {memory}
                        {performance}
                        weakTopics={dashboard.weakTopics}
                        checklistRequirements={estimatedGreChecklist}
                        calibration={calibrationStats}
                        variant="compact"
                    />
                </div>
                <div class="progress-prediction-panel">
                    <ReadinessSummary
                        {readiness}
                        {memory}
                        {performance}
                        {coverage}
                        weakTopics={dashboard.weakTopics}
                        calibration={calibrationStats}
                        confidenceChange={metricChanges.confidence ?? null}
                        variant="compact"
                    />
                </div>
            </div>

            <div class="progress-calibration-header">
                <h2 class="progress-section-label">Calibration</h2>
                <GreButton variant="ghost" size="sm" href="/readiness">
                    View calibration
                </GreButton>
            </div>

            <GreCalibrationPanel
                {readiness}
                calibration={calibrationStats!}
                variant="compact"
                showImprovements={false}
            />

            <h2 class="progress-section-label">Charts</h2>

            <div class="progress-charts gre-stagger">
                <ProgressChart
                    title="Memory"
                    subtitle={memoryDatum().detail}
                    renderChart={renderMemoryChart}
                />
                <ProgressChart
                    title="Performance"
                    subtitle={performanceDatum().detail}
                    renderChart={renderPerformanceChart}
                />
                <ProgressChart
                    title="Estimated GRE"
                    subtitle={estimatedGreChartContext(estimatedGre, readiness)}
                    renderChart={renderEstimatedGreChart}
                />
                <ProgressChart
                    title="Readiness score"
                    subtitle={readinessDatum().detail}
                    renderChart={renderReadinessChart}
                />
                {#if metricChanges.topicMastery}
                    <div class="progress-chart-change">
                        <GreMetricChangeInspect change={metricChanges.topicMastery} />
                    </div>
                {/if}
                <ProgressChart
                    title="Topic mastery"
                    subtitle={topicMasterySubtitle()}
                    renderChart={renderMasteryChart}
                    wide
                    tall
                />
            </div>
        </div>
    {/if}
</GreSection>
