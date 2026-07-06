// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";
import {
    presentBuildInformation,
    presentDocumentationEvidence,
    presentSyncVerification,
} from "./build-sync-presentation";

describe("presentBuildInformation", () => {
    test("maps verification fields to build rows", () => {
        const model = presentBuildInformation({
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

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.rows.some((row) => row.label === "Commit hash" && row.value === "abc123")).toBe(
            true,
        );
    });

    test("shows insufficient data when verification is unavailable", () => {
        const model = presentBuildInformation(null);
        expect(model.available).toBe(false);
        if (model.available) {
            throw new Error("expected unavailable model");
        }
        expect(model.emptyMessage).toBe(EVIDENCE_INSUFFICIENT_MESSAGE);
    });
});

describe("presentSyncVerification", () => {
    test("maps verification fields to sync rows", () => {
        const model = presentSyncVerification({
            desktopBuild: "2.1.0 (abc123) macos",
            mobileBuild: "",
            syncStatus: "2 pending upload (USN 3)",
            offlineQueue: "2 row(s) queued",
            conflictResolution: "mtime_secs LWW",
            duplicateProtection: "Active (similarity ≥ 0.85)",
            commitHash: "abc123",
            appVersion: "2.1.0",
            rustVersion: "1.92.0",
            aiEnabled: "Disabled",
        });

        expect(model.available).toBe(true);
        if (!model.available) {
            throw new Error("expected available model");
        }
        expect(model.rows.map((row) => row.label)).toEqual([
            "Sync status",
            "Offline queue",
            "Conflict resolution",
            "Duplicate protection",
        ]);
    });
});

describe("presentDocumentationEvidence", () => {
    test("exposes documentation links without inventing metrics", () => {
        const model = presentDocumentationEvidence();
        expect(model.docLinks.some((link) => link.id === "memory-model")).toBe(true);
        expect(model.docLinks.some((link) => link.id === "evaluation")).toBe(true);
    });
});
