// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { TopicMasteryEntry } from "@generated/anki/stats_pb";

/** A topic has started once at least one flashcard in it has been reviewed. */
export function isTopicMasteryStarted(topic: TopicMasteryEntry): boolean {
    return topic.studiedCards > 0;
}

/**
 * Multi-evidence mastery for progress charts (0–100). This is the backend
 * `displayMastery`, which blends FSRS recall, practice accuracy, and recent
 * trend over whichever signals a topic has (see topic_mastery_display.rs).
 */
export function topicDisplayMasteryPercent(topic: TopicMasteryEntry): number | undefined {
    if (!isTopicMasteryStarted(topic)) {
        return undefined;
    }
    return topic.displayMastery * 100;
}

/** Confidence label ("Low"/"Moderate"/"High") shown separately from mastery. */
export function topicMasteryConfidenceLabel(topic: TopicMasteryEntry): string | undefined {
    if (!isTopicMasteryStarted(topic)) {
        return undefined;
    }
    return topic.confidenceLabel || "Low";
}

export function topicMasteryConfidencePercent(topic: TopicMasteryEntry): number | undefined {
    if (!isTopicMasteryStarted(topic)) {
        return undefined;
    }
    return Math.round(topic.masteryConfidence * 100);
}

/** Short explanation shown under the Topic mastery section heading. */
export function topicMasterySectionExplanation(): string {
    return "Mastery blends FSRS recall, practice accuracy, and recent trend. Confidence is shown separately and stays low until a topic has enough reviews.";
}

/** Shown when studied topics cluster at similar recall levels. */
export function topicMasteryClusterNote(topics: TopicMasteryEntry[]): string | undefined {
    const studied = topics.filter(isTopicMasteryStarted);
    if (studied.length < 3) {
        return undefined;
    }
    const percents = studied
        .map(topicDisplayMasteryPercent)
        .filter((percent): percent is number => percent !== undefined);
    if (percents.length < 3) {
        return undefined;
    }
    const min = Math.min(...percents);
    const max = Math.max(...percents);
    if (max - min <= 5) {
        return "Similar percentages usually mean recently reviewed cards at similar retention levels.";
    }
    return undefined;
}

/** Hover tooltip for a topic row in the mastery chart. */
export function topicMasteryRowTooltip(topic: TopicMasteryEntry): string {
    if (!isTopicMasteryStarted(topic)) {
        return "Not started — no reviewed flashcards in this topic yet.";
    }
    const mastery = Math.round(topic.displayMastery * 100);
    const confidence = topic.confidenceLabel || "Low";
    const evidence = topic.evidenceCount;
    const evidenceLabel = evidence === 1 ? "evidence item" : "evidence items";
    let tooltip = `${mastery}% mastery · ${confidence} confidence · ${evidence} ${evidenceLabel}`;
    if (topic.practiceAttempts > 0) {
        tooltip += ` · practice ${Math.round(topic.practiceAccuracy * 100)}%`;
    }
    return tooltip;
}

export interface TopicMasteryVerificationRow {
    topicId: string;
    displayName: string;
    studiedCards: number;
    displayPercent: number | undefined;
    confidenceLabel: string | undefined;
    confidencePercent: number | undefined;
}

/** Maps backend topic entries to verification rows (display must match backend). */
export function topicMasteryVerificationRows(
    topics: TopicMasteryEntry[],
): TopicMasteryVerificationRow[] {
    return topics.map((topic) => ({
        topicId: topic.topicId,
        displayName: topic.displayName,
        studiedCards: topic.studiedCards,
        displayPercent: topicDisplayMasteryPercent(topic),
        confidenceLabel: topicMasteryConfidenceLabel(topic),
        confidencePercent: topicMasteryConfidencePercent(topic),
    }));
}
