<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        AbstentionRequirement,
        DashboardCoverage,
        DashboardTopicInsight,
        EstimatedGreScore,
        MemoryScore,
        PerformanceScore,
        ReadinessScore,
    } from "@generated/anki/brainlift_pb";

    import EstimatedGreSummary from "../summaries/EstimatedGreSummary.svelte";
    import ReadinessSummary from "../summaries/ReadinessSummary.svelte";
    import type { MetricChanges } from "../metric-change";
    import { coverageAwareReadinessUnlocked } from "../coverage-presentation";
    import { estimatedGreHero } from "../summary-metrics";
    import { formatGreScoreRange, formatRange } from "../score-format";
    import GreIcon from "../GreIcon.svelte";
    import GreProgressRing from "../ui/GreProgressRing.svelte";

    export let estimate: EstimatedGreScore;
    export let readiness: ReadinessScore;
    export let memory: MemoryScore;
    export let performance: PerformanceScore;
    export let coverage: DashboardCoverage | undefined = undefined;
    export let weakTopics: DashboardTopicInsight[] = [];
    export let checklistRequirements: AbstentionRequirement[] = [];
    export let metricChanges: MetricChanges = {};
</script>

<section class="dashboard-hero gre-animate-in" aria-label="GRE goal progress">
    <div class="dashboard-hero-scores">
        <div class="dashboard-hero-metric dashboard-hero-metric-primary">
            <span class="dashboard-hero-label">
                <GreIcon name="score" size="sm" />
                Estimated GRE
            </span>
            <span class="dashboard-hero-value">{estimatedGreHero(estimate)}</span>
            {#if formatGreScoreRange(estimate.combinedScoreLow, estimate.combinedScoreHigh)}
                <span class="dashboard-hero-detail">
                    {formatGreScoreRange(
                        estimate.combinedScoreLow,
                        estimate.combinedScoreHigh,
                    )}
                </span>
            {/if}
        </div>

        <div class="dashboard-hero-divider" aria-hidden="true"></div>

        <div class="dashboard-hero-metric dashboard-hero-metric-readiness">
            <span class="dashboard-hero-label">
                <GreIcon name="readiness" size="sm" />
                Readiness score
            </span>
            <div class="dashboard-hero-readiness-visual">
                <GreProgressRing
                    value={coverageAwareReadinessUnlocked(readiness, coverage)
                        ? readiness.projectedScore!
                        : null}
                    size="lg"
                    label="Readiness score"
                    color="var(--state-review)"
                />
                {#if formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                    <span class="dashboard-hero-detail">
                        {formatRange(
                            readiness.projectedScoreLow,
                            readiness.projectedScoreHigh,
                        )}
                    </span>
                {/if}
            </div>
        </div>
    </div>

    <div class="dashboard-hero-predictions">
        <EstimatedGreSummary
            {estimate}
            {readiness}
            {memory}
            {performance}
            {weakTopics}
            {checklistRequirements}
            metricChange={metricChanges.estimatedGre ?? null}
            variant="compact"
        />
        <ReadinessSummary
            {readiness}
            {memory}
            {performance}
            {coverage}
            {weakTopics}
            metricChange={metricChanges.readiness ?? null}
            confidenceChange={metricChanges.confidence ?? null}
            variant="compact"
        />
    </div>
</section>

<style lang="scss">
    .dashboard-hero-predictions {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: var(--gre-space-4);
        padding-top: var(--gre-space-3);
        border-top: 1px solid color-mix(in srgb, var(--border) 45%, transparent);
    }

    .dashboard-hero-predictions :global(.gre-prediction-score-header) {
        display: none;
    }
</style>
