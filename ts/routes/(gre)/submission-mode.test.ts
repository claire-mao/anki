// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { get } from "svelte/store";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

vi.mock("$app/environment", () => ({
    browser: true,
}));

import {
    GRE_SUBMISSION_MODE_STORAGE_KEY,
    submissionMode,
    submissionModeShortcut,
    toggleSubmissionMode,
} from "./submission-mode";

describe("submission mode", () => {
    beforeEach(() => {
        sessionStorage.clear();
        submissionMode.set(false);
    });

    afterEach(() => {
        sessionStorage.clear();
        submissionMode.set(false);
    });

    test("toggleSubmissionMode persists to session storage", () => {
        expect(toggleSubmissionMode()).toBe(true);
        expect(get(submissionMode)).toBe(true);
        expect(sessionStorage.getItem(GRE_SUBMISSION_MODE_STORAGE_KEY)).toBe("1");

        expect(toggleSubmissionMode()).toBe(false);
        expect(get(submissionMode)).toBe(false);
        expect(sessionStorage.getItem(GRE_SUBMISSION_MODE_STORAGE_KEY)).toBe("0");
    });

    test("submissionModeShortcut handles Cmd+Shift+S", () => {
        const event = {
            key: "S",
            shiftKey: true,
            metaKey: true,
            ctrlKey: false,
            preventDefault: vi.fn(),
        } as unknown as KeyboardEvent;

        expect(submissionModeShortcut(event)).toBe(true);
        expect(event.preventDefault).toHaveBeenCalled();
        expect(get(submissionMode)).toBe(true);
    });

    test("submissionModeShortcut handles Ctrl+Shift+S", () => {
        const event = {
            key: "s",
            shiftKey: true,
            metaKey: false,
            ctrlKey: true,
            preventDefault: vi.fn(),
        } as unknown as KeyboardEvent;

        expect(submissionModeShortcut(event)).toBe(true);
        expect(get(submissionMode)).toBe(true);
    });

    test("submissionModeShortcut ignores unrelated keys", () => {
        const event = {
            key: "s",
            shiftKey: false,
            metaKey: true,
            ctrlKey: false,
            preventDefault: vi.fn(),
        } as unknown as KeyboardEvent;

        expect(submissionModeShortcut(event)).toBe(false);
        expect(event.preventDefault).not.toHaveBeenCalled();
    });
});
