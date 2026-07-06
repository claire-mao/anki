// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    AbstentionRequirement,
    DashboardCoverage,
    MemoryScore,
    PerformanceAttempt,
    PerformanceScore,
    ReadinessCalibrationStats,
    ReadinessScore,
} from "@generated/anki/brainlift_pb";

import {
    coverageAwareReadinessUnlocked,
    coverageBlocksReadiness,
    coverageReadinessReason,
    sectionCoverageRatio,
} from "../coverage-presentation";
import { formatCalibrationChecksProgress } from "../calibration-presentation";
import { MIN_COVERAGE_PERCENT, MIN_STUDIED_CARDS } from "../empty-states";
import { unmetRequirements } from "../summary-metrics";

export const HONEST_REPORTING_TITLE = "Honest Reporting";
export const HONEST_REPORTING_LEAD =
    "Active limitations detected from your current study evidence. GRE Atlas abstains rather than guessing.";
export const HONEST_REPORTING_ALL_CLEAR =
    "No active limitations detected. Keep studying — uncertainty can return if evidence thins out.";

export const MIN_AWA_TOPIC_COVERAGE_RATIO = 0.5;
export const MIN_AWA_PRACTICE_ATTEMPTS = 3;

export type HonestReportingLimitation = {
    id: string;
    title: string;
    why: string;
    missingEvidence: string;
    howToImprove: string;
};

export type HonestReportingPresentation = {
    limitations: HonestReportingLimitation[];
    allClear: boolean;
    readinessScoreVisible: boolean;
};

export type HonestReportingInput = {
    memory: MemoryScore;
    performance: PerformanceScore;
    readiness: ReadinessScore;
    coverage: DashboardCoverage;
    calibration: ReadinessCalibrationStats;
    aiEnabled?: string;
    recentAttempts?: PerformanceAttempt[];
};

function requirementById(
    requirements: AbstentionRequirement[],
    id: string,
): AbstentionRequirement | undefined {
    return requirements.find((req) => req.id === id);
}

function awaSection(coverage: DashboardCoverage) {
    return coverage.sections.find((section) => section.section === "awa");
}

export function countAwaPracticeAttempts(attempts: PerformanceAttempt[]): number {
    return attempts.filter((attempt) => attempt.topic.startsWith("gre::awa::")).length;
}

function limitationReadinessAbstained(input: HonestReportingInput): HonestReportingLimitation | null {
    if (coverageAwareReadinessUnlocked(input.readiness, input.coverage)) {
        return null;
    }

    const unmet = unmetRequirements(input.readiness.abstentionRequirements);
    const coverageBlocked = coverageBlocksReadiness(input.coverage);
    const why = coverageBlocked
        ? coverageReadinessReason(input.coverage)
        : input.readiness.abstainReason
            || unmet.map((req) => `${req.label}: ${req.status}`).join("; ")
            || "Readiness gates are not satisfied yet.";

    const missing = unmet.length
        ? unmet.map((req) => `${req.label} — ${req.status}`).join("; ")
        : coverageBlocked
            ? "Enough GRE topic coverage for a composite readiness score."
            : "Combined memory, practice, and coverage evidence for readiness.";

    const improve = unmet
        .map((req) => req.nextStep || req.status)
        .filter((step, index, steps) => step && steps.indexOf(step) === index)
        .join(" ");

    return {
        id: "readiness_abstained",
        title: "Readiness abstained",
        why,
        missingEvidence: missing,
        howToImprove: improve
            || "Review flashcards, answer practice questions, and expand GRE topic coverage until all readiness gates clear.",
    };
}

function limitationLowTopicCoverage(input: HonestReportingInput): HonestReportingLimitation | null {
    const coverageReq = requirementById(
        input.memory.abstentionRequirements,
        "topic_coverage",
    );
    if (coverageReq?.met && input.coverage.readinessAvailable) {
        return null;
    }

    const pct = Math.round(input.memory.coverageRatio * 100);
    return {
        id: "low_topic_coverage",
        title: "Low topic coverage",
        why: coverageReq?.status
            || coverageReadinessReason(input.coverage)
            || `Only ${pct}% of the GRE catalog has flashcard review evidence.`,
        missingEvidence:
            "Reviewed flashcards across enough distinct GRE topics to reach the coverage gate "
            + `(minimum ${MIN_COVERAGE_PERCENT}% exam-weighted catalog coverage).`,
        howToImprove:
            coverageReq?.nextStep
            || "Review cards tagged with GRE topics you have not studied yet, starting with high-weight Quant and Verbal areas.",
    };
}

function limitationTooFewReviews(input: HonestReportingInput): HonestReportingLimitation | null {
    const studiedReq = requirementById(
        input.memory.abstentionRequirements,
        "studied_cards",
    );
    if (studiedReq?.met) {
        return null;
    }

    return {
        id: "too_few_reviews",
        title: "Too few reviews",
        why: studiedReq?.status
            || `${input.memory.studiedCards} reviewed GRE flashcards so far.`,
        missingEvidence:
            `At least ${MIN_STUDIED_CARDS} distinct GRE flashcards with at least one review each.`,
        howToImprove:
            studiedReq?.nextStep
            || `Keep reviewing the GRE Atlas deck until ${MIN_STUDIED_CARDS} cards have been studied at least once.`,
    };
}

function limitationInsufficientCalibration(
    input: HonestReportingInput,
): HonestReportingLimitation | null {
    if (input.calibration.sufficientData) {
        return null;
    }

    const progress = formatCalibrationChecksProgress(input.calibration);
    return {
        id: "insufficient_calibration",
        title: "Insufficient calibration data",
        why: input.calibration.assessment
            || `Past readiness estimates have not completed enough held-out checks (${progress}).`,
        missingEvidence:
            "Five resolved held-out readiness predictions with observed outcomes "
            + "(predictions age at least 3 days or accumulate follow-up practice).",
        howToImprove:
            "Keep studying and practicing so GRE Atlas can compare past score estimates to your later results.",
    };
}

function limitationSparseAwaData(input: HonestReportingInput): HonestReportingLimitation | null {
    const awa = awaSection(input.coverage);
    if (!awa || awa.catalogLeafCount === 0) {
        return null;
    }

    const topicRatio = sectionCoverageRatio(awa.coveredLeafCount, awa.catalogLeafCount);
    const awaAttempts = countAwaPracticeAttempts(input.recentAttempts ?? []);
    const sparseCoverage = topicRatio < MIN_AWA_TOPIC_COVERAGE_RATIO;
    const sparsePractice = awaAttempts < MIN_AWA_PRACTICE_ATTEMPTS;

    if (!sparseCoverage && !sparsePractice) {
        return null;
    }

    const coveragePct = Math.round(topicRatio * 100);
    const whyParts: string[] = [];
    if (sparseCoverage) {
        whyParts.push(
            `${coveragePct}% of AWA catalog topics reviewed (${awa.coveredLeafCount}/${awa.catalogLeafCount}).`,
        );
    }
    if (sparsePractice) {
        whyParts.push(
            `${awaAttempts} AWA practice attempt${awaAttempts === 1 ? "" : "s"} recorded `
                + `(minimum ${MIN_AWA_PRACTICE_ATTEMPTS}).`,
        );
    }

    const missingParts: string[] = [];
    if (sparseCoverage) {
        missingParts.push("Flashcard reviews on Analyze an Issue and Analyze an Argument topics.");
    }
    if (sparsePractice) {
        missingParts.push("Essay-style AWA practice attempts.");
    }

    const improveParts: string[] = [];
    if (sparseCoverage) {
        improveParts.push("Review AWA-tagged flashcards in the GRE Atlas deck.");
    }
    if (sparsePractice) {
        improveParts.push("Answer Analyze an Issue and Analyze an Argument practice prompts.");
    }

    return {
        id: "sparse_awa_data",
        title: "Sparse AWA data",
        why: whyParts.join(" "),
        missingEvidence: missingParts.join(" "),
        howToImprove: improveParts.join(" "),
    };
}

function limitationAiDisabled(aiEnabled: string | undefined): HonestReportingLimitation | null {
    const normalized = aiEnabled?.trim().toLowerCase();
    if (!normalized || normalized === "enabled" || normalized === "unknown") {
        return null;
    }
    if (!normalized.includes("disabled") && normalized !== "off") {
        return null;
    }

    return {
        id: "ai_disabled",
        title: "AI disabled",
        why: "GRE Atlas is running in offline mode without the optional AI pipeline.",
        missingEvidence:
            "Live AI-generated explanations and question generation are unavailable in this session.",
        howToImprove:
            "Enable AI in your environment to use generated explanations. Memory, performance, and readiness scores still work without AI.",
    };
}

function limitationLowConfidence(input: HonestReportingInput): HonestReportingLimitation | null {
    if (!coverageAwareReadinessUnlocked(input.readiness, input.coverage)) {
        return null;
    }
    if (input.readiness.confidenceLevel !== "low") {
        return null;
    }

    const range =
        input.readiness.projectedScoreLow !== undefined
        && input.readiness.projectedScoreHigh !== undefined
            ? `${Math.round(input.readiness.projectedScoreLow)}–${Math.round(input.readiness.projectedScoreHigh)}`
            : "wide";

    return {
        id: "low_confidence",
        title: "Low confidence",
        why: input.readiness.calibrationNote
            || `Readiness interval is ${range} because evidence is still thin or past estimates have not stabilized.`,
        missingEvidence:
            "More reviewed cards, broader topic coverage, and additional practice attempts to narrow the readiness interval.",
        howToImprove:
            input.readiness.calibrationNote
            || "Continue daily review and practice until the readiness range tightens and confidence improves.",
    };
}

const LIMITATION_BUILDERS: ((input: HonestReportingInput) => HonestReportingLimitation | null)[] = [
    limitationReadinessAbstained,
    limitationLowTopicCoverage,
    limitationTooFewReviews,
    limitationInsufficientCalibration,
    limitationSparseAwaData,
    (input) => limitationAiDisabled(input.aiEnabled),
    limitationLowConfidence,
];

export function presentHonestReporting(input: HonestReportingInput): HonestReportingPresentation {
    const seen = new Set<string>();
    const limitations: HonestReportingLimitation[] = [];
    for (const build of LIMITATION_BUILDERS) {
        const limitation = build(input);
        if (!limitation || seen.has(limitation.id)) {
            continue;
        }
        seen.add(limitation.id);
        limitations.push(limitation);
    }

    return {
        limitations,
        allClear: limitations.length === 0,
        readinessScoreVisible: coverageAwareReadinessUnlocked(input.readiness, input.coverage),
    };
}
