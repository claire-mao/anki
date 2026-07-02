// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformanceAttempt } from "@generated/anki/brainlift_pb";

import type { GreMetricSnapshot } from "./metric-snapshot";
import { capitalizeLabel } from "./summary-metrics";

export type MetricChangeEvidence = {
    id: string;
    label: string;
    positive: boolean;
};

export type MetricChangePresentation = {
    metricId: "readiness" | "estimatedGre" | "topicMastery" | "confidence";
    direction: "increased" | "decreased" | "changed";
    headline: string;
    deltaLabel?: string;
    evidence: MetricChangeEvidence[];
};

export type MetricChanges = Partial<
    Record<MetricChangePresentation["metricId"], MetricChangePresentation>
>;

const READINESS_DELTA = 0.5;
const GRE_DELTA = 1;
const MASTERY_DELTA = 1;
const COMPONENT_DELTA = 1;
const COVERAGE_DELTA = 0.01;
const RECENT_ACCURACY_DELTA = 5;
const TOPIC_PRACTICE_DELTA = 5;
const TOPIC_MEMORY_DELTA = 3;
const MAX_EVIDENCE = 4;

function directionFromDelta(delta: number): MetricChangePresentation["direction"] | null {
    if (delta > 0) {
        return "increased";
    }
    if (delta < 0) {
        return "decreased";
    }
    return null;
}

function formatSignedDelta(delta: number, suffix: string): string {
    const rounded = Math.round(delta * 10) / 10;
    const sign = rounded > 0 ? "+" : "";
    return `${sign}${rounded}${suffix}`;
}

function topicMasteryHeadline(
    direction: MetricChangePresentation["direction"] | null,
    masteredDelta: number,
): string {
    if (direction === "increased") {
        return "Topic mastery increased";
    }
    if (direction === "decreased") {
        return "Topic mastery decreased";
    }
    return masteredDelta >= 1 ? "Topic mastery increased" : "Topic mastery decreased";
}

function confidenceDirection(delta: number): MetricChangePresentation["direction"] {
    if (delta > 0) {
        return "increased";
    }
    if (delta < 0) {
        return "decreased";
    }
    return "changed";
}

function confidenceHeadline(direction: MetricChangePresentation["direction"]): string {
    if (direction === "increased") {
        return "Confidence increased";
    }
    if (direction === "decreased") {
        return "Confidence lowered";
    }
    return "Confidence changed";
}

function topicDisplayName(snapshot: GreMetricSnapshot, topicId: string): string {
    return snapshot.topics[topicId]?.displayName || topicId;
}

function incorrectTopicsSince(
    attempts: PerformanceAttempt[] | undefined,
    sinceMillis: number,
): string[] {
    if (!attempts?.length) {
        return [];
    }
    const sinceSecs = Math.floor(sinceMillis / 1000);
    const topics = new Set<string>();
    for (const attempt of attempts) {
        if (!attempt.correct && attempt.answeredAtSecs > sinceSecs) {
            topics.add(attempt.topic);
        }
    }
    return [...topics];
}

function topicPracticeEvidence(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
    direction: "up" | "down",
): MetricChangeEvidence[] {
    const evidence: MetricChangeEvidence[] = [];
    for (const [topicId, currentTopic] of Object.entries(current.topics)) {
        const previousTopic = previous.topics[topicId];
        if (
            currentTopic.practiceAccuracy === undefined ||
            previousTopic?.practiceAccuracy === undefined
        ) {
            continue;
        }
        const delta = currentTopic.practiceAccuracy - previousTopic.practiceAccuracy;
        if (direction === "up" && delta >= TOPIC_PRACTICE_DELTA) {
            evidence.push({
                id: `topic-up-${topicId}`,
                label: `${currentTopic.displayName} improved`,
                positive: true,
            });
        } else if (direction === "down" && delta <= -TOPIC_PRACTICE_DELTA) {
            evidence.push({
                id: `topic-down-${topicId}`,
                label: `${currentTopic.displayName} errors`,
                positive: false,
            });
        }
    }
    return evidence.sort((a, b) => a.label.localeCompare(b.label));
}

function topicMemoryEvidence(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
    direction: "up" | "down",
): MetricChangeEvidence[] {
    const evidence: MetricChangeEvidence[] = [];
    for (const [topicId, currentTopic] of Object.entries(current.topics)) {
        const previousTopic = previous.topics[topicId];
        if (!previousTopic) {
            continue;
        }
        const delta = currentTopic.retrievability - previousTopic.retrievability;
        if (direction === "up" && delta >= TOPIC_MEMORY_DELTA) {
            evidence.push({
                id: `memory-up-${topicId}`,
                label: `${currentTopic.displayName} improved`,
                positive: true,
            });
        }
    }
    return evidence.sort((a, b) => a.label.localeCompare(b.label));
}

function componentEvidence(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
    direction: "up" | "down",
    recentAttempts?: PerformanceAttempt[],
): MetricChangeEvidence[] {
    const evidence: MetricChangeEvidence[] = [];
    const masteredDelta = current.masteredCards - previous.masteredCards;
    const studiedDelta = current.studiedCards - previous.studiedCards;
    const memoryDelta =
        current.memoryValue !== undefined && previous.memoryValue !== undefined
            ? current.memoryValue - previous.memoryValue
            : 0;
    const performanceDelta =
        current.performanceValue !== undefined && previous.performanceValue !== undefined
            ? current.performanceValue - previous.performanceValue
            : 0;
    const coverageDelta = current.coverageRatio - previous.coverageRatio;
    const recentDelta =
        current.recentAccuracy !== undefined && previous.recentAccuracy !== undefined
            ? current.recentAccuracy - previous.recentAccuracy
            : 0;

    if (direction === "up") {
        if (masteredDelta >= 1) {
            evidence.push({
                id: "mastered-cards",
                label: `${masteredDelta} card${masteredDelta === 1 ? "" : "s"} matured`,
                positive: true,
            });
        }
        if (memoryDelta >= COMPONENT_DELTA) {
            evidence.push({
                id: "memory-up",
                label: "Memory retention improved",
                positive: true,
            });
        }
        if (performanceDelta >= COMPONENT_DELTA) {
            evidence.push({
                id: "performance-up",
                label: "Practice accuracy improved",
                positive: true,
            });
        }
        if (coverageDelta >= COVERAGE_DELTA) {
            evidence.push({
                id: "coverage-up",
                label: "Coverage increased",
                positive: true,
            });
        }
        if (studiedDelta >= 1 && masteredDelta <= 0) {
            evidence.push({
                id: "studied-cards",
                label: `${studiedDelta} new card${studiedDelta === 1 ? "" : "s"} studied`,
                positive: true,
            });
        }
        evidence.push(...topicPracticeEvidence(previous, current, "up"));
        evidence.push(...topicMemoryEvidence(previous, current, "up"));
    } else {
        if (memoryDelta <= -COMPONENT_DELTA) {
            evidence.push({
                id: "memory-down",
                label: "Memory decay",
                positive: false,
            });
        }
        if (performanceDelta <= -COMPONENT_DELTA) {
            evidence.push({
                id: "performance-down",
                label: "Lower practice accuracy",
                positive: false,
            });
        }
        if (recentDelta <= -RECENT_ACCURACY_DELTA) {
            evidence.push({
                id: "recent-accuracy",
                label: "Lower recent accuracy",
                positive: false,
            });
        }
        if (coverageDelta <= -COVERAGE_DELTA) {
            evidence.push({
                id: "coverage-down",
                label: "Coverage decreased",
                positive: false,
            });
        }
        evidence.push(...topicPracticeEvidence(previous, current, "down"));
        for (const topicId of incorrectTopicsSince(recentAttempts, previous.savedAtMillis)) {
            const label = `${topicDisplayName(current, topicId)} errors`;
            if (!evidence.some((item) => item.label === label)) {
                evidence.push({
                    id: `attempt-errors-${topicId}`,
                    label,
                    positive: false,
                });
            }
        }
    }

    const seen = new Set<string>();
    return evidence.filter((item) => {
        if (seen.has(item.label)) {
            return false;
        }
        seen.add(item.label);
        return true;
    });
}

function limitEvidence(evidence: MetricChangeEvidence[]): MetricChangeEvidence[] {
    return evidence.slice(0, MAX_EVIDENCE);
}

function presentReadinessChange(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
    recentAttempts?: PerformanceAttempt[],
): MetricChangePresentation | null {
    if (!previous.readinessUnlocked && !current.readinessUnlocked) {
        return null;
    }
    if (!previous.readinessUnlocked && current.readinessUnlocked) {
        return {
            metricId: "readiness",
            direction: "changed",
            headline: "Readiness unlocked",
            evidence: limitEvidence(componentEvidence(previous, current, "up", recentAttempts)),
        };
    }
    if (
        previous.readinessScore === undefined ||
        current.readinessScore === undefined ||
        !current.readinessUnlocked
    ) {
        return null;
    }

    const delta = current.readinessScore - previous.readinessScore;
    const direction = directionFromDelta(delta);
    if (!direction || Math.abs(delta) < READINESS_DELTA) {
        return null;
    }

    return {
        metricId: "readiness",
        direction,
        headline: direction === "increased" ? "Readiness increased" : "Readiness decreased",
        deltaLabel: formatSignedDelta(delta, " pts"),
        evidence: limitEvidence(
            componentEvidence(previous, current, direction === "increased" ? "up" : "down", recentAttempts),
        ),
    };
}

function presentEstimatedGreChange(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
    recentAttempts?: PerformanceAttempt[],
): MetricChangePresentation | null {
    if (!previous.estimatedGreUnlocked && !current.estimatedGreUnlocked) {
        return null;
    }
    if (!previous.estimatedGreUnlocked && current.estimatedGreUnlocked) {
        return {
            metricId: "estimatedGre",
            direction: "changed",
            headline: "Estimated GRE unlocked",
            evidence: limitEvidence(componentEvidence(previous, current, "up", recentAttempts)),
        };
    }
    if (
        previous.estimatedGreScore === undefined ||
        current.estimatedGreScore === undefined
    ) {
        return null;
    }

    const delta = current.estimatedGreScore - previous.estimatedGreScore;
    const direction = directionFromDelta(delta);
    if (!direction || Math.abs(delta) < GRE_DELTA) {
        return null;
    }

    return {
        metricId: "estimatedGre",
        direction,
        headline: direction === "increased" ? "Estimated GRE increased" : "Estimated GRE dropped",
        deltaLabel: formatSignedDelta(delta, ""),
        evidence: limitEvidence(
            componentEvidence(previous, current, direction === "increased" ? "up" : "down", recentAttempts),
        ),
    };
}

function presentTopicMasteryChange(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
): MetricChangePresentation | null {
    if (!previous.topicMasteryUnlocked && !current.topicMasteryUnlocked) {
        return null;
    }
    if (!previous.topicMasteryUnlocked && current.topicMasteryUnlocked) {
        const evidence: MetricChangeEvidence[] = [];
        const masteredDelta = current.masteredCards - previous.masteredCards;
        if (masteredDelta >= 1) {
            evidence.push({
                id: "mastered-cards",
                label: `${masteredDelta} card${masteredDelta === 1 ? "" : "s"} matured`,
                positive: true,
            });
        }
        return {
            metricId: "topicMastery",
            direction: "changed",
            headline: "Topic mastery unlocked",
            evidence: limitEvidence(evidence),
        };
    }
    if (
        previous.topicMasteryPercent === undefined ||
        current.topicMasteryPercent === undefined
    ) {
        return null;
    }

    const delta = current.topicMasteryPercent - previous.topicMasteryPercent;
    const direction = directionFromDelta(delta);
    const masteredDelta = current.masteredCards - previous.masteredCards;
    if (
        (!direction || Math.abs(delta) < MASTERY_DELTA) &&
        Math.abs(masteredDelta) < 1
    ) {
        return null;
    }

    const evidence: MetricChangeEvidence[] = [];
    if (masteredDelta >= 1) {
        evidence.push({
            id: "mastered-cards",
            label: `${masteredDelta} card${masteredDelta === 1 ? "" : "s"} matured`,
            positive: true,
        });
    } else if (masteredDelta <= -1) {
        evidence.push({
            id: "mastered-cards-down",
            label: `${Math.abs(masteredDelta)} fewer mastered cards`,
            positive: false,
        });
    }
    if (direction === "increased") {
        evidence.push(...topicMemoryEvidence(previous, current, "up"));
    } else if (direction === "decreased") {
        if (
            current.memoryValue !== undefined &&
            previous.memoryValue !== undefined &&
            current.memoryValue - previous.memoryValue <= -COMPONENT_DELTA
        ) {
            evidence.push({
                id: "memory-down",
                label: "Memory decay",
                positive: false,
            });
        }
    }

    const headline = topicMasteryHeadline(direction, masteredDelta);

    return {
        metricId: "topicMastery",
        direction: direction ?? (masteredDelta >= 0 ? "increased" : "decreased"),
        headline,
        deltaLabel:
            direction && Math.abs(delta) >= MASTERY_DELTA
                ? formatSignedDelta(delta, " pts")
                : undefined,
        evidence: limitEvidence(evidence),
    };
}

function presentConfidenceChange(
    previous: GreMetricSnapshot,
    current: GreMetricSnapshot,
): MetricChangePresentation | null {
    const previousLevel = previous.confidenceLevel.trim().toLowerCase();
    const currentLevel = current.confidenceLevel.trim().toLowerCase();
    if (!currentLevel || previousLevel === currentLevel) {
        return null;
    }

    const rank = (level: string): number => {
        if (level.includes("high")) {
            return 3;
        }
        if (level.includes("medium") || level.includes("med")) {
            return 2;
        }
        if (level.includes("low")) {
            return 1;
        }
        return 0;
    };

    const delta = rank(currentLevel) - rank(previousLevel);
    const direction = confidenceDirection(delta);
    const evidence: MetricChangeEvidence[] = [];

    for (const [id, req] of Object.entries(current.requirements)) {
        const previousReq = previous.requirements[id];
        if (req.met && !previousReq?.met) {
            evidence.push({
                id: `req-met-${id}`,
                label: `${req.label} met`,
                positive: true,
            });
        } else if (!req.met && previousReq?.met) {
            evidence.push({
                id: `req-unmet-${id}`,
                label: `${req.label} no longer met`,
                positive: false,
            });
        }
    }

    if (
        direction === "decreased" &&
        current.calibrationNote &&
        ((previous.calibrationWellCalibrated && !current.calibrationWellCalibrated) ||
            current.calibrationNote !== previous.calibrationNote)
    ) {
        evidence.push({
            id: "calibration-note",
            label: current.calibrationNote,
            positive: false,
        });
    } else if (
        direction === "increased" &&
        current.calibrationWellCalibrated &&
        !previous.calibrationWellCalibrated &&
        current.calibrationNote
    ) {
        evidence.push({
            id: "calibration-note",
            label: current.calibrationNote,
            positive: true,
        });
    }

    return {
        metricId: "confidence",
        direction,
        headline: confidenceHeadline(direction),
        deltaLabel: `${capitalizeLabel(previous.confidenceLevel || "unknown")} → ${capitalizeLabel(current.confidenceLevel)}`,
        evidence: limitEvidence(evidence),
    };
}

export function presentMetricChanges(
    previous: GreMetricSnapshot | null,
    current: GreMetricSnapshot,
    recentAttempts?: PerformanceAttempt[],
): MetricChanges {
    if (!previous) {
        return {};
    }

    const changes: MetricChanges = {};
    const readiness = presentReadinessChange(previous, current, recentAttempts);
    const estimatedGre = presentEstimatedGreChange(previous, current, recentAttempts);
    const topicMastery = presentTopicMasteryChange(previous, current);
    const confidence = presentConfidenceChange(previous, current);

    if (readiness) {
        changes.readiness = readiness;
    }
    if (estimatedGre) {
        changes.estimatedGre = estimatedGre;
    }
    if (topicMastery) {
        changes.topicMastery = topicMastery;
    }
    if (confidence) {
        changes.confidence = confidence;
    }

    return changes;
}
