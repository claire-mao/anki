<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { StudyPlanRecommendation } from "@generated/anki/brainlift_pb";

    import { formatPercent, formatRatio } from "../score-format";
    import type { PageData } from "./$types";

    export let data: PageData;

    const plan = data.plan;
    const coverage = plan.coverage!;

    const factorLabels: Record<string, string> = {
        coverage_gap: "Coverage gap",
        low_mastery: "Low mastery",
        low_performance: "Low performance",
        no_practice: "No practice",
        high_importance: "High importance",
    };

    function factorLabel(factor: string): string {
        return factorLabels[factor] ?? factor;
    }

    function topicMeta(topic: StudyPlanRecommendation): string {
        const parts = [topic.section, `${formatRatio(topic.examWeight)} exam weight`];
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

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString();
    }
</script>

<h1>Study Plan</h1>

<p class="muted study-plan-summary">{plan.summary}</p>
<p class="muted dashboard-updated">
    Last updated {formatTimestampMillis(plan.computedAtMillis)}
</p>

<div class="gre-panel">
    <h2>Coverage</h2>
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
    <h2>Recommended topics</h2>
    {#if plan.recommendations.length === 0}
        <p class="muted">{plan.summary}</p>
    {:else}
        <ol class="study-plan-list">
            {#each plan.recommendations as topic, index}
                <li>
                    <div class="study-plan-rank">{index + 1}</div>
                    <div class="study-plan-body">
                        <strong>{topic.displayName}</strong>
                        <span class="muted">{topicMeta(topic)}</span>
                        <p>{topic.explanation}</p>
                        <div class="factor-tags">
                            {#each topic.factors as factor}
                                <span class="factor-tag">{factorLabel(factor)}</span>
                            {/each}
                        </div>
                    </div>
                </li>
            {/each}
        </ol>
    {/if}
</div>

<div class="gre-panel study-plan-actions">
    <a class="btn btn-primary" href="/review">GRE review</a>
    <a class="btn" href="/practice">Practice</a>
    <a class="btn" href="/dashboard">Dashboard</a>
</div>
