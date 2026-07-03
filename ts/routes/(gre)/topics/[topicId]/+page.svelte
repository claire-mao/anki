<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { PerformanceAttempt, Question } from "@generated/anki/brainlift_pb";

    import GrePageHeader from "../../GrePageHeader.svelte";
    import { buildTopicExplainability } from "../../prediction-explainability";
    import {
        readinessWhy,
        topicGlobalReadinessConfidence,
        topicGlobalReadinessEvidence,
        topicGlobalReadinessNextAction,
        topicGlobalReadinessScore,
    } from "../../prediction-presentation";
    import { emptyStateContent } from "../../empty-states";
    import {
        formatPercent,
        formatRange,
        formatResponseTimeMs,
    } from "../../score-format";
    import GreEmptyState from "../../ui/GreEmptyState.svelte";
    import GrePredictionBrief from "../../ui/GrePredictionBrief.svelte";
    import GreProgressBar from "../../ui/GreProgressBar.svelte";
    import GreTopicMasteryBar from "../../ui/GreTopicMasteryBar.svelte";
    import { greNavAction, greNavItem } from "../../gre-navigation";
    import GreButton from "../../ui/GreButton.svelte";
    import GreButtonRow from "../../ui/GreButtonRow.svelte";
    import type { PageData } from "./$types";

    import "../../gre.scss";
    import "./topic-details.scss";

    export let data: PageData;

    const details = data.details;
    const contribution = details.readinessContribution!;
    $: topicExplainability = buildTopicExplainability(details);

    function formatAnsweredAt(secs: bigint): string {
        return new Date(Number(secs) * 1000).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }

    function formatContribution(value: number | undefined): string {
        if (value === undefined) {
            return "—";
        }
        return `~${value.toFixed(1)} pts`;
    }

    function attemptSummary(attempt: PerformanceAttempt): string {
        const parts = [
            attempt.correct ? "Correct" : "Incorrect",
            formatResponseTimeMs(attempt.responseTimeMs),
            formatAnsweredAt(attempt.answeredAtSecs),
        ];
        return parts.join(" · ");
    }

    function questionPreview(question: Question): string {
        const stem = question.stem.trim();
        if (stem.length <= 120) {
            return stem;
        }
        return `${stem.slice(0, 117)}…`;
    }
</script>

<GrePageHeader title={details.displayName} icon="topic">
    <GreButtonRow className="topic-header-actions">
        <GreButton href="/study-plan">Study plan</GreButton>
        <GreButton navAction={greNavAction(greNavItem("practice"))}>Practice</GreButton>
    </GreButtonRow>
</GrePageHeader>

<ul class="topic-meta">
    <li>{details.section}</li>
    {#if details.isLeaf}
        <li>{formatPercent(details.examWeight * 100)} of section exam weight</li>
    {:else}
        <li>Parent topic</li>
    {/if}
    <li>{details.covered ? "Covered" : "Not covered"}</li>
</ul>

<div class="topic-grid gre-stagger">
    <section class="gre-panel">
        <h2>Mastery</h2>
        <div class="topic-mastery-visuals">
            {#if details.totalCards > 0}
                <GreProgressBar
                    label="Mastered cards"
                    value={details.masteredCards}
                    max={details.totalCards}
                    formatValue={(value) => `${value} / ${details.totalCards}`}
                />
            {/if}
            {#if details.memoryScore !== undefined}
                <GreTopicMasteryBar label="Memory score" value={details.memoryScore} />
            {/if}
        </div>
        <dl class="topic-stat-grid">
            <div>
                <dt>Studied cards</dt>
                <dd>{details.studiedCards}</dd>
            </div>
            <div>
                <dt>Reviews</dt>
                <dd>{details.totalReviews}</dd>
            </div>
            {#if formatRange(details.avgRetrievabilityLow, details.avgRetrievabilityHigh)}
                <div>
                    <dt>Retrievability range</dt>
                    <dd>
                        {formatRange(
                            details.avgRetrievabilityLow,
                            details.avgRetrievabilityHigh,
                        )}
                    </dd>
                </div>
            {/if}
            {#if details.memoryScore === undefined}
                <div class="topic-stat-empty">
                    <GreEmptyState
                        content={emptyStateContent("studiedCards")}
                        inline
                        compact
                        showChecklist={false}
                    />
                </div>
            {/if}
        </dl>
    </section>

    <section class="gre-panel">
        <h2>Performance</h2>
        {#if details.practiceAccuracy !== undefined}
            <GreTopicMasteryBar label="Accuracy" value={details.practiceAccuracy} />
        {/if}
        <dl class="topic-stat-grid">
            <div>
                <dt>Practice attempts</dt>
                <dd>{details.practiceTotal}</dd>
            </div>
            <div>
                <dt>Correct</dt>
                <dd>{details.practiceCorrect}</dd>
            </div>
            {#if details.practiceAccuracy === undefined}
                <div class="topic-stat-empty">
                    <GreEmptyState
                        content={emptyStateContent("practiceAttempts")}
                        inline
                        compact
                        showChecklist={false}
                    />
                </div>
            {/if}
        </dl>
    </section>

    <section class="gre-panel topic-grid-wide">
        <h2>Readiness contribution</h2>
        <GrePredictionBrief
            title="Global readiness"
            score={topicGlobalReadinessScore(
                details.globalReadinessScore,
                formatPercent,
            )}
            unlocked={details.globalReadinessScore !== undefined}
            confidence={topicGlobalReadinessConfidence(details.globalReadinessScore)}
            confidenceAsText={true}
            why={readinessWhy()}
            evidence={topicGlobalReadinessEvidence(
                details.globalReadinessSummary,
                contribution.estimatedTotalContribution,
            )}
            nextAction={topicGlobalReadinessNextAction(
                details.covered,
                details.practiceTotal,
            )}
            explainability={topicExplainability}
            detailRows={[
                {
                    label: "Coverage",
                    value: formatContribution(contribution.coverageContribution),
                },
                {
                    label: "Memory",
                    value: formatContribution(contribution.memoryContribution),
                },
                {
                    label: "Performance",
                    value: formatContribution(contribution.performanceContribution),
                },
                {
                    label: "Topic total",
                    value: formatContribution(contribution.estimatedTotalContribution),
                },
            ]}
            variant="inline"
            showScoreHeader={true}
            expandLabel="Inspect evidence"
        />
    </section>

    <section class="gre-panel">
        <h2>Practice questions</h2>
        {#if details.practiceQuestions.length > 0}
            <ul class="topic-list-plain">
                {#each details.practiceQuestions as question}
                    <li>
                        <strong>{questionPreview(question)}</strong>
                        <span class="muted">· {question.format}</span>
                    </li>
                {/each}
            </ul>
        {:else}
            <GreEmptyState content={emptyStateContent("topicQuestions")} />
        {/if}
    </section>

    <section class="gre-panel">
        <h2>Recent attempts</h2>
        {#if details.recentAttempts.length > 0}
            <ul class="topic-list-plain">
                {#each details.recentAttempts as attempt}
                    <li>
                        <span class="muted">{attemptSummary(attempt)}</span>
                    </li>
                {/each}
            </ul>
        {:else}
            <GreEmptyState content={emptyStateContent("topicAttempts")} />
        {/if}
    </section>
</div>
