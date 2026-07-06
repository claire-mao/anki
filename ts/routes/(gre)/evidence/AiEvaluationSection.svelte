<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        AI_EVAL_INSUFFICIENT_MESSAGE,
        AI_EVAL_OFFLINE_MESSAGE,
        type AiEvaluationPresentation,
    } from "./ai-evaluation-presentation";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import GreProgressBar from "../ui/GreProgressBar.svelte";
    import GreText from "../ui/GreText.svelte";

    export let model: AiEvaluationPresentation;
</script>

<section class="evidence-section" aria-labelledby="evidence-ai-eval-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-ai-eval-heading">AI Evaluation</h2>
        <p class="evidence-section-lead">
            Held-out gold-set release gate and retrieval baseline comparison.
        </p>
    </header>

    {#if model.mode === "offline"}
        <GreText variant="body" className="evidence-ai-eval-message">
            {AI_EVAL_OFFLINE_MESSAGE}
        </GreText>
    {:else if model.mode === "insufficient"}
        <p class="evidence-section-empty">{AI_EVAL_INSUFFICIENT_MESSAGE}</p>
    {:else}
        <div class="evidence-metrics">
            <GreMetricRow label="Held-out questions" value={model.heldOutQuestions} />
            <GreMetricRow label="Accuracy" value={model.accuracy} />
            <GreMetricRow label="Wrong-answer rate" value={model.wrongAnswerRate} />
            <GreMetricRow label="Confidence threshold" value={model.confidenceThreshold} />
            <GreMetricRow label="Release cutoff" value={model.releaseCutoff} />
            <GreMetricRow label="Release verdict" value={model.verdict} />
        </div>

        <div class="evidence-ai-eval-baselines">
            <h3 class="evidence-subheading">Baseline Comparison</h3>
            <div class="evidence-ai-eval-bars">
                {#each model.baselines as baseline (baseline.label)}
                    <GreProgressBar
                        label={baseline.label}
                        value={baseline.accuracyPercent}
                        max={100}
                    />
                {/each}
            </div>
        </div>
    {/if}
</section>
