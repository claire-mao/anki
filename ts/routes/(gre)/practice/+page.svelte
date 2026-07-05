<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { AnswerExplanation, Question } from "@generated/anki/brainlift_pb";
    import { explainAnswer, recordAttempt } from "@generated/backend";
    import { fade, fly } from "svelte/transition";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { greMotionDuration } from "../motion";
    import { emptyStateContent, emptyStateTitle } from "../empty-states";
    import {
        GRE_CTA_STUDY_PLAN,
        runGreNavAction,
        studyPlanNavAction,
    } from "../gre-navigation";
    import {
        buildPracticeSessionSummary,
        type SessionAttemptRecord,
    } from "../session-completion";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import GreChip from "../ui/GreChip.svelte";
    import GrePanel from "../ui/GrePanel.svelte";
    import GreEmptyState from "../ui/GreEmptyState.svelte";
    import GreSessionCompletePanel from "../ui/GreSessionCompletePanel.svelte";
    import type { PageData } from "./$types";

    import { scheduleGreAtlasAutoSync } from "../gre-sync";
    import {
        buildPracticeRevealRows,
        computeSessionStreak,
        displayQuestionStem,
        formatExplanationCitation,
        formatPracticeMetadataLine,
        formatPracticeTopicLabel,
        formatQuestionType,
        formatSessionAccuracy,
        orderExplanationChoices,
        progressLabelForSession,
        progressPercentForSession,
        resolveCorrectChoice,
        resolveExplanationProvenanceNote,
    } from "./practice-presentation";
    import {
        buildQuestionQueue,
        formatSectionLabel,
        type PracticeSectionFilter,
    } from "./practice-session";

    import "../gre.scss";
    import "../ui/session-complete.scss";
    import "./practice.scss";

    export let data: PageData;

    const sessionId = data.sessionId;
    const questionBank = data.questions;
    const topicFilter = data.topicFilter;

    $: practiceTitle = topicFilter ? formatPracticeTopicLabel(topicFilter) : "Practice";
    $: practiceSubtitle =
        topicFilter && currentQuestion
            ? formatQuestionType(currentQuestion.format)
            : undefined;

    let sectionFilter: PracticeSectionFilter = "all";
    let queue: Question[] = data.queue;
    let questionIndex = 0;
    let questionsCompleted = 0;
    let attemptsRecorded = 0;
    let selected = "";
    let startedAt = Date.now();
    let submitting = false;
    let submitError = "";
    const submitErrorId = "practice-submit-error";
    let sessionComplete = false;
    let sessionAttempts: SessionAttemptRecord[] = [];
    let result: {
        correct: boolean;
        explanation: string;
        topic: string;
        responseTimeMs: number;
        section: string;
        format: string;
        correctChoice: string | null;
    } | null = null;
    // Structured post-answer explanation (AI when enabled, else offline template).
    // Never blocks the result panel; a failure just leaves the plain explanation.
    let explanation: AnswerExplanation | null = null;

    $: currentQuestion = queue[questionIndex];
    $: displayStem = currentQuestion ? displayQuestionStem(currentQuestion.stem) : "";
    $: sessionStreak = computeSessionStreak(sessionAttempts);
    $: sessionAccuracyLabel = formatSessionAccuracy(sessionAttempts);
    $: revealRows = result
        ? buildPracticeRevealRows({
              topic: result.topic,
              section: result.section,
              format: result.format,
          })
        : [];
    $: explanationChoices = explanation
        ? orderExplanationChoices(
              explanation.choices.map((choice) => ({
                  choice: choice.choice,
                  isCorrect: choice.isCorrect,
                  reasoning: choice.reasoning,
              })),
          )
        : [];
    $: explanationCitation = explanation
        ? formatExplanationCitation({
              sourceName: explanation.citationSourceName,
              sourceSection: explanation.citationSourceSection,
              excerpt: explanation.citationExcerpt,
          })
        : null;
    $: explanationNote = explanation
        ? resolveExplanationProvenanceNote({
              provenance: explanation.provenance,
              provenanceNote: explanation.provenanceNote,
          })
        : null;
    $: practiceMetadata = currentQuestion
        ? formatPracticeMetadataLine(
              currentQuestion.topic,
              currentQuestion.section,
              currentQuestion.attribution?.sourceName,
          )
        : null;

    $: progressPercent = progressPercentForSession({
        questionsCompleted,
        queueLength: queue.length,
        sessionComplete,
    });
    $: progressLabel = progressLabelForSession({
        questionsCompleted,
        queueLength: queue.length,
        sessionComplete,
        emptyLabel: emptyStateTitle("noQuestionsFilter"),
    });

    function resetQuestionState(): void {
        selected = "";
        startedAt = Date.now();
        result = null;
        explanation = null;
        submitError = "";
    }

    function applySectionFilter(section: PracticeSectionFilter): void {
        sectionFilter = section;
        queue = buildQuestionQueue(questionBank, section, {
            topicFilter: topicFilter || undefined,
        });
        questionIndex = 0;
        questionsCompleted = 0;
        sessionComplete = queue.length === 0;
        resetSessionAttempts();
        resetQuestionState();
    }

    function nextQuestion(): void {
        resetQuestionState();
        const nextIndex = questionIndex + 1;
        if (nextIndex >= queue.length) {
            sessionComplete = true;
            if (attemptsRecorded > 0) {
                scheduleGreAtlasAutoSync();
            }
            return;
        }
        questionIndex = nextIndex;
    }

    function completeCurrentQuestion(): void {
        questionsCompleted += 1;
        nextQuestion();
    }

    function skipQuestion(): void {
        completeCurrentQuestion();
    }

    async function submit(): Promise<void> {
        if (!selected || submitting || result || !currentQuestion) {
            return;
        }
        submitting = true;
        submitError = "";
        const responseTimeMs = Date.now() - startedAt;
        try {
            const response = await recordAttempt({
                questionId: currentQuestion.id,
                answer: selected,
                responseTimeMs,
                sessionId,
            });
            attemptsRecorded += 1;
            sessionAttempts = [
                ...sessionAttempts,
                { topic: response.topic, correct: response.correct },
            ];
            const correctChoice = resolveCorrectChoice({
                choices: currentQuestion.choices,
                selected,
                correct: response.correct,
                explanation: response.explanation,
            });
            result = {
                correct: response.correct,
                explanation: response.explanation,
                topic: response.topic,
                responseTimeMs,
                section: currentQuestion.section,
                format: currentQuestion.format,
                correctChoice,
            };
            questionsCompleted += 1;

            // Best-effort structured explanation. AI-unavailability is handled
            // server-side (deterministic fallback); a transport failure here
            // simply leaves the plain explanation from recordAttempt in place.
            void loadExplanation(currentQuestion.id, selected);
        } catch {
            submitError = "Could not record this attempt. Please try again.";
        } finally {
            submitting = false;
        }
    }

    async function loadExplanation(questionId: string, answer: string): Promise<void> {
        try {
            const response = await explainAnswer({
                questionId,
                selectedAnswer: answer,
            });
            // Guard against a race where the learner already advanced.
            if (result && response.explanation) {
                explanation = response.explanation;
            }
        } catch {
            // Never surface an error for explanations; the plain text remains.
            explanation = null;
        }
    }

    const sectionFilters: PracticeSectionFilter[] = ["all", "quant", "verbal", "awa"];

    function optionLetter(index: number): string {
        return String.fromCharCode(65 + index);
    }

    function isCorrectChoice(choice: string): boolean {
        return result?.correctChoice === choice;
    }

    $: isLastQuestion = questionIndex + 1 >= queue.length;
    $: practiceSummary = buildPracticeSessionSummary(sessionAttempts);
    $: motionFade = greMotionDuration(160);
    $: motionFly = greMotionDuration(140);

    function resetSessionAttempts(): void {
        sessionAttempts = [];
        attemptsRecorded = 0;
    }
</script>

<div class="practice-page">
    {#if !sessionComplete && (currentQuestion || queue.length === 0)}
        <GrePageHeader
            title={practiceTitle}
            icon="practice"
            subtitle={practiceSubtitle}
        />
    {/if}
    {#if sessionComplete}
        <GrePanel interactive={false} className="practice-complete">
            {#if attemptsRecorded === 0}
                <GreEmptyState
                    content={emptyStateContent("noQuestionsFilter")}
                    showChecklist={false}
                    on:action={() => applySectionFilter("all")}
                />
            {:else}
                <GreSessionCompletePanel
                    summary={practiceSummary}
                    secondaryLabel={practiceSummary.nextAction.label ===
                    "Practice again"
                        ? GRE_CTA_STUDY_PLAN
                        : "Practice again"}
                    onSecondary={() => {
                        if (practiceSummary.nextAction.label === "Practice again") {
                            runGreNavAction(studyPlanNavAction());
                            return;
                        }
                        applySectionFilter(sectionFilter);
                    }}
                />
            {/if}
            {#if attemptsRecorded === 0}
                <GreButtonRow className="practice-actions">
                    <GreButton
                        variant="primary"
                        on:click={() => applySectionFilter("all")}
                    >
                        Show all sections
                    </GreButton>
                </GreButtonRow>
            {/if}
        </GrePanel>
    {:else if currentQuestion}
        <header class="practice-topbar">
            <div
                class="practice-progress"
                role="progressbar"
                aria-valuemin="0"
                aria-valuemax="100"
                aria-valuenow={progressPercent}
                aria-label={progressLabel}
            >
                <div class="practice-progress-track">
                    <div
                        class="practice-progress-fill"
                        style:width="{progressPercent}%"
                    ></div>
                </div>
                <div class="practice-progress-meta">
                    <span class="practice-progress-label">{progressLabel}</span>
                    {#if !result}
                        <span
                            class="practice-session-stats"
                            aria-label="Session performance"
                        >
                            <span>Streak {sessionStreak}</span>
                            <span aria-hidden="true">·</span>
                            <span>Accuracy {sessionAccuracyLabel}</span>
                        </span>
                    {/if}
                </div>
            </div>
            {#if !topicFilter}
                <div class="practice-sections" role="group" aria-label="Section filter">
                    {#each sectionFilters as section}
                        <GreChip
                            active={sectionFilter === section}
                            on:click={() => applySectionFilter(section)}
                        >
                            {formatSectionLabel(section)}
                        </GreChip>
                    {/each}
                </div>
            {/if}
        </header>

        {#key questionIndex}
            <section
                class="practice-card"
                in:fly={{ y: 6, duration: motionFly }}
                out:fade={{ duration: motionFade }}
            >
                <div class="question-focus">
                    <dl class="question-metadata">
                        {#if practiceMetadata}
                            <div class="question-metadata-row">
                                {#if practiceMetadata.kind === "source"}
                                    <dt class="sr-only">Source</dt>
                                    <dd>Source: {practiceMetadata.text}</dd>
                                {:else}
                                    <dt class="sr-only">Task type</dt>
                                    <dd>{practiceMetadata.text}</dd>
                                {/if}
                            </div>
                        {/if}
                        <div class="question-metadata-row">
                            <dt class="sr-only">Question type</dt>
                            <dd>
                                Question Type: {formatQuestionType(
                                    currentQuestion.format,
                                )}
                            </dd>
                        </div>
                    </dl>

                    <p class="question-stem">{displayStem}</p>
                </div>

                <div class="choices" role="radiogroup" aria-label="Answer choices">
                    {#each currentQuestion.choices as choice, index}
                        <label
                            class="choice"
                            class:choice-selected={selected === choice && !result}
                            class:choice-correct={result !== null &&
                                selected === choice &&
                                result.correct}
                            class:choice-incorrect={result !== null &&
                                selected === choice &&
                                !result.correct}
                            class:choice-correct-reveal={result !== null &&
                                isCorrectChoice(choice) &&
                                selected !== choice}
                            class:choice-locked={result !== null}
                        >
                            <input
                                type="radio"
                                bind:group={selected}
                                value={choice}
                                disabled={result !== null || submitting}
                            />
                            <span class="choice-letter" aria-hidden="true">
                                {optionLetter(index)}
                            </span>
                            <span class="choice-text">{choice}</span>
                        </label>
                    {/each}
                </div>

                {#if result}
                    <div
                        class="result-panel"
                        class:correct={result.correct}
                        class:incorrect={!result.correct}
                        in:fade={{ duration: motionFade }}
                    >
                        <p class="result-status">
                            {result.correct ? "Correct" : "Incorrect"}
                        </p>
                        {#if !result.correct && result.correctChoice}
                            <p class="result-correct-answer">
                                Correct answer: <strong>{result.correctChoice}</strong>
                            </p>
                        {/if}

                        {#if explanation}
                            <p class="result-explanation">{explanation.summary}</p>
                            {#if explanationChoices.length}
                                <ul class="result-choice-breakdown">
                                    {#each explanationChoices as row}
                                        <li class:breakdown-correct={row.isCorrect}>
                                            <span class="breakdown-choice">
                                                {row.choice}
                                                {#if row.isCorrect}
                                                    <span class="breakdown-tag">
                                                        Correct
                                                    </span>
                                                {/if}
                                            </span>
                                            <span class="breakdown-reasoning">
                                                {row.reasoning}
                                            </span>
                                        </li>
                                    {/each}
                                </ul>
                            {/if}
                            {#if explanationCitation}
                                <p class="result-citation">
                                    Source: {explanationCitation}
                                </p>
                            {/if}
                            {#if explanationNote}
                                <p class="result-provenance-note">{explanationNote}</p>
                            {/if}
                        {:else}
                            <p class="result-explanation">{result.explanation}</p>
                        {/if}

                        <dl class="result-details">
                            {#each revealRows as row}
                                <div class="result-details-row">
                                    <dt>{row.label}</dt>
                                    <dd>{row.value}</dd>
                                </div>
                            {/each}
                        </dl>
                    </div>

                    <GreButton
                        variant="primary"
                        size="lg"
                        className="practice-cta"
                        on:click={nextQuestion}
                    >
                        {isLastQuestion ? "Finish session" : "Next question"}
                    </GreButton>
                {:else}
                    <div class="practice-actions-block">
                        <GreButton
                            variant="primary"
                            size="lg"
                            className="practice-cta"
                            loading={submitting}
                            disabled={!selected}
                            ariaDescribedby={submitError ? submitErrorId : undefined}
                            on:click={submit}
                        >
                            {submitting ? "Checking…" : "Confirm Answer"}
                        </GreButton>

                        <GreButtonRow className="practice-secondary-actions">
                            <GreButton variant="secondary" on:click={skipQuestion}>
                                Skip
                            </GreButton>
                        </GreButtonRow>
                    </div>

                    {#if submitError}
                        <p class="practice-error" id={submitErrorId} role="alert">
                            {submitError}
                        </p>
                    {/if}
                {/if}
            </section>
        {/key}
    {:else}
        <GreEmptyState
            content={emptyStateContent("noQuestionsFilter")}
            showChecklist={false}
            on:action={() => applySectionFilter("all")}
        />
    {/if}
</div>
