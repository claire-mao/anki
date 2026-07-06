<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import {
        presentStudyFeatureExperiment,
        STUDY_FEATURE_EXPERIMENT_PAGE_SUBTITLE,
        STUDY_FEATURE_EXPERIMENT_PAGE_TITLE,
    } from "./study-feature-experiment-presentation";
    import type { PageData } from "./$types";

    import "../gre.scss";
    import "./study-feature-experiment.scss";

    export let data: PageData;

    $: model = presentStudyFeatureExperiment(data.evalReportJson);
</script>

<GrePageHeader
    title={STUDY_FEATURE_EXPERIMENT_PAGE_TITLE}
    icon="study"
    subtitle={STUDY_FEATURE_EXPERIMENT_PAGE_SUBTITLE}
/>

<GreSection>
    <div class="study-feature-experiment-page">
        <section class="study-feature-experiment-card" aria-labelledby="study-feature-design">
            <header class="study-feature-experiment-header">
                <h2 class="gre-section-title" id="study-feature-design">Experiment design</h2>
                <p class="study-feature-experiment-lead">
                    Pre-registered protocol for one study feature ablation with three builds.
                </p>
            </header>

            <div class="study-feature-experiment-metrics">
                <GreMetricRow label="Feature" value={model.design.feature} />
                <GreMetricRow label="Hypothesis" value={model.design.hypothesis} />
                <GreMetricRow label="Equal study time" value={model.design.equalStudyTime} />
                <GreMetricRow label="Evaluation metric" value={model.design.evaluationMetric} />
            </div>

            <div class="study-feature-experiment-versions">
                <h3 class="study-feature-experiment-subheading">Versions tested</h3>
                <ul class="study-feature-experiment-version-list">
                    {#each model.design.versions as version (version.label)}
                        <li class="study-feature-experiment-version">
                            <span class="study-feature-experiment-version-label">{version.label}</span>
                            <span class="study-feature-experiment-version-description">
                                {version.description}
                            </span>
                        </li>
                    {/each}
                </ul>
            </div>
        </section>

        <section class="study-feature-experiment-card" aria-labelledby="study-feature-results">
            <header class="study-feature-experiment-header">
                <h2 class="gre-section-title" id="study-feature-results">Results</h2>
            </header>

            {#if model.resultsAvailable}
                <div class="study-feature-experiment-metrics">
                    {#each model.results as result (result.label)}
                        <GreMetricRow label={result.label} value={result.value} />
                    {/each}
                </div>
            {:else}
                <p class="study-feature-experiment-empty">{model.resultsMessage}</p>
            {/if}
        </section>

        <section class="study-feature-experiment-card" aria-labelledby="study-feature-conclusion">
            <header class="study-feature-experiment-header">
                <h2 class="gre-section-title" id="study-feature-conclusion">Conclusion</h2>
            </header>

            {#if model.resultsAvailable}
                <p class="study-feature-experiment-conclusion">{model.conclusion}</p>
            {:else}
                <p class="study-feature-experiment-empty">{model.conclusion}</p>
            {/if}
        </section>
    </div>
</GreSection>
