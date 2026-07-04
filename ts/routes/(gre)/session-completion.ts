// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DashboardTopicInsight } from "@generated/anki/brainlift_pb";

import { formatPercent } from "./score-format";
import {
    greNavAction,
    greNavItem,
    GRE_CTA_PRACTICE,
    GRE_CTA_REVIEW,
    GRE_CTA_STUDY_PLAN,
    studyPlanNavAction,
    type GreNavAction,
} from "./gre-navigation";

export type SessionAttemptRecord = {
    topic: string;
    correct: boolean;
};

export type SessionCompletionRow = {
    label: string;
    value: string;
};

export type SessionCompletionSummary = {
    headline: string;
    subline: string;
    rows: SessionCompletionRow[];
    nextAction: GreNavAction;
    nextActionDetail: string;
    secondaryAction?: GreNavAction;
};

type TopicStats = Map<string, { correct: number; total: number }>;

function topicStats(attempts: SessionAttemptRecord[]): TopicStats {
    const stats: TopicStats = new Map();
    for (const attempt of attempts) {
        const topic = attempt.topic.trim() || "Unknown topic";
        const current = stats.get(topic) ?? { correct: 0, total: 0 };
        stats.set(topic, {
            correct: current.correct + (attempt.correct ? 1 : 0),
            total: current.total + 1,
        });
    }
    return stats;
}

function strongestFromStats(stats: TopicStats): string | null {
    let best: { topic: string; accuracy: number; correct: number } | null = null;
    for (const [topic, value] of stats) {
        const accuracy = value.correct / value.total;
        if (
            !best
            || accuracy > best.accuracy
            || (accuracy === best.accuracy && value.correct > best.correct)
        ) {
            best = { topic, accuracy, correct: value.correct };
        }
    }
    return best?.topic ?? null;
}

function weakestFromStats(stats: TopicStats, strongest: string | null): string | null {
    if (stats.size === 0) {
        return null;
    }
    let worst: { topic: string; accuracy: number; total: number } | null = null;
    for (const [topic, value] of stats) {
        const accuracy = value.correct / value.total;
        if (
            !worst
            || accuracy < worst.accuracy
            || (accuracy === worst.accuracy && value.total > worst.total)
        ) {
            worst = { topic, accuracy, total: value.total };
        }
    }
    if (!worst) {
        return null;
    }
    if (strongest && worst.topic === strongest && stats.size === 1) {
        return worst.accuracy < 1 ? worst.topic : null;
    }
    if (strongest && worst.topic === strongest) {
        return null;
    }
    return worst.topic;
}

function topicScore(topic: DashboardTopicInsight): number {
    if (topic.practiceAccuracy !== undefined) {
        return topic.practiceAccuracy;
    }
    if (topic.memoryScore !== undefined) {
        return topic.memoryScore;
    }
    return topic.covered ? 0.5 : 0;
}

function strongestDashboardTopic(
    recommendedTopics: DashboardTopicInsight[],
    weakTopics: DashboardTopicInsight[],
): string | null {
    const candidates = topicsWithStudyEvidence([...recommendedTopics, ...weakTopics]);
    if (candidates.length === 0) {
        return null;
    }
    const sorted = [...candidates].sort((a, b) => topicScore(b) - topicScore(a));
    return sorted[0]?.displayName ?? null;
}

function topicsWithStudyEvidence(
    topics: DashboardTopicInsight[],
): DashboardTopicInsight[] {
    return topics.filter(
        (topic) =>
            topic.studiedCards > 0
            || topic.memoryScore !== undefined
            || topic.practiceAccuracy !== undefined,
    );
}

function lacksGreFlashcardEvidence(studiedCards: number, coveredLeafCount: number): boolean {
    return studiedCards === 0 && coveredLeafCount === 0;
}

export function buildPracticeSessionSummary(
    attempts: SessionAttemptRecord[],
): SessionCompletionSummary {
    const total = attempts.length;
    const correct = attempts.filter((attempt) => attempt.correct).length;
    const stats = topicStats(attempts);
    const strongest = strongestFromStats(stats);
    const weakest = weakestFromStats(stats, strongest);

    const rows: SessionCompletionRow[] = [
        {
            label: "Questions answered",
            value: String(total),
        },
    ];

    if (total > 0) {
        rows.push({
            label: "Accuracy",
            value: formatPercent((correct / total) * 100),
        });
    }
    if (strongest) {
        rows.push({ label: "Strongest topic", value: strongest });
    }
    if (weakest) {
        rows.push({ label: "Focus next", value: weakest });
    }

    let nextAction = greNavAction(greNavItem("practice"));
    nextAction.label = "Practice again";
    let nextActionDetail = "Run another short set while this material is fresh.";

    if (weakest) {
        nextActionDetail = `Your weakest area this session was ${weakest}. Another short set there will help it stick.`;
    } else if (total > 0 && correct === total) {
        nextAction = greNavAction(greNavItem("study"));
        nextAction.label = GRE_CTA_REVIEW;
        nextActionDetail = "Strong accuracy — reinforce recall with a quick flashcard review.";
    }

    return {
        headline: "Session complete",
        subline: "Here's how this set went.",
        rows,
        nextAction,
        nextActionDetail,
        secondaryAction: greNavAction(greNavItem("studyPlan")),
    };
}

export function buildStudyCaughtUpSummary(input: {
    weakTopics: DashboardTopicInsight[];
    recommendedTopics: DashboardTopicInsight[];
    dueTotal: number;
    deckName: string;
    studiedCards: number;
    coveredLeafCount: number;
}): SessionCompletionSummary {
    if (lacksGreFlashcardEvidence(input.studiedCards, input.coveredLeafCount)) {
        const startStudy = greNavAction(greNavItem("study"));
        startStudy.label = GRE_CTA_REVIEW;
        const practiceAction = greNavAction(greNavItem("practice"));
        practiceAction.label = GRE_CTA_PRACTICE;
        return {
            headline: "Your GRE flashcards are ready",
            subline:
                "GRE Atlas includes built-in flashcards — no Anki import needed.",
            rows: [
                {
                    label: "Flashcards reviewed",
                    value: "0",
                },
            ],
            nextAction: startStudy,
            nextActionDetail:
                "Tap Start review to open your first cards and begin building memory evidence.",
            secondaryAction: practiceAction,
        };
    }

    const studiedWeakTopics = topicsWithStudyEvidence(input.weakTopics);
    const weakest = studiedWeakTopics[0]?.displayName ?? null;
    const strongest = strongestDashboardTopic(
        input.recommendedTopics,
        input.weakTopics,
    );

    const rows: SessionCompletionRow[] = [
        {
            label: "Cards due now",
            value: String(input.dueTotal),
        },
        {
            label: "Flashcards reviewed",
            value: String(input.studiedCards),
        },
    ];
    if (strongest) {
        rows.push({ label: "Strongest area", value: strongest });
    }
    if (weakest) {
        rows.push({ label: "Focus next", value: weakest });
    } else if (input.studiedCards === 0) {
        rows.push({
            label: "Flashcard history",
            value: "Not enough yet",
        });
    }

    const nextAction = studyPlanNavAction(GRE_CTA_STUDY_PLAN);
    let nextActionDetail =
        "Nothing due right now. Open your study plan to see what to add or review next.";

    if (weakest) {
        nextActionDetail = `Nothing due right now. Your study plan suggests more flashcards on ${weakest}.`;
    } else if (input.studiedCards > 0) {
        nextActionDetail =
            "You're caught up for now. Check your study plan for what to work on next.";
    }

    const secondaryAction = greNavAction(greNavItem("practice"));
    secondaryAction.label = GRE_CTA_PRACTICE;

    return {
        headline: input.dueTotal === 0 ? "Review complete" : "Session complete",
        subline:
            input.dueTotal === 0
                ? "You're caught up on flashcards due right now."
                : "Nice pause point — you can pick up remaining cards later.",
        rows,
        nextAction,
        nextActionDetail,
        secondaryAction,
    };
}
