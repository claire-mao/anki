// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// In-page reviewer data flow for the BrainLift Study page. This mirrors the
// mobile bridge (mobile/mobile_bridge/src/study_pages.rs) exactly and uses the
// standard scheduler RPCs, so FSRS scheduling/grading is fully preserved — the
// frontend only renders cards and forwards the learner's rating.

import type { RenderedTemplateNode } from "@generated/anki/card_rendering_pb";
import { CardAnswer_Rating, type SchedulingStates } from "@generated/anki/scheduler_pb";
import { QueuedCards_Queue } from "@generated/anki/scheduler_pb";
import {
    answerCard,
    describeNextStates,
    getDeckIdByName,
    getGreStudyStatus,
    getQueuedCards,
    renderExistingCard,
    setCurrentDeck,
    startGreExtraStudy,
} from "@generated/backend";

import { gradeButtonsFromLabels, type StudyGradeButton } from "./study-session";

const GRE_DECK_NAME = "GRE Atlas";

export interface StudyReviewCard {
    cardId: bigint;
    queue: string;
    questionHtml: string;
    answerHtml: string;
    css: string;
    buttons: StudyGradeButton[];
    states: SchedulingStates;
}

export interface StudyReviewState {
    deckExists: boolean;
    deckName: string;
    dueNew: number;
    dueLearn: number;
    dueReview: number;
    dueTotal: number;
    card: StudyReviewCard | null;
}

function queueLabel(queue: QueuedCards_Queue): string {
    switch (queue) {
        case QueuedCards_Queue.NEW:
            return "new";
        case QueuedCards_Queue.LEARNING:
            return "learning";
        default:
            return "review";
    }
}

function nodesToHtml(nodes: RenderedTemplateNode[]): string {
    return nodes
        .map((node) => (node.value.case === "text" ? node.value.value : ""))
        .join("");
}

async function ensureGreDeckSelected(): Promise<void> {
    const deck = await getDeckIdByName({ val: GRE_DECK_NAME });
    await setCurrentDeck({ did: deck.did });
}

/**
 * Load the next due card (rendered HTML + scheduler-provided grade labels).
 * Pass `unlockExtra` to release the next paced batch before fetching, matching
 * the "Study N more cards" action.
 */
export async function loadStudyReview(unlockExtra = false): Promise<StudyReviewState> {
    const status = await getGreStudyStatus({});
    if (!status.deckExists) {
        return {
            deckExists: false,
            deckName: status.deckName,
            dueNew: 0,
            dueLearn: 0,
            dueReview: 0,
            dueTotal: 0,
            card: null,
        };
    }

    await ensureGreDeckSelected();
    if (unlockExtra) {
        await startGreExtraStudy({});
    }

    return fetchNextCard(status.deckName);
}

async function fetchNextCard(deckName: string): Promise<StudyReviewState> {
    const queued = await getQueuedCards({
        fetchLimit: 1,
        intradayLearningOnly: false,
    });
    const dueTotal = queued.newCount + queued.learningCount + queued.reviewCount;
    const base = {
        deckExists: true,
        deckName,
        dueNew: queued.newCount,
        dueLearn: queued.learningCount,
        dueReview: queued.reviewCount,
        dueTotal,
    };

    const top = queued.cards[0];
    if (!top?.card || !top.states) {
        return { ...base, card: null };
    }

    const rendered = await renderExistingCard({
        cardId: top.card.id,
        browser: false,
        partialRender: false,
    });
    const labels = await describeNextStates(top.states);

    return {
        ...base,
        card: {
            cardId: top.card.id,
            queue: queueLabel(top.queue),
            questionHtml: nodesToHtml(rendered.questionNodes),
            answerHtml: nodesToHtml(rendered.answerNodes),
            css: rendered.css,
            buttons: gradeButtonsFromLabels(labels.vals),
            states: top.states,
        },
    };
}

function stateForRating(
    states: SchedulingStates,
    rating: number,
): SchedulingStates["again"] {
    switch (rating) {
        case CardAnswer_Rating.AGAIN:
            return states.again;
        case CardAnswer_Rating.HARD:
            return states.hard;
        case CardAnswer_Rating.EASY:
            return states.easy;
        default:
            return states.good;
    }
}

/**
 * Grade the current card via the scheduler, then return the next card.
 * `answer_card` receives the scheduler's own computed states, so intervals and
 * FSRS memory updates are identical to the native reviewer.
 */
export async function answerStudyCard(input: {
    deckName: string;
    cardId: bigint;
    rating: number;
    states: SchedulingStates;
    millisecondsTaken: number;
}): Promise<StudyReviewState> {
    await answerCard({
        cardId: input.cardId,
        currentState: input.states.current,
        newState: stateForRating(input.states, input.rating),
        rating: input.rating,
        answeredAtMillis: BigInt(Date.now()),
        millisecondsTaken: input.millisecondsTaken,
    });
    return fetchNextCard(input.deckName);
}
