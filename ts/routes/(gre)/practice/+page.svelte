<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { PerformanceScore, Question } from "@generated/anki/brainlift_pb";
    import { getScores, recordAttempt } from "@generated/backend";

    import {
        formatPercent,
        formatRange,
        formatResponseTimeMs,
        performanceSummary,
    } from "../score-format";
    import type { PageData } from "./$types";

    export let data: PageData;

    const sessionId = data.sessionId;
    const questions = data.questions;

    let performance: PerformanceScore = data.performance;
    let questionIndex = 0;
    let currentQuestion: Question = questions[0];
    let selected = "";
    let confidence: number | null = null;
    let startedAt = Date.now();
    let submitting = false;
    let sessionComplete = false;
    let result: {
        correct: boolean;
        explanation: string;
        topic: string;
        responseTimeMs: number;
    } | null = null;

    function resetQuestionState(): void {
        selected = "";
        confidence = null;
        startedAt = Date.now();
        result = null;
    }

    function nextQuestion(): void {
        resetQuestionState();
        const nextIndex = questionIndex + 1;
        if (nextIndex >= questions.length) {
            sessionComplete = true;
            return;
        }
        questionIndex = nextIndex;
        currentQuestion = questions[questionIndex];
    }

    async function submit(): Promise<void> {
        if (!selected || submitting || result) {
            return;
        }
        submitting = true;
        const responseTimeMs = Date.now() - startedAt;
        try {
            const response = await recordAttempt({
                questionId: currentQuestion.id,
                answer: selected,
                responseTimeMs,
                confidence: confidence ?? undefined,
                sessionId,
            });
            result = {
                correct: response.correct,
                explanation: response.explanation,
                topic: response.topic,
                responseTimeMs,
            };

            const scores = await getScores({});
            performance = scores.performance!;
        } finally {
            submitting = false;
        }
    }
</script>

<h1>Practice</h1>

<p class="muted practice-intro">
    GRE multiple-choice questions are stored in the performance database, separate from Anki
    flashcards and FSRS scheduling.
</p>

<div class="gre-panel performance-panel">
    <h2>Performance</h2>
    {#if performance.sufficientData && performance.value !== undefined}
        <div class="score-value">{formatPercent(performance.value)}</div>
        {#if formatRange(performance.valueLow, performance.valueHigh)}
            <p class="score-range">{formatRange(performance.valueLow, performance.valueHigh)}</p>
        {/if}
        <p class="muted">{performance.detail}</p>
    {:else}
        <p class="score-abstain">{performance.abstainReason}</p>
        <p class="muted">{performance.attemptCount} attempts · {performance.detail}</p>
    {/if}
</div>

{#if sessionComplete}
    <div class="gre-panel">
        <p>Session complete for the available question bank.</p>
        <p class="muted">Performance: {performanceSummary(performance)}</p>
        <a class="btn btn-primary primary-button" href="/dashboard">View dashboard</a>
    </div>
{:else}
    <div class="gre-panel">
        <p class="muted">
            {currentQuestion.section} · {currentQuestion.format} · {currentQuestion.topic}
        </p>
        <p class="question-stem">{currentQuestion.stem}</p>

        {#if result}
            <p class="result-heading">
                <strong>{result.correct ? "Correct" : "Incorrect"}</strong>
                · {formatResponseTimeMs(result.responseTimeMs)}
            </p>
            <p>{result.explanation}</p>
            <p class="muted">{result.topic}</p>
            <div class="practice-actions">
                <button class="btn btn-primary" on:click={nextQuestion}>Next question</button>
                <a class="btn" href="/dashboard">Dashboard</a>
            </div>
        {:else}
            <div class="choices">
                {#each currentQuestion.choices as choice}
                    <label class="choice">
                        <input type="radio" bind:group={selected} value={choice} />
                        {choice}
                    </label>
                {/each}
            </div>

            <p class="muted">Confidence (optional)</p>
            <div class="confidence">
                {#each [1, 2, 3, 4, 5] as level}
                    <button
                        class="btn btn-sm"
                        class:btn-primary={confidence === level}
                        on:click={() => (confidence = level)}
                    >
                        {level}
                    </button>
                {/each}
            </div>

            <button
                class="btn btn-primary primary-button"
                disabled={!selected || submitting}
                on:click={submit}
            >
                Submit answer
            </button>
        {/if}
    </div>
{/if}
