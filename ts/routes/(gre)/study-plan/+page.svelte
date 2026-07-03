<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import DailyStudyPlan from "../DailyStudyPlan.svelte";
    import { emptyStateContent } from "../empty-states";
    import { presentStudyPlanRecommendations } from "../recommendation-presentation";
    import GrePageHeader from "../GrePageHeader.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreSection from "../ui/GreSection.svelte";
    import GreCoverageSummary from "../ui/GreCoverageSummary.svelte";
    import GreStudyRecommendationList from "../ui/GreStudyRecommendationList.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    const plan = data.plan;
    const coverage = plan.coverage!;
    const dailyPlan = plan.dailyPlan!;
    const status = data.status;
    const recommendations = presentStudyPlanRecommendations(plan.recommendations);

    function formatTimestampMillis(millis: bigint): string {
        return new Date(Number(millis)).toLocaleString(undefined, {
            dateStyle: "medium",
            timeStyle: "short",
        });
    }
</script>

<GrePageHeader
    title="Study plan"
    icon="calendar"
    subtitle={plan.summary}
    meta="Last updated {formatTimestampMillis(plan.computedAtMillis)}"
/>

<GreSection>
    <DailyStudyPlan plan={dailyPlan} studyStatus={status} />

    <GrePanel title="Coverage">
        <GreCoverageSummary {coverage} />
    </GrePanel>

    <GrePanel title="Impact-ranked topics">
        {#if recommendations.length === 0}
            <GreEmptyState content={emptyStateContent("studyPlanRecommendations")} />
        {:else}
            <p class="study-plan-ranked-intro">
                Sorted by expected GRE impact, not study order.
            </p>
            <GreStudyRecommendationList {recommendations} />
        {/if}
    </GrePanel>
</GreSection>

<style lang="scss">
    .study-plan-ranked-intro {
        margin: 0 0 var(--gre-space-3);
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }
</style>
