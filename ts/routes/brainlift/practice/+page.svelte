<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { recordAttempt } from "@generated/backend";

    import type { PageData } from "./$types";

    export let data: PageData;

    const question = data.question;
    let selected = "";
    let confidence: number | null = null;
    const startedAt = Date.now();
    let submitting = false;
    let result: {
        correct: boolean;
        explanation: string;
        topic: string;
    } | null = null;

    async function submit(): Promise<void> {
        if (!selected || submitting) {
            return;
        }
        submitting = true;
        try {
            const response = await recordAttempt({
                questionId: question.id,
                answer: selected,
                responseTimeMs: Date.now() - startedAt,
                confidence: confidence ?? undefined,
            });
            result = {
                correct: response.correct,
                explanation: response.explanation,
                topic: response.topic,
            };
        } finally {
            submitting = false;
        }
    }
</script>

<h1>Practice</h1>

<div class="brainlift-panel">
    <p class="muted">{question.section} · {question.format} · {question.topic}</p>
    <p>{question.stem}</p>

    {#if result}
        <p><strong>{result.correct ? "Correct" : "Incorrect"}</strong></p>
        <p>{result.explanation}</p>
        <p class="muted">Stored in brainlift.db (not FSRS / revlog).</p>
        <a class="btn btn-primary primary-button" href="/brainlift/readiness">
            View readiness
        </a>
    {:else}
        <div class="choices">
            {#each question.choices as choice}
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

<style lang="scss">
    .choices {
        display: grid;
        gap: 0.5rem;
        margin: 1rem 0;
    }

    .choice {
        display: flex;
        gap: 0.5rem;
        align-items: flex-start;
    }

    .confidence {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
    }
</style>
