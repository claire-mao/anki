// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    presentPredictionReadiness,
    type PredictionReadinessInput,
    type PredictionReadinessPresentation,
} from "./prediction-readiness-presentation";

export type OnboardingInput = PredictionReadinessInput;
export type OnboardingPresentation = PredictionReadinessPresentation;

export function presentOnboarding(input: OnboardingInput): OnboardingPresentation {
    return presentPredictionReadiness(input);
}

export type { PredictionReadinessPresentation, PredictionReadinessInput, OnboardingContext } from "./prediction-readiness-presentation";
