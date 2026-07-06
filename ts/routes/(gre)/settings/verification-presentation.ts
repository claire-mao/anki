// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreAtlasVerificationResponse } from "@generated/anki/brainlift_pb";

export const VERIFICATION_UNKNOWN = "Unknown";

export type VerificationRow = {
    label: string;
    value: string;
};

export type VerificationDocLink = {
    id: string;
    label: string;
    relativePath: string;
};

export type VerificationPresentation = {
    rows: VerificationRow[];
    docLinks: VerificationDocLink[];
};

export const GRE_VERIFICATION_DOC_LINKS: VerificationDocLink[] = [
    {
        id: "architecture",
        label: "Architecture",
        relativePath: "docs/gre-atlas-submission/ARCHITECTURE.md",
    },
    {
        id: "submission",
        label: "Submission",
        relativePath: "docs/gre-atlas-submission/SUBMISSION.md",
    },
    {
        id: "memory-model",
        label: "Memory Model",
        relativePath: "docs/models/memory-model.md",
    },
    {
        id: "performance-model",
        label: "Performance Model",
        relativePath: "docs/models/performance-model.md",
    },
    {
        id: "ai-report",
        label: "AI Report",
        relativePath: "docs/gre-atlas-submission/results/gre-atlas-ai-eval.md",
    },
    {
        id: "benchmark-report",
        label: "Benchmark Report",
        relativePath: "docs/gre-atlas-submission/results/gre-atlas-benchmark.md",
    },
];

export function presentVerificationField(value: string | undefined): string {
    const trimmed = value?.trim();
    return trimmed ? trimmed : VERIFICATION_UNKNOWN;
}

export function presentVerification(
    response: GreAtlasVerificationResponse | null | undefined,
): VerificationPresentation {
    if (!response) {
        return {
            rows: verificationRowLabels().map((label) => ({
                label,
                value: VERIFICATION_UNKNOWN,
            })),
            docLinks: GRE_VERIFICATION_DOC_LINKS,
        };
    }

    return {
        rows: [
            { label: "Desktop build", value: presentVerificationField(response.desktopBuild) },
            { label: "Mobile build", value: presentVerificationField(response.mobileBuild) },
            { label: "Sync status", value: presentVerificationField(response.syncStatus) },
            { label: "Offline queue", value: presentVerificationField(response.offlineQueue) },
            {
                label: "Conflict resolution",
                value: presentVerificationField(response.conflictResolution),
            },
            {
                label: "Duplicate protection",
                value: presentVerificationField(response.duplicateProtection),
            },
            { label: "Commit hash", value: presentVerificationField(response.commitHash) },
            { label: "App version", value: presentVerificationField(response.appVersion) },
            { label: "Rust version", value: presentVerificationField(response.rustVersion) },
            { label: "AI enabled/disabled", value: presentVerificationField(response.aiEnabled) },
        ],
        docLinks: GRE_VERIFICATION_DOC_LINKS,
    };
}

function verificationRowLabels(): string[] {
    return [
        "Desktop build",
        "Mobile build",
        "Sync status",
        "Offline queue",
        "Conflict resolution",
        "Duplicate protection",
        "Commit hash",
        "App version",
        "Rust version",
        "AI enabled/disabled",
    ];
}

export function greVerificationDocBridgeCommand(link: VerificationDocLink): string {
    return `greOpenGreDoc:${link.id}`;
}
