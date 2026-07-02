// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { createSession, getScores, listQuestions } from "@generated/backend";

import { buildQuestionQueue } from "./practice-session";

import type { PageLoad } from "./$types";

export const load = (async () => {
    const [session, questionsResponse, scoresResponse] = await Promise.all([
        createSession({ source: "practice" }),
        listQuestions({ limit: 200, topicPrefix: "" }),
        getScores({}),
    ]);

    const questions = questionsResponse.questions;
    if (questions.length === 0) {
        throw new Error("No GRE practice questions in the performance database.");
    }

    const queue = buildQuestionQueue(questions, "all");

    return {
        sessionId: session.sessionId,
        questions,
        queue,
        memory: scoresResponse.memory!,
        performance: scoresResponse.performance!,
    };
}) satisfies PageLoad;
