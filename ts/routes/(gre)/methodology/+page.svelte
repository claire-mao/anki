<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreIcon from "../GreIcon.svelte";
    import GrePageHeader from "../GrePageHeader.svelte";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import { greNavAction, greNavItem, GRE_CTA_REVIEW } from "../gre-navigation";
    import {
        MIN_COVERAGE_PERCENT,
        MIN_PRACTICE_ATTEMPTS,
        MIN_STUDIED_CARDS,
    } from "../empty-states";
    import {
        METHODOLOGY_INTRO,
        METHODOLOGY_PAGE_SUBTITLE,
        METHODOLOGY_PAGE_TITLE,
    } from "../methodology-presentation";

    import "../gre.scss";
    import "./methodology.scss";
</script>

<GrePageHeader
    title={METHODOLOGY_PAGE_TITLE}
    icon="info"
    subtitle={METHODOLOGY_PAGE_SUBTITLE}
/>

<section class="methodology-launch">
    <GreButtonRow>
        <GreButton variant="primary" navAction={greNavAction(greNavItem("study"))}>
            {GRE_CTA_REVIEW}
        </GreButton>
        <GreButton navAction={greNavAction(greNavItem("dashboard"))}>
            Back to dashboard
        </GreButton>
    </GreButtonRow>
</section>

<div class="methodology">
    <p class="methodology-intro">
        {METHODOLOGY_INTRO}
    </p>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="memory" size="md" />
            Memory
        </summary>
        <p>
            As you review GRE flashcards, GRE Atlas uses the FSRS spaced-repetition
            model to estimate how likely you are to recall each card at test time.
            Averaged across your deck, that's your <strong>Memory</strong> signal.
        </p>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="performance" size="md" />
            Performance
        </summary>
        <p>
            Every GRE-style practice question you answer is recorded.
            <strong>Performance</strong> is your accuracy on those questions, with a
            confidence interval that tightens as you answer more.
        </p>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="readiness" size="md" />
            Readiness
        </summary>
        <p>
            <strong>Readiness</strong> combines Memory, Performance, and topic coverage
            into a projected GRE score, continuously checked against your earlier
            predictions (calibration).
        </p>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="score" size="md" />
            Why a range, not one number
        </summary>
        <p>
            Every prediction carries uncertainty, so GRE Atlas shows a range instead of
            a false-precise number. A wider range means less evidence.
        </p>
    </details>

    <details class="methodology-card">
        <summary class="methodology-heading">
            <GreIcon name="alert" size="md" />
            When no estimate is shown
        </summary>
        <p>
            Without enough evidence, GRE Atlas <strong>abstains</strong> and shows a
            locked state instead of a number. The estimate stays hidden until you've
            collected:
        </p>
        <ul class="methodology-thresholds">
            <li>at least {MIN_STUDIED_CARDS} reviewed flashcards,</li>
            <li>at least {MIN_PRACTICE_ATTEMPTS} practice questions, and</li>
            <li>at least {MIN_COVERAGE_PERCENT}% GRE topic coverage.</li>
        </ul>
    </details>
</div>

<style lang="scss">
    .methodology-launch {
        margin-bottom: var(--gre-space-4);
    }

    .methodology-card summary {
        cursor: pointer;
        list-style: none;
    }

    .methodology-card summary::-webkit-details-marker {
        display: none;
    }

    .methodology-card[open] summary {
        margin-bottom: var(--gre-space-3);
    }
</style>
