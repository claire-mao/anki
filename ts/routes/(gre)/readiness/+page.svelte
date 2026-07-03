<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreCalibrationPanel from "../ui/GreCalibrationPanel.svelte";
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

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }
</script>

<GrePageHeader
    title="Readiness"
    icon="readiness"
    subtitle="How much evidence supports your GRE readiness estimate."
    meta="Last updated {formatTimestampMillis(response.computedAtMillis)}"
/>

<GreSection>
    <ReadinessEstimatePanel {model} />
    <GreCalibrationPanel
        {readiness}
        {calibration}
        variant="full"
        showImprovements={false}
    />
</GreSection>
