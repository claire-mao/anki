<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import type { StudyFeatureExperimentPresentation } from "./eval-report-presentation";

    export let model: StudyFeatureExperimentPresentation;
</script>

<section class="evidence-section" aria-labelledby="evidence-study-feature-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-study-feature-heading">
            Study Feature Experiment
        </h2>
        <p class="evidence-section-lead">
            Topic-priority ablation compares GRE Atlas focus ordering against random and
            vanilla Anki ordering at equal study time.
        </p>
    </header>

    {#if !model.available}
        <p class="evidence-section-empty">{model.emptyMessage}</p>
    {:else}
        <div class="evidence-metrics">
            <GreMetricRow label="Focus topics per session" value={model.focusTopicCount} />
            <GreMetricRow label="Scenario" value={model.scenarioLabel} />
        </div>

        {#if model.assessment}
            <p class="evidence-section-assessment">{model.assessment}</p>
        {/if}

        <div class="evidence-metrics">
            {#each model.policyResults as result (result.label)}
                <GreMetricRow label={result.label} value={result.value} />
            {/each}
        </div>

        {#if model.winners.length > 0}
            <div class="evidence-winners">
                <h3 class="evidence-subheading">Winners</h3>
                <div class="evidence-metrics">
                    {#each model.winners as winner (winner.label)}
                        <GreMetricRow label={winner.label} value={winner.value} />
                    {/each}
                </div>
            </div>
        {/if}
    {/if}
</section>
