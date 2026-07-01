<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { PageData } from "./$types";

    export let data: PageData;

    function formatScore(value: number | undefined): string {
        if (value === undefined) {
            return "—";
        }
        return `${Math.round(value)}%`;
    }

    const memory = data.scores.memory!;
    const performance = data.scores.performance!;
    const readiness = data.scores.readiness!;
</script>

<h1>Readiness</h1>

<div class="score-grid">
    <div class="score-card">
        <h2>Memory</h2>
        <div class="score-value">{formatScore(memory.value)}</div>
        <p class="muted">{memory.detail}</p>
    </div>
    <div class="score-card">
        <h2>Performance</h2>
        <div class="score-value">{formatScore(performance.value)}</div>
        <p class="muted">{performance.detail}</p>
    </div>
    <div class="score-card">
        <h2>Readiness</h2>
        <div class="score-value">{formatScore(readiness.value)}</div>
        <p class="muted">{readiness.detail}</p>
    </div>
</div>

<div class="brainlift-panel">
    <h2>Recent practice</h2>
    {#if data.recent.attempts.length === 0}
        <p class="muted">No practice attempts yet. Finish review, then try Practice.</p>
    {:else}
        <ul>
            {#each data.recent.attempts as attempt}
                <li>
                    {attempt.questionId} · {attempt.correct ? "correct" : "incorrect"} ·
                    {(attempt.responseTimeMs / 1000).toFixed(1)}s
                </li>
            {/each}
        </ul>
    {/if}
</div>
