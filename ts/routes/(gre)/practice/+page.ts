// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { createSession, getScores, listQuestions } from "@generated/backend";

import type { PageLoad } from "./$types";

function shuffle<T>(items: T[]): T[] {
    const copy = [...items];
    for (let i = copy.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [copy[i], copy[j]] = [copy[j], copy[i]];
    }
    return copy;
}

export const load = (async () => {
    const [session, questionsResponse, scoresResponse] = await Promise.all([
        createSession({ source: "practice" }),
        listQuestions({ limit: 100, topicPrefix: "" }),
        getScores({}),
    ]);

    const questions = shuffle(questionsResponse.questions);
    if (questions.length === 0) {
        throw new Error("No GRE practice questions available.");
    }

    return {
        sessionId: session.sessionId,
        questions,
        performance: scoresResponse.performance!,
    };
}) satisfies PageLoad;
