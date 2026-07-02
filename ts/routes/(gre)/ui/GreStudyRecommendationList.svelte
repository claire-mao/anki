<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { StudyRecommendationPresentation } from "../recommendation-presentation";

    import GreStudyRecommendationCard from "./GreStudyRecommendationCard.svelte";

    export let recommendations: StudyRecommendationPresentation[];
    export let compact = false;
</script>

<div class="gre-study-recommendation-list gre-stagger" role="list">
    {#each recommendations as recommendation (recommendation.topicId ?? recommendation.title)}
        <div role="listitem">
            <GreStudyRecommendationCard {recommendation} {compact} />
        </div>
    {/each}
</div>

<style lang="scss">
    .gre-study-recommendation-list {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-3);
    }
</style>
