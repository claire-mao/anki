<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { browser } from "$app/environment";
    import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";
    import { onDestroy, onMount } from "svelte";
    import { fade, fly } from "svelte/transition";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { GRE_CTA_REVIEW } from "../gre-navigation";
    import { buildStudyCaughtUpSummary } from "../session-completion";
    import { greMotionDuration } from "../motion";
    import GreButton from "../ui/GreButton.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreSessionCompletePanel from "../ui/GreSessionCompletePanel.svelte";
    import GreText from "../ui/GreText.svelte";
    import {
        queueBadge,
        studyMetrics,
        studyProgressLabel,
        studyProgressPercent,
    } from "./study-session";
    import {
        answerStudyCard,
        loadStudyReview,
        type StudyReviewState,
    } from "./study-review";
    import type { PageData } from "./$types";

    import "../gre.scss";
    import "../ui/session-complete.scss";
    import "./study.scss";

    export let data: PageData;

    const status = data.status;
    const dashboard = data.dashboard;

    let review: StudyReviewState | null = null;
    let loading = true;
    let revealed = false;
    let answering = false;
    let loadError = false;
    let reviewedThisSession = 0;
    let cardStartedAt = Date.now();

    $: motionFade = greMotionDuration(160);
    $: motionFly = greMotionDuration(140);
    $: card = review?.card ?? null;
    $: metrics = review ? studyMetrics(review) : [];
    $: progressPercent = review
        ? studyProgressPercent(reviewedThisSession, review.dueTotal)
        : 0;
    $: progressLabel = review
        ? studyProgressLabel(reviewedThisSession, review.dueTotal)
        : "";
    $: badge = card ? queueBadge(card.queue) : null;

    $: caughtUpSummary = buildStudyCaughtUpSummary({
        weakTopics: dashboard.weakTopics,
        recommendedTopics: dashboard.recommendedTopics,
        dueTotal: review?.dueTotal ?? 0,
        deckName: review?.deckName ?? status.deckName,
        studiedCards: dashboard.memory?.studiedCards ?? 0,
        coveredLeafCount: dashboard.coverage?.coveredLeafCount ?? 0,
        extraStudyAvailable: status.extraStudyAvailable,
        availableNewCount: status.availableNewCount,
        nextReviewInDays: status.nextReviewInDays,
    });

    async function load(unlockExtra = false): Promise<void> {
        loading = true;
        loadError = false;
        try {
            review = await loadStudyReview(unlockExtra);
            revealed = false;
            cardStartedAt = Date.now();
        } catch {
            loadError = true;
        } finally {
            loading = false;
        }
    }

    // Anki card templates ship their own CSS. Inject it through the DOM (not a
    // template <style>, which the Svelte preprocessor would try to parse) so the
    // rendered .card content looks correct, scoped to this reviewer.
    let cardStyleEl: HTMLStyleElement | null = null;

    function applyCardCss(css: string): void {
        if (!browser) {
            return;
        }
        if (!cardStyleEl) {
            cardStyleEl = document.createElement("style");
            cardStyleEl.dataset.greStudyCard = "true";
            document.head.appendChild(cardStyleEl);
        }
        cardStyleEl.textContent = css;
    }

    $: if (browser && card) {
        applyCardCss(card.css);
    }

    onDestroy(() => {
        cardStyleEl?.remove();
        cardStyleEl = null;
    });

    function reveal(): void {
        if (card && !revealed) {
            revealed = true;
        }
    }

    async function grade(rating: number): Promise<void> {
        if (!review || !card || !revealed || answering) {
            return;
        }
        answering = true;
        const current = card;
        try {
            const next = await answerStudyCard({
                deckName: review.deckName,
                cardId: current.cardId,
                rating,
                states: current.states,
                millisecondsTaken: Date.now() - cardStartedAt,
            });
            reviewedThisSession += 1;
            review = next;
            revealed = false;
            cardStartedAt = Date.now();
        } catch {
            loadError = true;
        } finally {
            answering = false;
        }
    }

    function startNativeReview(): void {
        if (bridgeCommandsAvailable()) {
            bridgeCommand("greStartReview");
        }
    }

    function onKeydown(event: KeyboardEvent): void {
        if (!card || answering) {
            return;
        }
        if (!revealed && (event.code === "Space" || event.code === "Enter")) {
            event.preventDefault();
            reveal();
            return;
        }
        if (revealed) {
            const index = ["Digit1", "Digit2", "Digit3", "Digit4"].indexOf(event.code);
            if (index >= 0 && index < card.buttons.length) {
                event.preventDefault();
                void grade(card.buttons[index].rating);
            } else if (event.code === "Space" || event.code === "Enter") {
                // Default to Good on space once the answer is shown.
                const good = card.buttons.find((b) => b.variant === "good");
                if (good) {
                    event.preventDefault();
                    void grade(good.rating);
                }
            }
        }
    }

    onMount(load);
</script>

<svelte:window on:keydown={onKeydown} />

<GrePageHeader
    title="Study"
    icon="study"
    subtitle="Today's Review · {review?.deckName ?? status.deckName}"
/>

{#if card}
    <header class="study-topbar">
        <div
            class="study-progress"
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow={progressPercent}
            aria-label={progressLabel}
        >
            <div class="study-progress-track">
                <div class="study-progress-fill" style:width="{progressPercent}%"></div>
            </div>
            <div class="study-progress-meta">
                <span class="study-progress-label">{progressLabel}</span>
                <span class="study-progress-count">Reviewed {reviewedThisSession}</span>
            </div>
        </div>
    </header>

    <div class="study-metrics" role="group" aria-label="Session overview">
        {#each metrics as metric}
            <div class="study-metric">
                <span class="study-metric-value">{metric.value}</span>
                <span class="study-metric-label">{metric.label}</span>
            </div>
        {/each}
    </div>

    <div class="study-layout">
        {#key card.cardId}
            <section
                class="study-card"
                in:fly={{ y: 6, duration: motionFly }}
                out:fade={{ duration: motionFade }}
            >
                {#if badge}
                    <span class="study-card-badge study-card-badge-{badge.tone}">
                        {badge.label}
                    </span>
                {/if}

                <div class="study-card-render card">
                    {@html revealed ? card.answerHtml : card.questionHtml}
                </div>

                {#if !revealed}
                    <GreButton
                        variant="primary"
                        size="lg"
                        className="study-reveal"
                        on:click={reveal}
                    >
                        Show answer
                    </GreButton>
                    <p class="study-hint">Press Space to reveal</p>
                {:else}
                    <div class="study-grades" role="group" aria-label="Grade this card">
                        {#each card.buttons as button}
                            <button
                                type="button"
                                class="study-grade study-grade-{button.variant}"
                                disabled={answering}
                                aria-label="{button.name} — next review {button.label}"
                                on:click={() => grade(button.rating)}
                            >
                                <span class="study-grade-name">{button.name}</span>
                                <span class="study-grade-interval">{button.label}</span>
                                <span class="study-grade-key" aria-hidden="true">
                                    {button.shortcut}
                                </span>
                            </button>
                        {/each}
                    </div>
                    <p class="study-hint">How well did you remember it?</p>
                {/if}
            </section>
        {/key}

        <aside class="study-sidebar" aria-label="Card details">
            <h2 class="study-sidebar-title">This card</h2>
            <dl class="study-sidebar-list">
                <div class="study-sidebar-row">
                    <dt>Queue</dt>
                    <dd>{badge?.label ?? "Review"}</dd>
                </div>
                <div class="study-sidebar-row">
                    <dt>Deck</dt>
                    <dd>{review?.deckName ?? status.deckName}</dd>
                </div>
                <div class="study-sidebar-row">
                    <dt>Remaining</dt>
                    <dd>{review?.dueTotal ?? 0}</dd>
                </div>
            </dl>
            <p class="study-sidebar-tip">
                Grade honestly — the scheduler uses your rating to time the next review
                for lasting memory.
            </p>
        </aside>
    </div>
{:else if loading}
    <GrePanel>
        <div class="study-loading" role="status" aria-live="polite">
            <div class="gre-loading-spinner" aria-hidden="true"></div>
            <GreText variant="caption" muted>Loading your review session…</GreText>
        </div>
    </GrePanel>
{:else if loadError}
    <GrePanel className="study-guide">
        <h2 class="study-guide-title">Couldn't load your review session</h2>
        <p class="study-guide-body">
            Something interrupted the review queue. Try again, or open the full reviewer.
        </p>
        <div class="study-guide-actions">
            <GreButton variant="primary" on:click={() => load()}>Try again</GreButton>
            {#if bridgeCommandsAvailable()}
                <GreButton variant="ghost" on:click={startNativeReview}>
                    {GRE_CTA_REVIEW}
                </GreButton>
            {/if}
        </div>
    </GrePanel>
{:else if !review?.deckExists}
    <GrePanel className="study-guide">
        <h2 class="study-guide-title">Couldn't load your GRE flashcards</h2>
        <p class="study-guide-body">
            GRE Atlas includes starter flashcards. Reload this page to try again.
        </p>
        <GreButton variant="primary" on:click={() => location.reload()}>
            Try again
        </GreButton>
    </GrePanel>
{:else}
    <GrePanel className="study-guide study-caught-up">
        <GreSessionCompletePanel
            summary={caughtUpSummary}
            showNextSteps={caughtUpSummary.nextAction !== undefined}
            secondaryLabel={caughtUpSummary.secondaryAction?.label}
        />
    </GrePanel>
{/if}
