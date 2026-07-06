// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { createSession, getRecentAttempts, getScores, getStudyPlan, listQuestions } from "@generated/backend";

import { buildQuestionQueue, PRACTICE_QUESTION_LIST_LIMIT } from "./practice-session";

import type { PageLoad } from "./$types";

export const load = (async ({ url }) => {
    const topicFilter = url.searchParams.get("topic")?.trim() ?? "";
    const [session, questionsResponse, scoresResponse, studyPlan, recentAttemptsResponse] =
        await Promise.all([
        createSession({ source: "practice" }),
        listQuestions({ limit: PRACTICE_QUESTION_LIST_LIMIT, topicPrefix: topicFilter }),
        getScores({}),
        getStudyPlan({ limit: 3 }),
        getRecentAttempts({ limit: 30 }),
    ]);

    const questions = questionsResponse.questions;
    if (questions.length === 0) {
        throw new Error("No GRE practice questions in the performance database.");
    }

    const queue = buildQuestionQueue(questions, "all", {
        topicFilter: topicFilter || undefined,
    });

    const focusTask = topicFilter
        ? studyPlan.dailyPlan?.tasks.find(
            (task) => task.id === "focus_topic" && task.topicId === topicFilter,
        )
        : undefined;

    return {
        sessionId: session.sessionId,
        questions,
        queue,
        topicFilter,
        focusTask,
        recentAttempts: recentAttemptsResponse.attempts,
        memory: scoresResponse.memory!,
        performance: scoresResponse.performance!,
        readiness: scoresResponse.readiness!,
    };
}) satisfies PageLoad;
