// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { Preferences_Scheduling_NewReviewMix } from "@generated/anki/config_pb";

export interface SettingsSelectOption {
    value: string;
    label: string;
}

export const newReviewMixOptions: SettingsSelectOption[] = [
    {
        value: String(Preferences_Scheduling_NewReviewMix.DISTRIBUTE),
        label: "Mix new and review cards",
    },
    {
        value: String(Preferences_Scheduling_NewReviewMix.REVIEWS_FIRST),
        label: "Show review cards first",
    },
    {
        value: String(Preferences_Scheduling_NewReviewMix.NEW_FIRST),
        label: "Show new cards first",
    },
];

export function newReviewMixLabel(value: string): string {
    return (
        newReviewMixOptions.find((option) => option.value === value)?.label ??
        newReviewMixOptions[0]?.label ??
        ""
    );
}
