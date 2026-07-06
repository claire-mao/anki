// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { COVERAGE_EXPLANATION } from "./coverage-presentation";

export const METHODOLOGY_PAGE_TITLE = "How GRE Atlas estimates your score";

export const METHODOLOGY_INTRO =
    "GRE Atlas turns flashcard reviews and practice questions into Memory, Performance, and Coverage signals, then combines them into a projected GRE score.";

export const METHODOLOGY_MEMORY: readonly string[] = [
    "When you review GRE flashcards, FSRS (Free Spaced Repetition Scheduler) estimates how well you will remember each card on test day.",
    "Each rating (Again, Hard, Good, or Easy) updates memory stability and when the card should appear again.",
    "Memory is average retrievability across studied cards, scaled 0–100. Retrievability is the chance you would recall a card right now.",
    "The range around Memory shows how much retrievability varies by topic. More cards and more even spread across topics narrow the band.",
    "Flashcards feed Memory. Practice feeds Performance. Readiness uses both.",
];

export const METHODOLOGY_PERFORMANCE: readonly string[] = [
    "Performance is practice accuracy: correct attempts divided by total attempts, shown as a percentage.",
    "All attempts count equally. A Wilson 95% interval shows uncertainty; the band narrows as you answer more.",
    "Practice updates Performance only. It does not reschedule flashcards or replace deck study.",
];

export const METHODOLOGY_COVERAGE: readonly string[] = [
    COVERAGE_EXPLANATION,
    "Topics reviewed is the share of catalog topics you have reviewed at least once.",
    "Readiness coverage blends Quant, Verbal, and AWA section percentages into one overall percentage.",
];

export const METHODOLOGY_READINESS: readonly string[] = [
    "Readiness blends Memory (45%), Performance (45%), and coverage (10%) into one projected GRE score.",
    "You need both flashcards and practice. Neither alone produces a full estimate.",
    "Snapshots track your projected score over time. After enough study days or new attempts, earlier predictions are checked against updates.",
    "Held-out checks use Brier score and mean absolute error. Good tracking keeps confidence high; poor tracking lowers it even when a score is shown.",
    "Review this track record on the Readiness page once enough checks have completed.",
];

export const METHODOLOGY_RANGE: readonly string[] = [
    "GRE Atlas shows a range because every estimate carries uncertainty.",
    "The band merges uncertainty from Memory and Performance into one projected score interval.",
    "Confidence (high, medium, or low) reflects how much evidence you have and how tight the interval is. Weak calibration history can lower confidence even when a score appears.",
];
