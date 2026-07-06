// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Pure presentation helpers for the in-page BrainLift Study reviewer.
// These keep the Svelte component thin and are unit-tested in isolation; no
// backend/proto types are imported here on purpose.

export type StudyQueueKind = "new" | "learning" | "review";

export interface StudyGradeButton {
    rating: number;
    /** Human-readable rating word shown as the primary button label. */
    name: string;
    /** Scheduler interval label (e.g. "<1m", "10d") shown as a secondary hint. */
    label: string;
    /** Keyboard shortcut digit (1-4). */
    shortcut: string;
    /** Design-system button variant used for styling. */
    variant: "again" | "hard" | "good" | "easy";
}

export interface StudyMetric {
    label: string;
    value: string;
}

export interface StudyCountsInput {
    dueNew: number;
    dueLearn: number;
    dueReview: number;
}

const GRADE_VARIANTS: StudyGradeButton["variant"][] = [
    "again",
    "hard",
    "good",
    "easy",
];

// Intuitive rating words, matching Anki's reviewer, keyed by rating index.
const GRADE_NAMES = ["Again", "Hard", "Good", "Easy"];

/** Rough per-card time budget used only for the "Estimated time" metric. */
const SECONDS_PER_CARD = 10;

/**
 * Build grade buttons from the scheduler's own next-state labels
 * (e.g. ["1m", "6m", "10m", "4d"]). Rating index matches Anki: 0=Again … 3=Easy.
 */
export function gradeButtonsFromLabels(labels: string[]): StudyGradeButton[] {
    return labels.map((label, index) => ({
        rating: index,
        name: GRADE_NAMES[index] ?? "Good",
        label,
        shortcut: String(index + 1),
        variant: GRADE_VARIANTS[index] ?? "good",
    }));
}

export function studyDueTotal(counts: StudyCountsInput): number {
    return counts.dueNew + counts.dueLearn + counts.dueReview;
}

export function formatEstimatedTime(dueTotal: number): string {
    if (dueTotal <= 0) {
        return "0 min";
    }
    const minutes = Math.max(1, Math.round((dueTotal * SECONDS_PER_CARD) / 60));
    return `${minutes} min`;
}

/** Top metrics row shown above the card, mirroring Practice's summary strip. */
export function studyMetrics(counts: StudyCountsInput): StudyMetric[] {
    const total = studyDueTotal(counts);
    return [
        { label: "Cards remaining", value: String(total) },
        { label: "New", value: String(counts.dueNew) },
        { label: "Learning", value: String(counts.dueLearn) },
        { label: "Review", value: String(counts.dueReview) },
        { label: "Estimated time", value: formatEstimatedTime(total) },
    ];
}

const QUEUE_BADGES: Record<StudyQueueKind, { label: string; tone: string }> = {
    new: { label: "New", tone: "new" },
    learning: { label: "Learning", tone: "learning" },
    review: { label: "Review", tone: "review" },
};

export function queueBadge(queue: string): { label: string; tone: string } {
    return QUEUE_BADGES[queue as StudyQueueKind] ?? QUEUE_BADGES.review;
}

/**
 * Progress across today's session. `remaining` is the live scheduler count of
 * cards still due; `reviewed` is how many the learner has graded this session.
 */
export function studyProgressPercent(reviewed: number, remaining: number): number {
    const total = reviewed + remaining;
    if (total <= 0) {
        return 100;
    }
    return Math.round((reviewed / total) * 100);
}

export function studyProgressLabel(reviewed: number, remaining: number): string {
    if (remaining <= 0) {
        return reviewed > 0 ? "All caught up" : "No cards due";
    }
    const noun = remaining === 1 ? "card" : "cards";
    return `${remaining} ${noun} to go`;
}

/** Recall-probability line for the intelligence sidebar; hides when unknown. */
export function formatRecallProbability(probability: number | null | undefined): string | null {
    if (probability === null || probability === undefined || Number.isNaN(probability)) {
        return null;
    }
    const clamped = Math.min(1, Math.max(0, probability));
    return `${Math.round(clamped * 100)}%`;
}
