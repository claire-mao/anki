<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import DailyStudyPlan from "../DailyStudyPlan.svelte";
    import { emptyStateContent } from "../empty-states";
    import { presentStudyPlanRecommendations } from "../recommendation-presentation";
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreStudyRecommendationList from "../ui/GreStudyRecommendationList.svelte";
    import GreCoverageSummary from "../ui/GreCoverageSummary.svelte";
    import type { PageData } from "./$types";

    import "./study-plan.scss";

    export let data: PageData;

    const plan = data.plan;
    const coverage = plan.coverage!;
    const dailyPlan = plan.dailyPlan!;
    const status = data.status;
    const recommendations = presentStudyPlanRecommendations(plan.recommendations);
</script>

<GrePageHeader
    title="Study plan"
    icon="calendar"
    subtitle="Pick a task and start studying."
/>

<section class="study-plan-page">
    <DailyStudyPlan
        plan={dailyPlan}
        studyStatus={status}
        recentAttempts={data.recentAttempts}
        primary
    />

    <details class="study-plan-recommendations">
        <summary>Recommended focus areas</summary>
        {#if recommendations.length === 0}
            <GreEmptyState content={emptyStateContent("studyPlanRecommendations")} />
        {:else}
            <GreStudyRecommendationList {recommendations} />
        {/if}
    </details>

    <details class="study-plan-coverage">
        <summary>Topic coverage breakdown</summary>
        <GreCoverageSummary {coverage} compact />
    </details>
</section>
