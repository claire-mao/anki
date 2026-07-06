<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreIcon from "../GreIcon.svelte";
    import GrePageHeader from "../GrePageHeader.svelte";
    import {
        MIN_COVERAGE_PERCENT,
        MIN_PRACTICE_ATTEMPTS,
        MIN_STUDIED_CARDS,
    } from "../empty-states";
    import {
        METHODOLOGY_COVERAGE,
        METHODOLOGY_INTRO,
        METHODOLOGY_MEMORY,
        METHODOLOGY_PAGE_TITLE,
        METHODOLOGY_PERFORMANCE,
        METHODOLOGY_RANGE,
        METHODOLOGY_READINESS,
    } from "../methodology-presentation";

    import "../gre.scss";
    import "./methodology.scss";
</script>

<GrePageHeader title={METHODOLOGY_PAGE_TITLE} icon="info" />

<div class="methodology">
    <p class="methodology-intro">
        {METHODOLOGY_INTRO}
    </p>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="memory" size="md" />
            Memory
        </summary>
        <div class="methodology-body">
        {#each METHODOLOGY_MEMORY as paragraph}
            <p class="methodology-paragraph">{paragraph}</p>
        {/each}
        </div>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="performance" size="md" />
            Performance
        </summary>
        <div class="methodology-body">
        {#each METHODOLOGY_PERFORMANCE as paragraph}
            <p class="methodology-paragraph">{paragraph}</p>
        {/each}
        </div>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="dashboard" size="md" />
            Topic coverage
        </summary>
        <div class="methodology-body">
        {#each METHODOLOGY_COVERAGE as paragraph}
            <p class="methodology-paragraph">{paragraph}</p>
        {/each}
        </div>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="readiness" size="md" />
            Readiness
        </summary>
        <div class="methodology-body">
        {#each METHODOLOGY_READINESS as paragraph}
            <p class="methodology-paragraph">{paragraph}</p>
        {/each}
        </div>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="score" size="md" />
            Why a range, not one number
        </summary>
        <div class="methodology-body">
        {#each METHODOLOGY_RANGE as paragraph}
            <p class="methodology-paragraph">{paragraph}</p>
        {/each}
        </div>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="alert" size="md" />
            When no estimate is shown
        </summary>
        <div class="methodology-body">
            <p class="methodology-paragraph">
                Without enough evidence from flashcard reviews and practice, GRE Atlas <strong>abstains</strong>
                 and shows a locked state instead of a number. The estimate stays hidden until
                you've collected:
            </p>
            <ul class="methodology-thresholds">
                <li>at least {MIN_STUDIED_CARDS} reviewed flashcards,</li>
                <li>at least {MIN_PRACTICE_ATTEMPTS} practice questions, and</li>
                <li>at least {MIN_COVERAGE_PERCENT}% GRE topic coverage.</li>
            </ul>
        </div>
    </details>
</div>
