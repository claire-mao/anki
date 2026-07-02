<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";
    import type { PerformanceAttempt } from "@generated/anki/brainlift_pb";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { commitMetricSnapshot, type MetricChanges } from "../metric-change";
    import { presentOnboarding } from "../onboarding-presentation";
    import { emptyStateContent } from "../empty-states";
    import MemorySummary from "../summaries/MemorySummary.svelte";
    import PerformanceSummary from "../summaries/PerformanceSummary.svelte";
    import EstimatedGreSummary from "../summaries/EstimatedGreSummary.svelte";
    import ReadinessSummary from "../summaries/ReadinessSummary.svelte";
    import { checklistRequirementsForEstimatedGre } from "../summary-metrics";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreOnboardingPanel from "../ui/GreOnboardingPanel.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreScoreCard from "../ui/GreScoreCard.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import GreCoverageBars from "../ui/GreCoverageBars.svelte";
    import GreSparkline from "../ui/GreSparkline.svelte";
    import GreTopicMasteryBar from "../ui/GreTopicMasteryBar.svelte";
    import { presentTopicInsights } from "../recommendation-presentation";
    import GreStudyRecommendationList from "../ui/GreStudyRecommendationList.svelte";
    import { rollingAccuracySeries } from "../indicator-utils";
    import { topicDetailsPath } from "../topic-link";
    import type { PageData } from "./$types";

    export let data: PageData;

    const dashboard = data.dashboard;
    const status = data.status;
    const readinessCalibration = data.readinessCalibration;
    const memory = dashboard.memory!;
    const performance = dashboard.performance!;
    const readiness = dashboard.readiness!;
    const estimatedGre = dashboard.estimatedGre!;
    const coverage = dashboard.coverage!;
    const estimatedGreChecklist = checklistRequirementsForEstimatedGre(
        memory,
        performance,
        readiness,
    );
    const recommendedFocus = presentTopicInsights(dashboard.recommendedTopics);

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
        context: "dashboard",
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
            topicInsights: [...dashboard.weakTopics, ...dashboard.recommendedTopics],
            recentActivity: dashboard.recentActivity,
        });
    });

    function formatPercent(value: number): string {
        return `${Math.round(value)}%`;
    }

    function formatRatio(ratio: number): string {
        return formatPercent(ratio * 100);
    }

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }

    function formatAnsweredAt(secs: bigint): string {
        return new Date(Number(secs) * 1000).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }

    function attemptSummary(attempt: PerformanceAttempt): string {
        const parts = [
            attempt.correct ? "Correct" : "Incorrect",
            `${(attempt.responseTimeMs / 1000).toFixed(1)}s`,
            formatAnsweredAt(attempt.answeredAtSecs),
        ];
        return parts.join(" · ");
    }
</script>

<GrePageHeader
    title="Dashboard"
    icon="dashboard"
    subtitle={onboarding.active
        ? "Study, practice, and track your GRE progress"
        : "Scores, coverage, and focus topics"}
    meta="Last updated {formatTimestampMillis(dashboard.computedAtMillis)}"
/>

<GreSection>
    {#if onboarding.active}
        <GreOnboardingPanel model={onboarding} />
    {:else}
    <div class="score-grid gre-stagger">
        <GreScoreCard title="Memory" icon="memory">
            <MemorySummary {memory} />
        </GreScoreCard>

        <GreScoreCard title="Performance" icon="performance">
            <PerformanceSummary {performance} recentAttempts={dashboard.recentActivity} />
        </GreScoreCard>

        <GreScoreCard title="Estimated GRE" icon="score">
            <EstimatedGreSummary
                estimate={estimatedGre}
                {readiness}
                {memory}
                {performance}
                weakTopics={dashboard.weakTopics}
                checklistRequirements={estimatedGreChecklist}
                metricChange={metricChanges.estimatedGre ?? null}
            />
        </GreScoreCard>

        <GreScoreCard title="Readiness score" icon="readiness">
            <ReadinessSummary
                {readiness}
                {memory}
                {performance}
                weakTopics={dashboard.weakTopics}
                metricChange={metricChanges.readiness ?? null}
                confidenceChange={metricChanges.confidence ?? null}
            />
        </GreScoreCard>
    </div>

    <GrePanel title="Topic coverage">
        <GreCoverageBars
            weightedRatio={coverage.weightedRatio}
            unweightedRatio={coverage.unweightedRatio}
            coveredLeafCount={coverage.coveredLeafCount}
            catalogLeafCount={coverage.catalogLeafCount}
        />
    </GrePanel>

    <GrePanel title="Weak topics">
        {#if dashboard.weakTopics.length > 0}
            <ul class="topic-list">
                {#each dashboard.weakTopics as topic}
                    <li>
                        <a href={topicDetailsPath(topic.topicId)}>
                            <strong>{topic.displayName}</strong>
                        </a>
                        <span class="muted">{topic.section} · {formatRatio(topic.examWeight)}</span>
                        <div class="topic-insight-bars">
                            {#if topic.memoryScore !== undefined}
                                <GreTopicMasteryBar label="Memory" value={topic.memoryScore} />
                            {/if}
                            {#if topic.practiceAccuracy !== undefined}
                                <GreTopicMasteryBar label="Practice" value={topic.practiceAccuracy} />
                            {/if}
                        </div>
                    </li>
                {/each}
            </ul>
        {:else}
            <GreEmptyState content={emptyStateContent("weakTopics")} />
        {/if}
    </GrePanel>

    <GrePanel title="Recommended focus">
        {#if recommendedFocus.length > 0}
            <GreStudyRecommendationList recommendations={recommendedFocus} />
        {:else}
            <GreEmptyState content={emptyStateContent("recommendations")} />
        {/if}
    </GrePanel>

    <GrePanel title="Recent practice">
        {#if dashboard.recentActivity.length > 0}
            {@const trend = rollingAccuracySeries(dashboard.recentActivity)}
            {#if trend.length >= 2}
                <GreSparkline points={trend} label="Recent accuracy trend" />
            {/if}
            <ul class="activity-list">
                {#each dashboard.recentActivity as attempt}
                    <li>
                        <strong>{attempt.topic}</strong>
                        <span class="muted">{attemptSummary(attempt)}</span>
                    </li>
                {/each}
            </ul>
        {:else}
            <GreEmptyState content={emptyStateContent("recentPractice")} />
        {/if}
    </GrePanel>
    {/if}
</GreSection>
