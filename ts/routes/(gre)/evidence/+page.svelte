<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import { presentAiEvaluation } from "./ai-evaluation-presentation";
    import {
        presentBuildInformation,
        presentDocumentationEvidence,
        presentSyncVerification,
    } from "./build-sync-presentation";
    import AiEvaluationSection from "./AiEvaluationSection.svelte";
    import BuildInformationSection from "./BuildInformationSection.svelte";
    import DocumentationSection from "./DocumentationSection.svelte";
    import HonestReportingSection from "./HonestReportingSection.svelte";
    import MemoryModelSection from "./MemoryModelSection.svelte";
    import PerformanceModelSection from "./PerformanceModelSection.svelte";
    import ReadinessModelSection from "./ReadinessModelSection.svelte";
    import StudyFeatureExperimentSection from "./StudyFeatureExperimentSection.svelte";
    import SyncVerificationSection from "./SyncVerificationSection.svelte";
    import {
        parseEvalReportSections,
        presentReadinessModel,
        presentStudyFeatureExperiment,
    } from "./eval-report-presentation";
    import { presentHonestReporting } from "./honest-reporting-presentation";
    import type { PageData } from "./$types";

    import "../gre.scss";
    import "./evidence.scss";

    export let data: PageData;

    $: evalSections = parseEvalReportSections(data.evalReportJson);
    $: aiEvaluation = presentAiEvaluation({
        aiEnabled: data.aiEval.aiEnabled,
        reportJson: data.aiEval.json,
    });
    $: readinessModel = presentReadinessModel(evalSections.calibration);
    $: honestReporting = presentHonestReporting(data.honestReporting);
    $: studyFeatureExperiment = presentStudyFeatureExperiment(evalSections.ablation);
    $: syncVerification = presentSyncVerification(data.verification);
    $: buildInformation = presentBuildInformation(data.verification);
    $: documentation = presentDocumentationEvidence();
</script>

<GrePageHeader
    title="Evidence"
    icon="info"
    subtitle="Scientific evaluation outputs that demonstrate model validation and project requirements."
/>

<GreSection gap="lg">
    <div class="evidence-page">
        <MemoryModelSection response={data.memoryEval} />
        <PerformanceModelSection response={data.performanceEval} />
        <ReadinessModelSection model={readinessModel} />
        <AiEvaluationSection model={aiEvaluation} />
        <StudyFeatureExperimentSection model={studyFeatureExperiment} />
        <HonestReportingSection model={honestReporting} />
        <SyncVerificationSection model={syncVerification} />
        <BuildInformationSection model={buildInformation} />
        <DocumentationSection model={documentation} />
    </div>
</GreSection>
