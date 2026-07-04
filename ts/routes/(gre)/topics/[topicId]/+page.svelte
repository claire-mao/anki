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
    import {
        greNavAction,
        greNavItem,
        GRE_CTA_PRACTICE,
        GRE_CTA_REVIEW,
        GRE_CTA_STUDY_PLAN,
        studyPlanNavAction,
    } from "../../gre-navigation";
    import { practicePathForTopic } from "../../topic-link";
    import GreButton from "../../ui/GreButton.svelte";
    import GreButtonRow from "../../ui/GreButtonRow.svelte";
    import type { PageData } from "./$types";

    import "../../gre.scss";
    import "./topic-details.scss";

    export let data: PageData;

    const details = data.details;
    const contribution = details.readinessContribution!;
    $: topicExplainability = buildTopicExplainability(details);

    $: hasFlashcards = details.totalCards > 0;
    $: hasPracticeQuestions = details.practiceQuestions.length > 0;
    $: flashcardStatus = flashcardStatusLine(details);
    $: primaryOutcome = primaryOutcomeCopy(details, hasFlashcards, hasPracticeQuestions);

    function flashcardStatusLine(d: typeof details): string {
        if (d.totalCards === 0) {
            return "No flashcards for this topic yet — start with practice or your study plan";
        }
        const cardWord = d.totalCards === 1 ? "flashcard" : "flashcards";
        let line = `${d.totalCards} ${cardWord} for this topic`;
        if (d.studiedCards > 0) {
            line += ` · ${d.studiedCards} reviewed`;
        }
        return line;
    }

    function primaryOutcomeCopy(
        d: typeof details,
        flashcards: boolean,
        practice: boolean,
    ): string {
        if (flashcards) {
            return `Opens flashcard review in your GRE deck. Cards for ${d.displayName} appear when they're due for review.`;
        }
        if (practice) {
            return `Answer exam-style questions for ${d.displayName}. Practice builds your Performance score for this topic.`;
        }
        return "GRE Atlas includes built-in flashcards across the catalog. Practice here now, or follow your study plan for the next focus area.";
    }

    function noFlashcardsExplanation(d: typeof details): string {
        if (d.covered) {
            return "You've studied related material, but this topic doesn't have dedicated flashcards yet. Practice questions are the best next step.";
        }
        return "Built-in flashcards cover core GRE topics as you study. Until this area appears in your deck, practice questions build skill here.";
    }

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

<GrePageHeader
    title={details.displayName}
    icon="topic"
    subtitle="How do I start learning this topic?"
/>

<section class="topic-learn gre-panel">
    <p class="topic-flashcard-status" class:topic-flashcard-status-empty={!hasFlashcards}>
        {flashcardStatus}
    </p>

    {#if !hasFlashcards}
        <p class="topic-learn-hint">{noFlashcardsExplanation(details)}</p>
    {/if}

    <GreButtonRow className="topic-learn-actions">
        {#if hasFlashcards}
            <GreButton
                variant="primary"
                size="lg"
                navAction={{
                    ...greNavAction(greNavItem("study")),
                    label: GRE_CTA_REVIEW,
                }}
            >
                {GRE_CTA_REVIEW}
            </GreButton>
            {#if hasPracticeQuestions || details.practiceTotal > 0}
                <GreButton
                    navAction={{
                        label: GRE_CTA_PRACTICE,
                        href: practicePathForTopic(details.topicId),
                    }}
                >
                    {GRE_CTA_PRACTICE}
                </GreButton>
            {/if}
        {:else if hasPracticeQuestions}
            <GreButton
                variant="primary"
                size="lg"
                navAction={{
                    label: GRE_CTA_PRACTICE,
                    href: practicePathForTopic(details.topicId),
                }}
            >
                {GRE_CTA_PRACTICE}
            </GreButton>
            <GreButton navAction={studyPlanNavAction()}>{GRE_CTA_STUDY_PLAN}</GreButton>
        {:else}
            <GreButton variant="primary" size="lg" navAction={studyPlanNavAction()}>
                {GRE_CTA_STUDY_PLAN}
            </GreButton>
            <GreButton navAction={greNavAction(greNavItem("practice"))}>
                {GRE_CTA_PRACTICE}
            </GreButton>
        {/if}
    </GreButtonRow>

    <p class="topic-learn-outcome">{primaryOutcome}</p>

    <ul class="topic-meta topic-meta-inline">
        <li>{details.section}</li>
        {#if details.isLeaf}
            <li>{formatPercent(details.examWeight * 100)} exam weight</li>
        {:else}
            <li>Parent topic</li>
        {/if}
        <li>
            {details.covered
                ? "Covered — you've reviewed flashcards for this topic"
                : "Not covered yet"}
        </li>
    </ul>
</section>

<details class="topic-measurement">
    <summary>Topic progress & measurement</summary>

    <div class="topic-grid gre-stagger">
        <section class="gre-panel">
            <h2>Memory</h2>
            {#if details.totalCards > 0 || details.memoryScore !== undefined}
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
                        <GreTopicMasteryBar label="Recall likelihood" value={details.memoryScore} />
                    {/if}
                </div>
                {#if details.totalReviews > 0 || formatRange(details.avgRetrievabilityLow, details.avgRetrievabilityHigh)}
                    <p class="topic-measurement-caption">
                        {#if details.totalReviews > 0}
                            {details.totalReviews} review{details.totalReviews === 1 ? "" : "s"}
                        {/if}
                        {#if formatRange(details.avgRetrievabilityLow, details.avgRetrievabilityHigh)}
                            {#if details.totalReviews > 0}
                                ·
                            {/if}
                            Recall range
                            {formatRange(
                                details.avgRetrievabilityLow,
                                details.avgRetrievabilityHigh,
                            )}
                        {/if}
                    </p>
                {/if}
            {:else}
                <GreEmptyState
                    content={emptyStateContent("studiedCards")}
                    inline
                    compact
                    showChecklist={false}
                />
            {/if}
        </section>

        <section class="gre-panel">
            <h2>Performance</h2>
            {#if details.practiceAccuracy !== undefined}
                <GreTopicMasteryBar label="Practice accuracy" value={details.practiceAccuracy} />
                <p class="topic-measurement-caption">
                    {details.practiceCorrect} correct of {details.practiceTotal} attempt{details.practiceTotal === 1
                        ? ""
                        : "s"}
                </p>
            {:else if details.practiceTotal > 0}
                <p class="topic-measurement-caption">
                    {details.practiceTotal} attempt{details.practiceTotal === 1 ? "" : "s"} — accuracy
                    builds with more answers
                </p>
            {:else}
                <GreEmptyState
                    content={emptyStateContent("practiceAttempts")}
                    inline
                    compact
                    showChecklist={false}
                />
            {/if}
        </section>

        <section class="gre-panel topic-grid-wide">
            <h2>How this topic affects your score</h2>
            <GrePredictionBrief
                title="Score estimate"
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
            <h2>Sample questions</h2>
            {#if details.practiceQuestions.length > 0}
                <p class="topic-measurement-caption">
                    Preview only — start a practice session to answer.
                </p>
                <ul class="topic-list-plain">
                    {#each details.practiceQuestions as question}
                        <li>
                            <span>{questionPreview(question)}</span>
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
</details>
