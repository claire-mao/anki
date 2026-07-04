<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        DashboardCoverage,
        MemoryScore,
        PerformanceScore,
        ReadinessCalibrationStats,
    } from "@generated/anki/brainlift_pb";

    import { COVERAGE_EXPLANATION } from "../coverage-presentation";
    import {
        buildEvidenceItems,
        buildEvidenceSummary,
    } from "../evidence-presentation";
    import { runGreNavAction } from "../gre-navigation";
    import { METHODOLOGY_PAGE_TITLE } from "../methodology-presentation";
    import GreIcon from "../GreIcon.svelte";

    export let memory: MemoryScore;
    export let performance: PerformanceScore;
    export let coverage: DashboardCoverage;
    export let calibration: ReadinessCalibrationStats | undefined = undefined;
    export let computedAtMillis: bigint;

    $: items = buildEvidenceItems(memory, performance, coverage, calibration);
    $: summary = buildEvidenceSummary(memory, performance, coverage);

    function relativeTime(millis: bigint): string {
        const deltaSec = Math.round((Date.now() - Number(millis)) / 1000);
        const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
        const units: [Intl.RelativeTimeFormatUnit, number][] = [
            ["day", 86400],
            ["hour", 3600],
            ["minute", 60],
        ];
        for (const [unit, secs] of units) {
            if (Math.abs(deltaSec) >= secs) {
                return formatter.format(-Math.round(deltaSec / secs), unit);
            }
        }
        return formatter.format(-deltaSec, "second");
    }

    function learnMore(): void {
        runGreNavAction({
            label: METHODOLOGY_PAGE_TITLE,
            bridge: "greOpenMethodology",
            href: "/methodology",
        });
    }
</script>

<details class="gre-evidence-card">
    <summary class="gre-evidence-summary">
        <span class="gre-evidence-summary-text">
            <GreIcon name="info" size="sm" />
            {summary}
        </span>
        <span class="gre-evidence-updated">Updated {relativeTime(computedAtMillis)}</span>
    </summary>

    <div class="gre-evidence-body">
        <ul class="gre-evidence-list">
            {#each items as item}
                <li class="gre-evidence-item" class:gre-evidence-item-met={item.met}>
                    <GreIcon name="check" size="sm" />
                    <span>{item.label}</span>
                </li>
            {/each}
        </ul>
        <p class="gre-evidence-coverage-note">{COVERAGE_EXPLANATION}</p>
        <button type="button" class="gre-evidence-link" on:click={learnMore}>
            {METHODOLOGY_PAGE_TITLE}
        </button>
    </div>
</details>

<style lang="scss">
    .gre-evidence-card {
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg-muted);
        box-shadow: var(--gre-shadow-sm);
    }

    .gre-evidence-summary {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        justify-content: space-between;
        gap: var(--gre-space-2) var(--gre-space-3);
        padding: var(--gre-space-3) var(--gre-space-4);
        cursor: pointer;
        list-style: none;
    }

    .gre-evidence-summary::-webkit-details-marker {
        display: none;
    }

    .gre-evidence-summary-text {
        display: inline-flex;
        align-items: center;
        gap: var(--gre-space-2);
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-evidence-updated {
        font-size: var(--gre-font-caption);
        color: var(--fg-subtle);
    }

    .gre-evidence-body {
        padding: 0 var(--gre-space-4) var(--gre-space-3);
        border-top: 1px solid color-mix(in srgb, var(--border) 40%, transparent);
    }

    .gre-evidence-list {
        list-style: none;
        margin: var(--gre-space-3) 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .gre-evidence-item {
        display: flex;
        align-items: center;
        gap: var(--gre-space-2);
        font-size: var(--gre-font-caption);
        color: var(--fg-subtle);
    }

    .gre-evidence-item-met {
        color: var(--fg);
    }

    .gre-evidence-item-met :global(.gre-icon) {
        color: var(--gre-success);
    }

    .gre-evidence-coverage-note {
        margin: 0 0 var(--gre-space-3);
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-evidence-link {
        padding: 0;
        border: none;
        background: none;
        cursor: pointer;
        text-align: left;
        font-size: var(--gre-font-caption);
        color: var(--fg-link);
    }

    .gre-evidence-link:hover {
        text-decoration: underline;
    }
</style>
