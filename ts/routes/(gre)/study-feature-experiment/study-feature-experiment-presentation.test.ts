// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import {
    parseStudyFeatureExperimentJson,
    presentStudyFeatureExperiment,
    STUDY_FEATURE_EXPERIMENT_NOT_RUN,
} from "./study-feature-experiment-presentation";

describe("parseStudyFeatureExperimentJson", () => {
    it("parses a real-shaped study feature experiment report", () => {
        const report = parseStudyFeatureExperimentJson(
            JSON.stringify({
                study_feature_experiment: {
                    results: [
                        { label: "BrainLift", value: "72% held-out accuracy" },
                        {
                            label: "BrainLift without feature",
                            value: "64% held-out accuracy",
                        },
                        { label: "Plain Anki", value: "61% held-out accuracy" },
                    ],
                    conclusion:
                        "Topic-priority focus improved held-out accuracy versus both controls at equal study time.",
                },
            }),
        );

        expect(report?.results).toHaveLength(3);
        expect(report?.conclusion).toContain("Topic-priority");
    });

    it("rejects reports without study_feature_experiment", () => {
        expect(parseStudyFeatureExperimentJson("{}")).toBeNull();
        expect(parseStudyFeatureExperimentJson("{ invalid")).toBeNull();
    });
});

describe("presentStudyFeatureExperiment", () => {
    it("shows the not-run message when experiment data is unavailable", () => {
        const model = presentStudyFeatureExperiment("{}");

        expect(model.resultsAvailable).toBe(false);
        expect(model.resultsMessage).toBe(STUDY_FEATURE_EXPERIMENT_NOT_RUN);
        expect(model.conclusion).toBe(STUDY_FEATURE_EXPERIMENT_NOT_RUN);
        expect(model.design.versions.map((version) => version.label)).toEqual([
            "BrainLift",
            "BrainLift without feature",
            "Plain Anki",
        ]);
    });

    it("shows recorded results without inventing values", () => {
        const model = presentStudyFeatureExperiment(
            JSON.stringify({
                study_feature_experiment: {
                    results: [{ label: "BrainLift", value: "70%" }],
                    conclusion: "Feature arm led on the pre-registered metric.",
                },
            }),
        );

        expect(model.resultsAvailable).toBe(true);
        expect(model.results).toEqual([{ label: "BrainLift", value: "70%" }]);
        expect(model.conclusion).toBe("Feature arm led on the pre-registered metric.");
    });
});
