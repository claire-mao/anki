<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        DashboardCoverage,
        EstimatedGreScore,
        MemoryScore,
        PerformanceScore,
        ReadinessScore,
    } from "@generated/anki/brainlift_pb";

    import { buildEvidenceItems } from "../evidence-presentation";
    import {
        methodologyNavAction,
        readinessNavAction,
    } from "../gre-navigation";
    import {
        dashboardHeroEstimatedGre,
        dashboardHeroEstimatedGreAvailable,
        dashboardHeroEstimatedGreHint,
        dashboardHeroMetricsUnlocked,
    } from "../summary-metrics";
    import { formatGreScoreRange, formatRange } from "../score-format";
    import GreIcon from "../GreIcon.svelte";
    import GreInfoButton from "../ui/GreInfoButton.svelte";
    import GreProgressRing from "../ui/GreProgressRing.svelte";

    export let estimate: EstimatedGreScore;
    export let readiness: ReadinessScore;
    export let memory: MemoryScore;
    export let performance: PerformanceScore;
    export let coverage: DashboardCoverage;
</script>

<section class="dashboard-hero gre-animate-in" aria-label="Your GRE snapshot">
    <div class="dashboard-hero-scores">
        <div class="dashboard-hero-metric dashboard-hero-metric-estimate">
            <div class="dashboard-hero-label-row">
                <span class="dashboard-hero-label">
                    <GreIcon name="score" size="sm" />
                    Estimated GRE
                </span>
                <GreInfoButton
                    action={methodologyNavAction()}
                    label="How GRE Atlas estimates your score"
                />
            </div>
            <div class="dashboard-hero-visual">
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
            </div>
            <ul class="dashboard-hero-stats" aria-label="Evidence backing your estimate">
                {#each buildEvidenceItems(memory, performance, coverage).slice(0, 3) as item}
                    <li
                        class="dashboard-hero-stat"
                        class:dashboard-hero-stat-met={item.met}
                    >
                        <GreIcon name="check" size="sm" />
                        <span>{item.label}</span>
                    </li>
                {/each}
            </ul>
            {#if dashboardHeroEstimatedGreAvailable(estimate, readiness, coverage)}
                <span class="dashboard-hero-hint dashboard-hero-hint-compact">
                    {dashboardHeroEstimatedGreHint(estimate, readiness, coverage)}
                </span>
            {/if}
        </div>

        <div class="dashboard-hero-divider" aria-hidden="true"></div>

        <div class="dashboard-hero-metric dashboard-hero-metric-readiness">
            <div class="dashboard-hero-label-row">
                <span class="dashboard-hero-label">
                    <GreIcon name="readiness" size="sm" />
                    Readiness
                </span>
                <GreInfoButton
                    action={readinessNavAction()}
                    label="Readiness score details"
                />
            </div>
            <div class="dashboard-hero-visual">
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
