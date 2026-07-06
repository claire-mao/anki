// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { browser } from "$app/environment";
import { writable } from "svelte/store";

export const GRE_SUBMISSION_MODE_STORAGE_KEY = "gre-atlas-submission-mode";
export const GRE_SUBMISSION_MODE_TOGGLE_EVENT = "gre-toggle-submission-mode";

function readStoredSubmissionMode(): boolean {
    if (!browser) {
        return false;
    }
    return sessionStorage.getItem(GRE_SUBMISSION_MODE_STORAGE_KEY) === "1";
}

function persistSubmissionMode(enabled: boolean): void {
    if (!browser) {
        return;
    }
    sessionStorage.setItem(GRE_SUBMISSION_MODE_STORAGE_KEY, enabled ? "1" : "0");
}

/** Whether GRE Atlas is in grader/demo presentation mode. */
export const submissionMode = writable(false);

export function initSubmissionMode(): () => void {
    if (!browser) {
        return () => {};
    }

    submissionMode.set(readStoredSubmissionMode());

    const onToggle = (): void => {
        toggleSubmissionMode();
    };
    window.addEventListener(GRE_SUBMISSION_MODE_TOGGLE_EVENT, onToggle);
    return () => window.removeEventListener(GRE_SUBMISSION_MODE_TOGGLE_EVENT, onToggle);
}

export function toggleSubmissionMode(): boolean {
    let next = false;
    submissionMode.update((current) => {
        next = !current;
        persistSubmissionMode(next);
        return next;
    });
    return next;
}

/** Returns true when the shortcut was handled. */
export function submissionModeShortcut(event: KeyboardEvent): boolean {
    if (event.key.toLowerCase() !== "s") {
        return false;
    }
    if (!event.shiftKey) {
        return false;
    }
    if (!event.metaKey && !event.ctrlKey) {
        return false;
    }
    event.preventDefault();
    toggleSubmissionMode();
    return true;
}
