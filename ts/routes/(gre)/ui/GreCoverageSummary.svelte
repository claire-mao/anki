<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { DashboardCoverage } from "@generated/anki/brainlift_pb";

    import { presentCoverageSummary } from "../coverage-presentation";
    import GreText from "./GreText.svelte";

    export let coverage: DashboardCoverage;
    export let compact = false;
    export let showReadinessGate = true;
    export let recommendationLimit = 3;

    $: summary = presentCoverageSummary(coverage);
    $: recommendations = summary.recommendations.slice(0, recommendationLimit);
</script>

<section
    class="gre-coverage-summary"
    class:gre-coverage-summary-compact={compact}
    aria-label="GRE topic coverage"
>
    <div class="gre-coverage-summary-head">
        <GreText variant="h3" tag="h2" className="gre-coverage-summary-title">
            Coverage
        </GreText>
        <span class="gre-coverage-summary-total" aria-label="Total weighted coverage">
            {summary.totalPercent}%
        </span>
    </div>

    <div
        class="gre-coverage-summary-sections"
        role="list"
        aria-label="Section coverage"
    >
        {#each summary.sections as section (section.label)}
            <div class="gre-coverage-summary-section" role="listitem">
                <span class="gre-coverage-summary-section-label">{section.label}</span>
                <span class="gre-coverage-summary-section-value">
                    {section.percent}%
                </span>
            </div>
        {/each}
    </div>

    {#if showReadinessGate && !summary.readinessAvailable}
        <div class="gre-coverage-summary-gate" aria-live="polite">
            <p class="gre-coverage-summary-gate-title">Readiness unavailable</p>
            <p class="gre-coverage-summary-gate-reason">
                <span class="gre-coverage-summary-gate-label">Reason:</span>
                {summary.readinessReason}
            </p>
            {#if recommendations.length > 0}
                <div class="gre-coverage-summary-recommendations">
                    <p class="gre-coverage-summary-gate-label">Recommendations:</p>
                    <ul>
                        {#each recommendations as recommendation}
                            <li>{recommendation}</li>
                        {/each}
                    </ul>
                </div>
            {/if}
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
        align-items: baseline;
        justify-content: space-between;
        gap: var(--gre-space-3);
    }

    .gre-coverage-summary :global(.gre-coverage-summary-title) {
        margin: 0;
        color: var(--fg-subtle);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .gre-coverage-summary-total {
        font-size: var(--gre-font-display-sm);
        font-weight: var(--gre-weight-hero);
        line-height: var(--gre-lh-display);
        letter-spacing: var(--gre-tracking-display);
        color: var(--fg);
    }

    .gre-coverage-summary-sections {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: var(--gre-space-3);
    }

    .gre-coverage-summary-section {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
        min-width: 0;
        padding: var(--gre-space-3);
        border-radius: var(--gre-radius-md);
        background: color-mix(in srgb, var(--canvas) 70%, var(--border) 8%);
    }

    .gre-coverage-summary-section-label {
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-coverage-summary-section-value {
        font-size: var(--gre-font-h2);
        font-weight: var(--gre-weight-h2);
        line-height: var(--gre-lh-h2);
        color: var(--fg);
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

    .gre-coverage-summary-gate-label {
        font-weight: var(--gre-weight-h3);
        color: var(--fg);
    }

    .gre-coverage-summary-recommendations {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-coverage-summary-recommendations ul {
        margin: 0;
        padding-left: 1.25rem;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .gre-coverage-summary-compact {
        padding: var(--gre-space-3);
        gap: var(--gre-space-3);
        box-shadow: none;
        background: transparent;
    }

    @media (max-width: 40rem) {
        .gre-coverage-summary-sections {
            grid-template-columns: 1fr;
        }
    }
</style>
