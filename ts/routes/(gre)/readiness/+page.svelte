<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import { presentReadinessPage } from "../readiness-page-presentation";
    import ReadinessEstimatePanel from "./ReadinessEstimatePanel.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    const response = data.response;
    const readiness = response.readiness!;
    const calibration = response.calibration!;
    const memory = data.scores.memory!;
    const performance = data.scores.performance!;
    const estimatedGre = data.scores.estimatedGre!;
    const coverage = data.dashboard.coverage!;

    $: model = presentReadinessPage({
        readiness,
        calibration,
        memory,
        performance,
        estimatedGre,
        coverage,
        weakTopics: data.dashboard.weakTopics,
        computedAtMillis: response.computedAtMillis,
    });
</script>

<GrePageHeader
    title="Readiness"
    icon="readiness"
    subtitle="Score estimate details. Study first, inspect when curious."
/>

<GreSection>
    <ReadinessEstimatePanel {model} />
</GreSection>
