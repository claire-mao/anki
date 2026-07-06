<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { DashboardCoverage } from "@generated/anki/brainlift_pb";

    import {
        COVERAGE_BREAKDOWN_HEADERS,
        COVERAGE_EXPLANATION,
        COVERAGE_TOPIC_PERCENT_LABEL,
        COVERAGE_WEIGHTED_FORMULA_INTRO,
        COVERAGE_WEIGHTED_PERCENT_LABEL,
        type CoverageSummaryVariant,
        coverageShowsBreakdown,
        formatContributionPoints,
        formatCoverageTopicCount,
        presentCoverageSummary,
    } from "../coverage-presentation";
    import GreText from "./GreText.svelte";

    export let coverage: DashboardCoverage;
    export let variant: CoverageSummaryVariant = "dashboard";
    export let compact = false;
    export let showReadinessGate = true;

    $: summary = presentCoverageSummary(coverage);
    $: showBreakdown = coverageShowsBreakdown(variant, compact);
</script>

<section
    class="gre-coverage-summary"
    class:gre-coverage-summary-compact={compact}
    aria-label="GRE topic coverage"
>
    <div class="gre-coverage-summary-head">
        <div class="gre-coverage-summary-title-block">
            <GreText variant="h3" tag="h2" className="gre-coverage-summary-title">
                Topic coverage
            </GreText>
            {#if !compact}
                <p class="gre-coverage-summary-explainer">{COVERAGE_EXPLANATION}</p>
            {/if}
        </div>
        <div class="gre-coverage-summary-total-block">
            <span class="gre-coverage-summary-total-label">{COVERAGE_TOPIC_PERCENT_LABEL}</span>
            <span class="gre-coverage-summary-total" aria-label="Topics reviewed">
                {summary.topicPercent}%
            </span>
            <span class="gre-coverage-summary-topics">
                {formatCoverageTopicCount(summary.coveredLeafCount, summary.catalogLeafCount)}
            </span>
        </div>
    </div>

    {#if !compact}
        <div class="gre-coverage-summary-weighted">
            <div class="gre-coverage-summary-weighted-head">
                <span class="gre-coverage-summary-weighted-label">{COVERAGE_WEIGHTED_PERCENT_LABEL}</span>
                <span class="gre-coverage-summary-weighted-value">{summary.totalPercent}%</span>
            </div>
            {#if showBreakdown}
                <p class="gre-coverage-summary-formula">{COVERAGE_WEIGHTED_FORMULA_INTRO}</p>
                <div class="gre-coverage-summary-breakdown" role="table" aria-label="Readiness coverage calculation">
                    <div class="gre-coverage-summary-breakdown-row gre-coverage-summary-breakdown-head" role="row">
                        {#each COVERAGE_BREAKDOWN_HEADERS as header}
                            <span class="gre-coverage-summary-breakdown-cell" role="columnheader">
                                {header}
                            </span>
                        {/each}
                    </div>
                    {#each summary.breakdown as row (row.sectionSlug)}
                        <div class="gre-coverage-summary-breakdown-row" role="row">
                            <span class="gre-coverage-summary-breakdown-cell gre-coverage-summary-breakdown-section" role="cell">
                                {row.label}
                            </span>
                            <span class="gre-coverage-summary-breakdown-cell" role="cell">
                                {row.coveragePercent}%
                            </span>
                            <span class="gre-coverage-summary-breakdown-cell" role="cell">
                                {row.weightPercent}%
                            </span>
                            <span class="gre-coverage-summary-breakdown-cell gre-coverage-summary-breakdown-contribution" role="cell">
                                {formatContributionPoints(row.contributionPoints)}
                            </span>
                        </div>
                    {/each}
                    <div class="gre-coverage-summary-breakdown-row gre-coverage-summary-breakdown-total" role="row">
                        <span class="gre-coverage-summary-breakdown-cell gre-coverage-summary-breakdown-total-label" role="cell">
                            Total
                        </span>
                        <span class="gre-coverage-summary-breakdown-cell" role="cell" aria-hidden="true"></span>
                        <span class="gre-coverage-summary-breakdown-cell" role="cell" aria-hidden="true"></span>
                        <span class="gre-coverage-summary-breakdown-cell gre-coverage-summary-breakdown-contribution" role="cell">
                            {summary.totalPercent}%
                        </span>
                    </div>
                </div>
            {/if}
        </div>
    {/if}

    {#if showReadinessGate && !summary.readinessAvailable}
        <div class="gre-coverage-summary-gate" aria-live="polite">
            <p class="gre-coverage-summary-gate-title">Readiness needs more coverage</p>
            <p class="gre-coverage-summary-gate-reason">
                {summary.readinessReason}
                Need at least {summary.thresholdPercent}% readiness coverage.
            </p>
        </div>
    {/if}
</section>

<style lang="scss">
    .gre-coverage-summary {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
        padding: var(--gre-card-padding);
        border-radius: var(--gre-radius-lg);
        background: color-mix(in srgb, var(--canvas) 88%, var(--fg-link) 4%);
        box-shadow: var(--gre-shadow-sm);
    }

    .gre-coverage-summary-head {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: var(--gre-space-4);
    }

    .gre-coverage-summary-title-block {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        min-width: 0;
    }

    .gre-coverage-summary :global(.gre-coverage-summary-title) {
        margin: 0;
        color: var(--fg-subtle);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .gre-coverage-summary-explainer {
        margin: 0;
        max-width: 28rem;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-coverage-summary-total-block {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: var(--gre-space-1);
        flex-shrink: 0;
        text-align: right;
    }

    .gre-coverage-summary-total-label,
    .gre-coverage-summary-weighted-label {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .gre-coverage-summary-weighted {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
        padding: var(--gre-space-3);
        border-radius: var(--gre-radius-md);
        background: color-mix(in srgb, var(--canvas) 70%, var(--border) 8%);
    }

    .gre-coverage-summary-weighted-head {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: var(--gre-space-3);
    }

    .gre-coverage-summary-weighted-value {
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h2);
        line-height: var(--gre-lh-h2);
        color: var(--fg);
    }

    .gre-coverage-summary-formula {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-coverage-summary-breakdown {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-coverage-summary-breakdown-row {
        display: grid;
        grid-template-columns: minmax(0, 1.2fr) repeat(3, minmax(0, 1fr));
        gap: var(--gre-space-2);
        align-items: center;
        padding: var(--gre-space-2) var(--gre-space-3);
        border-radius: var(--gre-radius-sm);
    }

    .gre-coverage-summary-breakdown-head {
        padding-top: 0;
        padding-bottom: var(--gre-space-1);
    }

    .gre-coverage-summary-breakdown-head .gre-coverage-summary-breakdown-cell {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .gre-coverage-summary-breakdown-row:not(.gre-coverage-summary-breakdown-head):not(.gre-coverage-summary-breakdown-total) {
        background: color-mix(in srgb, var(--canvas) 82%, var(--border) 6%);
    }

    .gre-coverage-summary-breakdown-total {
        margin-top: var(--gre-space-1);
        border-top: 1px solid color-mix(in srgb, var(--border) 55%, transparent);
    }

    .gre-coverage-summary-breakdown-total-label {
        font-weight: var(--gre-weight-h3);
        color: var(--fg);
    }

    .gre-coverage-summary-breakdown-contribution {
        font-variant-numeric: tabular-nums;
        text-align: right;
    }

    .gre-coverage-summary-breakdown-cell:not(.gre-coverage-summary-breakdown-section):not(.gre-coverage-summary-breakdown-contribution) {
        text-align: right;
    }

    .gre-coverage-summary-breakdown-cell {
        min-width: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .gre-coverage-summary-breakdown-section {
        font-weight: var(--gre-weight-h3);
    }

    .gre-coverage-summary-total {
        font-size: var(--gre-font-display-sm);
        font-weight: var(--gre-weight-hero);
        line-height: var(--gre-lh-display);
        letter-spacing: var(--gre-tracking-display);
        color: var(--fg);
    }

    .gre-coverage-summary-topics {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-coverage-summary-gate {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        padding: var(--gre-space-3);
        border-radius: var(--gre-radius-md);
        border: 1px solid color-mix(in srgb, var(--state-review) 28%, var(--border));
        background: color-mix(in srgb, var(--state-review) 8%, var(--canvas));
    }

    .gre-coverage-summary-gate-title {
        margin: 0;
        font-size: var(--gre-font-h3);
        font-weight: var(--gre-weight-h3);
        line-height: var(--gre-lh-h3);
        color: var(--fg);
    }

    .gre-coverage-summary-gate-reason {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg-subtle);
    }

    .gre-coverage-summary-compact {
        padding: var(--gre-space-3);
        gap: var(--gre-space-3);
        box-shadow: none;
        background: transparent;
    }

    .gre-coverage-summary-compact .gre-coverage-summary-head {
        align-items: center;
    }

    .gre-coverage-summary-compact :global(.gre-coverage-summary-title) {
        font-size: var(--gre-font-caption);
    }

    .gre-coverage-summary-compact .gre-coverage-summary-total {
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h2);
        line-height: var(--gre-lh-h2);
        letter-spacing: normal;
    }

    .gre-coverage-summary-compact .gre-coverage-summary-topics {
        display: none;
    }

    @media (max-width: 40rem) {
        .gre-coverage-summary-head {
            flex-direction: column;
            align-items: stretch;
        }

        .gre-coverage-summary-total-block {
            align-items: flex-start;
            text-align: left;
        }

        .gre-coverage-summary-breakdown-row {
            grid-template-columns: 1fr 1fr;
        }

        .gre-coverage-summary-breakdown-head {
            display: none;
        }

        .gre-coverage-summary-breakdown-row:not(.gre-coverage-summary-breakdown-total) {
            grid-template-areas:
                "section section"
                "coverage weight"
                "contribution contribution";
        }

        .gre-coverage-summary-breakdown-section {
            grid-area: section;
        }

        .gre-coverage-summary-breakdown-row:not(.gre-coverage-summary-breakdown-total) .gre-coverage-summary-breakdown-cell:nth-child(2) {
            grid-area: coverage;
        }

        .gre-coverage-summary-breakdown-row:not(.gre-coverage-summary-breakdown-total) .gre-coverage-summary-breakdown-cell:nth-child(3) {
            grid-area: weight;
        }

        .gre-coverage-summary-breakdown-row:not(.gre-coverage-summary-breakdown-total) .gre-coverage-summary-breakdown-contribution {
            grid-area: contribution;
        }
    }
</style>
