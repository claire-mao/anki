<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        DashboardCoverage,
        EstimatedGreScore,
        ReadinessScore,
    } from "@generated/anki/brainlift_pb";

    import {
        dashboardHeroEstimatedGre,
        dashboardHeroEstimatedGreAvailable,
        dashboardHeroEstimatedGreHint,
        dashboardHeroMetricsUnlocked,
    } from "../summary-metrics";
    import { formatGreScoreRange, formatRange } from "../score-format";
    import GreIcon from "../GreIcon.svelte";
    import GreProgressRing from "../ui/GreProgressRing.svelte";

    export let estimate: EstimatedGreScore;
    export let readiness: ReadinessScore;
    export let coverage: DashboardCoverage | undefined = undefined;
</script>

<section class="dashboard-hero gre-animate-in" aria-label="Your GRE snapshot">
    <div class="dashboard-hero-scores">
        <div class="dashboard-hero-metric dashboard-hero-metric-primary">
            <span class="dashboard-hero-label">
                <GreIcon name="score" size="sm" />
                Estimated GRE
            </span>
            <span class="dashboard-hero-value">
                {dashboardHeroEstimatedGre(estimate, readiness, coverage)}
            </span>
            {#if dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage) && formatGreScoreRange(estimate.combinedScoreLow, estimate.combinedScoreHigh)}
                <span class="dashboard-hero-detail">
                    {formatGreScoreRange(
                        estimate.combinedScoreLow,
                        estimate.combinedScoreHigh,
                    )}
                </span>
            {/if}
            <span class="dashboard-hero-hint">
                {dashboardHeroEstimatedGreHint(estimate, readiness, coverage)}
            </span>
        </div>

        <div class="dashboard-hero-divider" aria-hidden="true"></div>

        <div class="dashboard-hero-metric dashboard-hero-metric-readiness">
            <span class="dashboard-hero-label">
                <GreIcon name="readiness" size="sm" />
                Readiness
            </span>
            <div class="dashboard-hero-readiness-visual">
                <GreProgressRing
                    value={dashboardHeroMetricsUnlocked(readiness, coverage)
                        ? readiness.projectedScore!
                        : null}
                    size="lg"
                    label="Readiness"
                    color="var(--state-review)"
                />
                {#if dashboardHeroMetricsUnlocked(readiness, coverage) && formatRange(readiness.projectedScoreLow, readiness.projectedScoreHigh)}
                    <span class="dashboard-hero-detail">
                        {formatRange(
                            readiness.projectedScoreLow,
                            readiness.projectedScoreHigh,
                        )}
                    </span>
                {/if}
            </div>
            <span class="dashboard-hero-hint">
                How much your study evidence supports that score.
            </span>
        </div>
    </div>
</section>

<style lang="scss">
    .dashboard-hero-hint {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
        max-width: 16rem;
    }
</style>
