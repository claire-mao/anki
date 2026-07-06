// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export const STUDY_FEATURE_EXPERIMENT_NOT_RUN = "Experiment not yet run.";

export const STUDY_FEATURE_EXPERIMENT_PAGE_TITLE = "Study Feature Experiment";
export const STUDY_FEATURE_EXPERIMENT_PAGE_SUBTITLE =
    "One educational experiment comparing BrainLift with and without a study feature against plain Anki.";

export type StudyFeatureExperimentVersion = {
    label: string;
    description: string;
};

export type StudyFeatureExperimentVersionResult = {
    label: string;
    value: string;
};

export type StudyFeatureExperimentDesign = {
    feature: string;
    hypothesis: string;
    versions: StudyFeatureExperimentVersion[];
    equalStudyTime: string;
    evaluationMetric: string;
};

export type StudyFeatureExperimentPresentation = {
    design: StudyFeatureExperimentDesign;
    resultsAvailable: boolean;
    results: StudyFeatureExperimentVersionResult[];
    resultsMessage: string;
    conclusion: string;
};

type StudyFeatureExperimentJson = {
    results: StudyFeatureExperimentVersionResult[];
    conclusion: string;
};

const STUDY_FEATURE_DESIGN: StudyFeatureExperimentDesign = {
    feature: "Topic-priority daily focus recommendations",
    hypothesis:
        "Ranking daily focus topics by coverage gaps, low mastery, and weak practice will improve learning outcomes compared with the same study time when topic priority is disabled or when using plain Anki scheduling.",
    versions: [
        {
            label: "BrainLift",
            description:
                "Full GRE Atlas with topic-priority study plan ranking enabled.",
        },
        {
            label: "BrainLift without feature",
            description:
                "GRE Atlas with topic-priority ranking disabled; other product layers unchanged.",
        },
        {
            label: "Plain Anki",
            description:
                "Unmodified Anki scheduling on the same GRE deck and cards.",
        },
    ],
    equalStudyTime:
        "Same learners, GRE question set, and study session time budget across all three arms.",
    evaluationMetric:
        "Primary: held-out practice accuracy after matched study sessions.",
};

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null;
}

function readVersionResults(value: unknown): StudyFeatureExperimentVersionResult[] | null {
    if (!Array.isArray(value) || value.length === 0) {
        return null;
    }

    const results: StudyFeatureExperimentVersionResult[] = [];
    for (const entry of value) {
        if (!isRecord(entry)) {
            return null;
        }
        const label = entry.label;
        const resultValue = entry.value;
        if (typeof label !== "string" || !label || typeof resultValue !== "string" || !resultValue) {
            return null;
        }
        results.push({ label, value: resultValue });
    }
    return results;
}

export function parseStudyFeatureExperimentJson(
    json: string,
): StudyFeatureExperimentJson | null {
    let parsed: unknown;
    try {
        parsed = JSON.parse(json);
    } catch {
        return null;
    }
    if (!isRecord(parsed)) {
        return null;
    }

    const experiment = parsed.study_feature_experiment;
    if (!isRecord(experiment)) {
        return null;
    }

    const results = readVersionResults(experiment.results);
    const conclusion = experiment.conclusion;
    if (!results || typeof conclusion !== "string" || !conclusion.trim()) {
        return null;
    }

    return {
        results,
        conclusion: conclusion.trim(),
    };
}

export function presentStudyFeatureExperiment(reportJson: string): StudyFeatureExperimentPresentation {
    const experiment = parseStudyFeatureExperimentJson(reportJson);
    if (!experiment) {
        return {
            design: STUDY_FEATURE_DESIGN,
            resultsAvailable: false,
            results: [],
            resultsMessage: STUDY_FEATURE_EXPERIMENT_NOT_RUN,
            conclusion: STUDY_FEATURE_EXPERIMENT_NOT_RUN,
        };
    }

    return {
        design: STUDY_FEATURE_DESIGN,
        resultsAvailable: true,
        results: experiment.results,
        resultsMessage: "",
        conclusion: experiment.conclusion,
    };
}
