<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type {
        MemoryScore,
        PerformanceScore,
        Question,
    } from "@generated/anki/brainlift_pb";
    import { getScores, recordAttempt } from "@generated/backend";
    import { fade, fly } from "svelte/transition";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { emptyStateContent, emptyStateTitle } from "../empty-states";
    import PracticeScoreStrip from "./PracticeScoreStrip.svelte";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import GreChip from "../ui/GreChip.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreText from "../ui/GreText.svelte";
    import { formatResponseTimeMs } from "../score-format";
    import type { PageData } from "./$types";
    import {
        buildQuestionQueue,
        formatSectionLabel,
        type PracticeSectionFilter,
    } from "./practice-session";

    import "../gre.scss";
    import "./practice.scss";

    export let data: PageData;

    const sessionId = data.sessionId;
    const questionBank = data.questions;

    let memory: MemoryScore = data.memory;
    let performance: PerformanceScore = data.performance;
    let sectionFilter: PracticeSectionFilter = "all";
    let queue: Question[] = data.queue;
    let questionIndex = 0;
    let attemptsRecorded = 0;
    let selected = "";
    let startedAt = Date.now();
    let submitting = false;
    let submitError = "";
    const submitErrorId = "practice-submit-error";
    let sessionComplete = false;
    let result: {
        correct: boolean;
        explanation: string;
        topic: string;
        responseTimeMs: number;
    } | null = null;

    $: currentQuestion = queue[questionIndex];

    function progressPercentValue(): number {
        if (queue.length === 0) {
            return 0;
        }
        if (sessionComplete) {
            return 100;
        }
        return Math.round(((questionIndex + 1) / queue.length) * 100);
    }

    function progressLabelValue(): string {
        if (queue.length === 0) {
            return emptyStateTitle("noQuestionsFilter");
        }
        if (sessionComplete) {
            return "Session complete";
        }
        return `Question ${questionIndex + 1} of ${queue.length}`;
    }

    $: progressPercent = progressPercentValue();
    $: progressLabel = progressLabelValue();

    function resetQuestionState(): void {
        selected = "";
        startedAt = Date.now();
        result = null;
        submitError = "";
    }

    function applySectionFilter(section: PracticeSectionFilter): void {
        sectionFilter = section;
        queue = buildQuestionQueue(questionBank, section);
        questionIndex = 0;
        sessionComplete = queue.length === 0;
        resetQuestionState();
    }

    function nextQuestion(): void {
        resetQuestionState();
        const nextIndex = questionIndex + 1;
        if (nextIndex >= queue.length) {
            sessionComplete = true;
            return;
        }
        questionIndex = nextIndex;
    }

    async function submit(): Promise<void> {
        if (!selected || submitting || result || !currentQuestion) {
            return;
        }
        submitting = true;
        submitError = "";
        const responseTimeMs = Date.now() - startedAt;
        try {
            const response = await recordAttempt({
                questionId: currentQuestion.id,
                answer: selected,
                responseTimeMs,
                sessionId,
            });
            attemptsRecorded += 1;
            result = {
                correct: response.correct,
                explanation: response.explanation,
                topic: response.topic,
                responseTimeMs,
            };

            const scores = await getScores({});
            memory = scores.memory!;
            performance = scores.performance!;
        } catch {
            submitError = "Could not record this attempt. Please try again.";
        } finally {
            submitting = false;
        }
    }

    const sectionFilters: PracticeSectionFilter[] = ["all", "quant", "verbal", "awa"];
</script>

<div class="practice-page">
    <GrePageHeader title="Practice" icon="practice">
        <div class="practice-toolbar">
            {#if queue.length > 0}
                <div
                    class="practice-progress"
                    role="progressbar"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow={progressPercent}
                    aria-label={progressLabel}
                >
                    <div class="practice-progress-row">
                        <span class="practice-progress-label">{progressLabel}</span>
                        <span class="practice-progress-count">{progressPercent}%</span>
                    </div>
                    <div class="practice-progress-track">
                        <div
                            class="practice-progress-fill"
                            style:width="{progressPercent}%"
                        ></div>
                    </div>
                </div>
            {/if}
            <div class="practice-filters" role="group" aria-label="Section filter">
                {#each sectionFilters as section}
                    <GreChip
                        active={sectionFilter === section}
                        on:click={() => applySectionFilter(section)}
                    >
                        {formatSectionLabel(section)}
                    </GreChip>
                {/each}
            </div>
        </div>
    </GrePageHeader>

    {#if !sessionComplete}
        <PracticeScoreStrip {memory} {performance} />
    {/if}

    {#if sessionComplete}
        <GrePanel interactive={false} className="practice-complete">
            {#if attemptsRecorded === 0}
                <GreEmptyState
                    content={emptyStateContent("noQuestionsFilter")}
                    showChecklist={false}
                    on:action={() => applySectionFilter("all")}
                />
            {:else}
                <GreText variant="body">
                    You finished {attemptsRecorded} question{attemptsRecorded === 1
                        ? ""
                        : "s"} in this session.
                </GreText>
            {/if}
            <GreButtonRow className="practice-actions">
                <GreButton
                    variant="primary"
                    on:click={() => applySectionFilter(sectionFilter)}
                >
                    Practice again
                </GreButton>
            </GreButtonRow>
        </GrePanel>
    {:else if currentQuestion}
        {#key currentQuestion.id}
            <div
                class="practice-question-wrap"
                in:fly={{ y: 8, duration: 180 }}
                out:fade={{ duration: 120 }}
            >
                <section class="practice-question-stage">
                    <p class="question-meta">
                        {currentQuestion.section} · {currentQuestion.format}
                    </p>

                    <GreText variant="body" tag="p" className="question-stem">
                        {currentQuestion.stem}
                    </GreText>

                    {#if result}
                        <div
                            class="result-panel"
                            class:correct={result.correct}
                            class:incorrect={!result.correct}
                        >
                            <p class="result-heading">
                                <strong class="result-status">
                                    {result.correct ? "✓ Correct" : "✗ Incorrect"}
                                </strong>
                                · {formatResponseTimeMs(result.responseTimeMs)}
                            </p>
                            <p class="result-explanation">{result.explanation}</p>
                            <p class="result-topic">{result.topic}</p>
                        </div>

                        <GreButton
                            variant="primary"
                            size="lg"
                            className="practice-continue"
                            on:click={nextQuestion}
                        >
                            Continue
                        </GreButton>
                    {:else}
                        <div
                            class="choices"
                            role="radiogroup"
                            aria-label="Answer choices"
                        >
                            {#each currentQuestion.choices as choice}
                                <label
                                    class="choice"
                                    class:choice-selected={selected === choice}
                                >
                                    <input
                                        type="radio"
                                        bind:group={selected}
                                        value={choice}
                                    />
                                    <span class="choice-text">{choice}</span>
                                </label>
                            {/each}
                        </div>

                        <GreButton
                            variant="primary"
                            size="lg"
                            className="practice-submit"
                            loading={submitting}
                            disabled={!selected}
                            ariaDescribedby={submitError ? submitErrorId : undefined}
                            on:click={submit}
                        >
                            {submitting ? "Saving attempt…" : "Submit answer"}
                        </GreButton>
                        {#if submitError}
                            <p class="practice-error" id={submitErrorId} role="alert">
                                {submitError}
                            </p>
                        {/if}
                    {/if}
                </section>
            </div>
        {/key}
    {:else}
        <GreEmptyState
            content={emptyStateContent("noQuestionsFilter")}
            showChecklist={false}
            on:action={() => applySectionFilter("all")}
        />
    {/if}
</div>
