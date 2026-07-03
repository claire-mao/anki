<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { OnboardingPresentation } from "../onboarding-presentation";
    import { runGreNavAction } from "../gre-navigation";
    import GreButton from "./GreButton.svelte";
    import GreProgressBar from "./GreProgressBar.svelte";
    import GreProgressRing from "./GreProgressRing.svelte";

    export let model: OnboardingPresentation;
</script>

<section class="gre-onboarding" aria-label={model.title}>
    <header class="gre-onboarding-header">
        <h2 class="gre-onboarding-title">{model.title}</h2>
        {#if model.lead}
            <p class="gre-onboarding-lead">{model.lead}</p>
        {/if}
    </header>

    <div class="gre-onboarding-meter">
        <GreProgressRing
            value={model.evidencePercent}
            size="lg"
            label={model.ringLabel}
            color="var(--state-review)"
        />
        <p class="gre-onboarding-meter-caption">{model.meterCaption}</p>
    </div>

    <div class="gre-onboarding-evidence" aria-label="Evidence breakdown">
        {#each model.categories as category (category.id)}
            <GreProgressBar
                label={category.label}
                value={category.percent}
                max={100}
                compact
            />
        {/each}
    </div>

    <section class="gre-onboarding-next" aria-label="Next best action">
        <h3 class="gre-onboarding-next-title">Next best action</h3>
        <p class="gre-onboarding-next-label">{model.nextAction.label}</p>
        <p class="gre-onboarding-next-impact">
            Estimated impact
            <span class="gre-onboarding-next-impact-value">
                +{model.nextAction.estimatedImpact} toward estimate
            </span>
        </p>
        <GreButton
            variant="primary"
            size="lg"
            on:click={(event) => runGreNavAction(model.nextAction, event)}
        >
            {model.nextAction.buttonLabel}
        </GreButton>
    </section>
</section>

<style lang="scss">
    .gre-onboarding {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-4);
        padding: var(--gre-card-padding);
        border-radius: var(--gre-radius-lg);
        background: var(--gre-surface-bg);
        box-shadow: var(--gre-shadow-md);
    }

    .gre-onboarding-header {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-1);
    }

    .gre-onboarding-title {
        margin: 0;
        font-size: var(--gre-font-h1);
        font-weight: var(--gre-weight-h1);
        line-height: var(--gre-lh-h1);
        letter-spacing: -0.02em;
    }

    .gre-onboarding-lead {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg-subtle);
        max-width: 36rem;
    }

    .gre-onboarding-meter {
        display: flex;
        align-items: center;
        gap: var(--gre-space-3);
        padding: var(--gre-card-padding-compact);
        border-radius: var(--gre-radius-md);
        background: var(--gre-surface-bg-muted);
        box-shadow: var(--gre-shadow-sm);
    }

    .gre-onboarding-meter-caption {
        margin: 0;
        font-size: var(--gre-font-body);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .gre-onboarding-evidence {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
    }

    .gre-onboarding-next {
        display: flex;
        flex-direction: column;
        gap: var(--gre-space-2);
        padding-top: var(--gre-space-2);
        border-top: 1px solid color-mix(in srgb, var(--border) 35%, transparent);
    }

    .gre-onboarding-next-title {
        margin: 0;
        font-size: var(--gre-font-caption);
        font-weight: var(--gre-weight-label);
        line-height: var(--gre-lh-caption);
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--fg-subtle);
    }

    .gre-onboarding-next-label {
        margin: 0;
        font-size: var(--gre-font-body);
        line-height: var(--gre-lh-body);
        color: var(--fg);
    }

    .gre-onboarding-next-impact {
        margin: 0;
        font-size: var(--gre-font-caption);
        line-height: var(--gre-lh-caption);
        color: var(--fg-subtle);
    }

    .gre-onboarding-next-impact-value {
        color: var(--state-new);
        font-weight: var(--gre-weight-label);
    }

    @media (max-width: 767px) {
        .gre-onboarding {
            padding: var(--gre-card-padding-compact);
        }
    }
</style>
