// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import {
    GRE_VERIFICATION_DOC_LINKS,
    presentVerification,
    presentVerificationField,
    VERIFICATION_UNKNOWN,
} from "./verification-presentation";

describe("presentVerificationField", () => {
    test("returns Unknown for empty values", () => {
        expect(presentVerificationField("")).toBe(VERIFICATION_UNKNOWN);
        expect(presentVerificationField(undefined)).toBe(VERIFICATION_UNKNOWN);
    });

    test("preserves non-empty backend values", () => {
        expect(presentVerificationField("2.1.0 (abc123) macos")).toBe(
            "2.1.0 (abc123) macos",
        );
    });
});

describe("presentVerification", () => {
    test("maps backend verification fields to settings rows", () => {
        const presentation = presentVerification({
            desktopBuild: "2.1.0 (abc123) macos",
            mobileBuild: "",
            syncStatus: "Up to date (USN 4)",
            offlineQueue: "Empty",
            conflictResolution: "mtime_secs LWW",
            duplicateProtection: "Active (similarity ≥ 0.85)",
            commitHash: "abc123",
            appVersion: "2.1.0",
            rustVersion: "1.92.0",
            aiEnabled: "Disabled",
        });

        expect(presentation.rows).toHaveLength(10);
        expect(presentation.rows[0]).toEqual({
            label: "Desktop build",
            value: "2.1.0 (abc123) macos",
        });
        expect(presentation.rows[1]).toEqual({
            label: "Mobile build",
            value: VERIFICATION_UNKNOWN,
        });
        expect(presentation.rows[9]).toEqual({
            label: "AI enabled/disabled",
            value: "Disabled",
        });
    });

    test("exposes submission doc links", () => {
        expect(GRE_VERIFICATION_DOC_LINKS.map((link) => link.label)).toEqual([
            "Architecture",
            "Submission",
            "Memory Model",
            "Performance Model",
            "AI Report",
            "Benchmark Report",
        ]);
    });

    test("falls back to Unknown when response is missing", () => {
        const presentation = presentVerification(null);
        expect(presentation.rows.every((row) => row.value === VERIFICATION_UNKNOWN)).toBe(
            true,
        );
    });
});
