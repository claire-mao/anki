<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { DashboardTopicInsight, PerformanceAttempt } from "@generated/anki/brainlift_pb";

    import AbstentionRequirements from "../AbstentionRequirements.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    const dashboard = data.dashboard;
    const memory = dashboard.memory!;
    const performance = dashboard.performance!;
    const readiness = dashboard.readiness!;
    const coverage = dashboard.coverage!;

    function formatPercent(value: number): string {
        return `${Math.round(value)}%`;
    }

    function formatRatio(ratio: number): string {
        return formatPercent(ratio * 100);
    }

    function formatRange(low: number | undefined, high: number | undefined): string | null {
        if (low === undefined || high === undefined) {
            return null;
        }
        return `${formatPercent(low)}–${formatPercent(high)}`;
    }

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString();
    }

    function formatAnsweredAt(secs: bigint): string {
        return new Date(Number(secs) * 1000).toLocaleString();
    }

    function topicInsightMeta(topic: DashboardTopicInsight): string {
        const parts: string[] = [topic.section];
        parts.push(formatRatio(topic.examWeight));
        parts.push(topic.covered ? "covered" : "not covered");
        if (topic.studiedCards > 0) {
            parts.push(`${topic.studiedCards} studied cards`);
        }
        if (topic.memoryScore !== undefined) {
            parts.push(`memory ${formatPercent(topic.memoryScore)}`);
        }
        if (topic.practiceAccuracy !== undefined) {
            parts.push(`practice ${formatPercent(topic.practiceAccuracy)}`);
        }
        return parts.join(" · ");
    }

    function attemptSummary(attempt: PerformanceAttempt): string {
        const parts = [
            attempt.topic,
            attempt.correct ? "correct" : "incorrect",
            `${(attempt.responseTimeMs / 1000).toFixed(1)}s`,
            formatAnsweredAt(attempt.answeredAtSecs),
        ];
        if (attempt.confidence !== undefined) {
            parts.push(`confidence ${attempt.confidence}`);
        }
        return parts.join(" · ");
    }
</script>

<h1>Dashboard</h1>

<p class="dashboard-updated muted">
    Last updated {formatTimestampMillis(dashboard.computedAtMillis)}
</p>

<div class="score-grid">
    <div class="score-card">
        <h2>Memory</h2>
        {#if memory.sufficientData && memory.value !== undefined}
            <div class="score-value">{formatPercent(memory.value)}</div>
            {#if formatRange(memory.valueLow, memory.valueHigh)}
                <p class="score-range">{formatRange(memory.valueLow, memory.valueHigh)}</p>
            {/if}
            <p class="muted">{memory.detail}</p>
            <p class="muted">{formatRatio(memory.coverageRatio)} catalog coverage</p>
        {:else}
            <p class="score-abstain">{memory.abstainReason}</p>
            <AbstentionRequirements requirements={memory.abstentionRequirements} />
            <p class="muted">{formatRatio(memory.coverageRatio)} catalog coverage</p>
            <p class="muted">{memory.studiedCards} studied cards</p>
        {/if}
    </div>

    <div class="score-card">
        <h2>Performance</h2>
        {#if performance.sufficientData && performance.value !== undefined}
            <div class="score-value">{formatPercent(performance.value)}</div>
            {#if formatRange(performance.valueLow, performance.valueHigh)}
                <p class="score-range">{formatRange(performance.valueLow, performance.valueHigh)}</p>
            {/if}
            <p class="muted">{performance.detail}</p>
        {:else}
            <p class="score-abstain">{performance.abstainReason}</p>
            <AbstentionRequirements requirements={performance.abstentionRequirements} />
            <p class="muted">{performance.attemptCount} attempts</p>
        {/if}
    </div>

    <div class="score-card">
        <h2>Readiness</h2>
        {#if readiness.sufficientData && readiness.projectedScore !== undefined}
            <div class="score-value">{formatPercent(readiness.projectedScore)}</div>
            {#if formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                <p class="score-range">
                    {formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                </p>
            {/if}
            <p class="muted">{readiness.evidenceSummary}</p>
            {#if readiness.confidenceLevel}
                <p class="muted">
                    {readiness.confidenceLevel} confidence · {formatRatio(readiness.coverageRatio)}
                    coverage
                </p>
            {/if}
        {:else}
            <p class="score-abstain">{readiness.abstainReason}</p>
            <AbstentionRequirements
                requirements={readiness.abstentionRequirements}
                heading="Readiness requires more evidence"
            />
            <p class="muted">{readiness.evidenceSummary}</p>
        {/if}
        {#if readiness.calibrationNote}
            <p class="calibration-note">{readiness.calibrationNote}</p>
        {/if}
        <p class="muted muted-small">
            Readiness data from {formatTimestampMillis(readiness.lastUpdatedMillis)}
        </p>
    </div>
</div>

<div class="gre-panel">
    <h2>Topic coverage</h2>
    <dl class="coverage-stats">
        <div>
            <dt>Weighted coverage</dt>
            <dd>{formatRatio(coverage.weightedRatio)}</dd>
        </div>
        <div>
            <dt>Unweighted coverage</dt>
            <dd>{formatRatio(coverage.unweightedRatio)}</dd>
        </div>
        <div>
            <dt>Covered leaves</dt>
            <dd>{coverage.coveredLeafCount} / {coverage.catalogLeafCount}</dd>
        </div>
    </dl>
</div>

<div class="gre-panel">
    <h2>Weak topics</h2>
    {#if dashboard.weakTopics.length > 0}
        <ul class="topic-list">
            {#each dashboard.weakTopics as topic}
                <li>
                    <strong>{topic.displayName}</strong>
                    <span class="muted">{topicInsightMeta(topic)}</span>
                    <p class="muted">{topic.reason}</p>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<div class="gre-panel">
    <h2>Study recommendations</h2>
    {#if dashboard.recommendedTopics.length > 0}
        <ul class="topic-list">
            {#each dashboard.recommendedTopics as topic}
                <li>
                    <strong>{topic.displayName}</strong>
                    <span class="muted">{topicInsightMeta(topic)}</span>
                    <p class="muted">{topic.reason}</p>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<div class="gre-panel">
    <h2>Recent practice</h2>
    {#if dashboard.recentActivity.length > 0}
        <ul class="activity-list">
            {#each dashboard.recentActivity as attempt}
                <li>
                    <strong>{attempt.questionId}</strong>
                    <span class="muted">{attemptSummary(attempt)}</span>
                </li>
            {/each}
        </ul>
    {/if}
</div>
