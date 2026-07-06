// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { CalibrationChartPoint } from "./memory-model-presentation";
import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";

export type EvalReportCalibration = {
    heldOutCount: number;
    sufficientData: boolean;
    wellCalibrated: boolean;
    assessment: string;
    brierScore: number | undefined;
    meanAbsoluteError: number | undefined;
    bins: CalibrationChartPoint[];
};

export type EvalReportAbstentionRequirement = {
    id: string;
    label: string;
    status: string;
    met: boolean;
};

export type EvalReportAbstention = {
    abstentionRate: number;
    memoryAbstaining: boolean;
    performanceAbstaining: boolean;
    readinessAbstaining: boolean;
    unmetRequirements: EvalReportAbstentionRequirement[];
};

export type EvalReportAblationPolicy = {
    policyLabel: string;
    expectedLearningGain: number;
    topicCoverageGain: number;
    readinessImprovement: number | null;
};

export type EvalReportAblationScenario = {
    label: string;
    sufficientData: boolean;
    assessment: string;
    policies: EvalReportAblationPolicy[];
    winners: Record<string, string>;
};

export type EvalReportAblation = {
    focusTopicCount: number;
    collection: EvalReportAblationScenario | null;
    syntheticReference: EvalReportAblationScenario | null;
};

export type EvalReportSections = {
    calibration: EvalReportCalibration | null;
    abstention: EvalReportAbstention | null;
    ablation: EvalReportAblation | null;
};

export type ReadinessModelPresentation =
    | { available: false; emptyMessage: typeof EVIDENCE_INSUFFICIENT_MESSAGE }
    | {
          available: true;
          heldOutPredictions: string;
          brierScore: string;
          meanAbsoluteError: string;
          wellCalibrated: string;
          assessment: string;
          calibrationCurve: CalibrationChartPoint[];
      };

export type EvalAbstentionSummaryPresentation =
    | { available: false; emptyMessage: typeof EVIDENCE_INSUFFICIENT_MESSAGE }
    | {
          available: true;
          abstentionRate: string;
          memoryAbstaining: string;
          performanceAbstaining: string;
          readinessAbstaining: string;
          unmetRequirements: EvalReportAbstentionRequirement[];
      };

/** @deprecated Use EvalAbstentionSummaryPresentation */
export type HonestReportingPresentation = EvalAbstentionSummaryPresentation;

export type StudyFeatureExperimentMetric = {
    label: string;
    value: string;
};

export type StudyFeatureExperimentPresentation =
    | { available: false; emptyMessage: typeof EVIDENCE_INSUFFICIENT_MESSAGE }
    | {
          available: true;
          focusTopicCount: string;
          scenarioLabel: string;
          assessment: string;
          policyResults: StudyFeatureExperimentMetric[];
          winners: StudyFeatureExperimentMetric[];
      };

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null;
}

function readNumber(value: unknown): number | undefined {
    return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function readBoolean(value: unknown): boolean | undefined {
    return typeof value === "boolean" ? value : undefined;
}

function readString(value: unknown): string | undefined {
    return typeof value === "string" ? value : undefined;
}

function formatEvalMetric(value: number | undefined): string {
    return value === undefined ? "—" : value.toFixed(4);
}

function formatPercentRatio(value: number): string {
    return `${Math.round(value * 100)}%`;
}

function readCalibrationBins(value: unknown): CalibrationChartPoint[] {
    if (!Array.isArray(value)) {
        return [];
    }
    const bins: CalibrationChartPoint[] = [];
    for (const entry of value) {
        if (!isRecord(entry)) {
            continue;
        }
        const predictedMean = readNumber(entry.predicted_mean);
        const outcomeMean = readNumber(entry.outcome_mean);
        const count = readNumber(entry.count);
        if (
            predictedMean === undefined
            || outcomeMean === undefined
            || count === undefined
            || count === 0
        ) {
            continue;
        }
        bins.push({ predictedMean, outcomeMean, count });
    }
    return bins;
}

function readAbstentionRequirements(
    value: unknown,
): EvalReportAbstentionRequirement[] {
    if (!Array.isArray(value)) {
        return [];
    }
    const requirements: EvalReportAbstentionRequirement[] = [];
    for (const entry of value) {
        if (!isRecord(entry)) {
            continue;
        }
        const id = readString(entry.id);
        const label = readString(entry.label);
        const status = readString(entry.status);
        const met = readBoolean(entry.met);
        if (id === undefined || label === undefined || status === undefined || met === undefined) {
            continue;
        }
        requirements.push({ id, label, status, met });
    }
    return requirements;
}

function readAblationScenario(value: unknown): EvalReportAblationScenario | null {
    if (!isRecord(value)) {
        return null;
    }

    const label = readString(value.label);
    const sufficientData = readBoolean(value.sufficient_data);
    const assessment = readString(value.assessment);
    const policiesValue = value.policies;
    const winnersValue = value.winners;

    if (
        label === undefined
        || sufficientData === undefined
        || assessment === undefined
        || !Array.isArray(policiesValue)
        || !isRecord(winnersValue)
    ) {
        return null;
    }

    const policies: EvalReportAblationPolicy[] = [];
    for (const entry of policiesValue) {
        if (!isRecord(entry)) {
            return null;
        }
        const policyLabel = readString(entry.policy_label);
        const expectedLearningGain = readNumber(entry.expected_learning_gain);
        const topicCoverageGain = readNumber(entry.topic_coverage_gain);
        if (policyLabel === undefined || expectedLearningGain === undefined || topicCoverageGain === undefined) {
            return null;
        }
        policies.push({
            policyLabel,
            expectedLearningGain,
            topicCoverageGain,
            readinessImprovement: readNumber(entry.readiness_improvement) ?? null,
        });
    }

    const winners: Record<string, string> = {};
    for (const [key, winner] of Object.entries(winnersValue)) {
        if (typeof winner === "string") {
            winners[key] = winner;
        }
    }

    return {
        label,
        sufficientData,
        assessment,
        policies,
        winners,
    };
}

export function parseEvalReportSections(json: string | null | undefined): EvalReportSections {
    if (!json) {
        return { calibration: null, abstention: null, ablation: null };
    }

    let parsed: unknown;
    try {
        parsed = JSON.parse(json);
    } catch {
        return { calibration: null, abstention: null, ablation: null };
    }
    if (!isRecord(parsed)) {
        return { calibration: null, abstention: null, ablation: null };
    }

    let calibration: EvalReportCalibration | null = null;
    if (isRecord(parsed.calibration)) {
        const section = parsed.calibration;
        const heldOutCount = readNumber(section.held_out_count);
        const sufficientData = readBoolean(section.sufficient_data);
        const wellCalibrated = readBoolean(section.well_calibrated);
        const assessment = readString(section.assessment);
        if (
            heldOutCount !== undefined
            && sufficientData !== undefined
            && wellCalibrated !== undefined
            && assessment !== undefined
        ) {
            calibration = {
                heldOutCount,
                sufficientData,
                wellCalibrated,
                assessment,
                brierScore: readNumber(section.brier_score),
                meanAbsoluteError: readNumber(section.mean_absolute_error),
                bins: readCalibrationBins(section.bins),
            };
        }
    }

    let abstention: EvalReportAbstention | null = null;
    if (isRecord(parsed.abstention)) {
        const section = parsed.abstention;
        const abstentionRate = readNumber(section.abstention_rate);
        const memoryAbstaining = readBoolean(section.memory_abstaining);
        const performanceAbstaining = readBoolean(section.performance_abstaining);
        const readinessAbstaining = readBoolean(section.readiness_abstaining);
        if (
            abstentionRate !== undefined
            && memoryAbstaining !== undefined
            && performanceAbstaining !== undefined
            && readinessAbstaining !== undefined
        ) {
            abstention = {
                abstentionRate,
                memoryAbstaining,
                performanceAbstaining,
                readinessAbstaining,
                unmetRequirements: readAbstentionRequirements(section.unmet_requirements),
            };
        }
    }

    let ablation: EvalReportAblation | null = null;
    if (isRecord(parsed.ablation)) {
        const focusTopicCount = readNumber(parsed.ablation.focus_topic_count);
        if (focusTopicCount !== undefined) {
            ablation = {
                focusTopicCount,
                collection: readAblationScenario(parsed.ablation.collection),
                syntheticReference: readAblationScenario(parsed.ablation.synthetic_reference),
            };
        }
    }

    return { calibration, abstention, ablation };
}

export function presentReadinessModel(
    calibration: EvalReportCalibration | null,
): ReadinessModelPresentation {
    if (!calibration?.sufficientData || calibration.heldOutCount === 0) {
        return {
            available: false,
            emptyMessage: EVIDENCE_INSUFFICIENT_MESSAGE,
        };
    }

    return {
        available: true,
        heldOutPredictions: String(calibration.heldOutCount),
        brierScore: formatEvalMetric(calibration.brierScore),
        meanAbsoluteError: formatEvalMetric(calibration.meanAbsoluteError),
        wellCalibrated: calibration.wellCalibrated ? "Yes" : "No",
        assessment: calibration.assessment,
        calibrationCurve: calibration.bins,
    };
}

function abstentionLabel(abstaining: boolean): string {
    return abstaining ? "Abstaining" : "Reporting";
}

export function presentEvalAbstentionSummary(
    abstention: EvalReportAbstention | null,
): EvalAbstentionSummaryPresentation {
    if (!abstention) {
        return {
            available: false,
            emptyMessage: EVIDENCE_INSUFFICIENT_MESSAGE,
        };
    }

    return {
        available: true,
        abstentionRate: formatPercentRatio(abstention.abstentionRate),
        memoryAbstaining: abstentionLabel(abstention.memoryAbstaining),
        performanceAbstaining: abstentionLabel(abstention.performanceAbstaining),
        readinessAbstaining: abstentionLabel(abstention.readinessAbstaining),
        unmetRequirements: abstention.unmetRequirements.filter((req) => !req.met),
    };
}

/** @deprecated Use presentEvalAbstentionSummary */
export const presentHonestReporting = presentEvalAbstentionSummary;

function pickAblationScenario(
    ablation: EvalReportAblation | null,
): EvalReportAblationScenario | null {
    if (!ablation) {
        return null;
    }
    if (ablation.collection?.sufficientData) {
        return ablation.collection;
    }
    if (ablation.syntheticReference?.sufficientData) {
        return ablation.syntheticReference;
    }
    return ablation.collection ?? ablation.syntheticReference;
}

export function presentStudyFeatureExperiment(
    ablation: EvalReportAblation | null,
): StudyFeatureExperimentPresentation {
    const scenario = pickAblationScenario(ablation);
    if (!scenario?.sufficientData || scenario.policies.length === 0) {
        return {
            available: false,
            emptyMessage: EVIDENCE_INSUFFICIENT_MESSAGE,
        };
    }

    const policyResults = scenario.policies.flatMap((policy) => [
        {
            label: `${policy.policyLabel} · expected learning gain`,
            value: policy.expectedLearningGain.toFixed(3),
        },
        {
            label: `${policy.policyLabel} · topic coverage gain`,
            value: formatPercentRatio(policy.topicCoverageGain),
        },
    ]);

    const winners = Object.entries(scenario.winners).map(([metric, winner]) => ({
        label: metric.replaceAll("_", " "),
        value: winner,
    }));

    return {
        available: true,
        focusTopicCount: String(ablation?.focusTopicCount ?? scenario.policies.length),
        scenarioLabel: scenario.label,
        assessment: scenario.assessment,
        policyResults,
        winners,
    };
}
