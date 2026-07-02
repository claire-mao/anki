<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GrePageHeader from "../GrePageHeader.svelte";
    import ReadinessSummary from "../summaries/ReadinessSummary.svelte";
    import GreCalibrationPanel from "../ui/GreCalibrationPanel.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    const response = data.response;
    const readiness = response.readiness!;
    const calibration = response.calibration!;

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
    subtitle="Prediction calibration and your readiness score."
    meta="Last updated {formatTimestampMillis(response.computedAtMillis)}"
/>

<GreSection>
    <GreCalibrationPanel {readiness} {calibration} variant="full" />
    <ReadinessSummary {readiness} {calibration} />
</GreSection>
