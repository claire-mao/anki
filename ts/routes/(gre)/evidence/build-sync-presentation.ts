// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { GreAtlasVerificationResponse } from "@generated/anki/brainlift_pb";

import {
    GRE_VERIFICATION_DOC_LINKS,
    presentVerificationField,
    type VerificationDocLink,
    type VerificationRow,
} from "../settings/verification-presentation";
import { EVIDENCE_INSUFFICIENT_MESSAGE } from "./constants";

export type EvidenceMetricRow = {
    label: string;
    value: string;
};

export type BuildInformationPresentation =
    | { available: false; emptyMessage: typeof EVIDENCE_INSUFFICIENT_MESSAGE }
    | {
          available: true;
          rows: EvidenceMetricRow[];
          docLinks: VerificationDocLink[];
      };

export type SyncVerificationPresentation =
    | { available: false; emptyMessage: typeof EVIDENCE_INSUFFICIENT_MESSAGE }
    | {
          available: true;
          rows: EvidenceMetricRow[];
          docLinks: VerificationDocLink[];
      };

export type DocumentationEvidencePresentation = {
    description: string;
    docLinks: VerificationDocLink[];
};

const BUILD_DOC_LINK: VerificationDocLink = {
    id: "build",
    label: "Build guide",
    relativePath: "docs/gre-atlas-submission/BUILD.md",
};

const SYNC_DOC_LINK: VerificationDocLink = {
    id: "sync-verification",
    label: "Sync verification",
    relativePath: "docs/gre-atlas-submission/SYNC-VERIFICATION.md",
};

const EVAL_DOC_LINK: VerificationDocLink = {
    id: "evaluation",
    label: "Evaluation guide",
    relativePath: "docs/gre-atlas-submission/EVALUATION.md",
};

function verificationRows(
    response: GreAtlasVerificationResponse | null | undefined,
): VerificationRow[] {
    if (!response) {
        return [];
    }

    return [
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
    ];
}

function pickRows(rows: VerificationRow[], labels: string[]): EvidenceMetricRow[] {
    return labels.flatMap((label) => {
        const row = rows.find((entry) => entry.label === label);
        return row ? [row] : [];
    });
}

export function presentBuildInformation(
    response: GreAtlasVerificationResponse | null | undefined,
): BuildInformationPresentation {
    const rows = verificationRows(response);
    if (rows.length === 0) {
        return {
            available: false,
            emptyMessage: EVIDENCE_INSUFFICIENT_MESSAGE,
        };
    }

    return {
        available: true,
        rows: pickRows(rows, [
            "Desktop build",
            "Mobile build",
            "Commit hash",
            "App version",
            "Rust version",
            "AI enabled/disabled",
        ]),
        docLinks: [BUILD_DOC_LINK, ...GRE_VERIFICATION_DOC_LINKS.filter((link) => link.id === "benchmark-report")],
    };
}

export function presentSyncVerification(
    response: GreAtlasVerificationResponse | null | undefined,
): SyncVerificationPresentation {
    const rows = verificationRows(response);
    if (rows.length === 0) {
        return {
            available: false,
            emptyMessage: EVIDENCE_INSUFFICIENT_MESSAGE,
        };
    }

    return {
        available: true,
        rows: pickRows(rows, [
            "Sync status",
            "Offline queue",
            "Conflict resolution",
            "Duplicate protection",
        ]),
        docLinks: [SYNC_DOC_LINK],
    };
}

export function presentDocumentationEvidence(): DocumentationEvidencePresentation {
    return {
        description:
            "Architecture, model specifications, evaluation methodology, and submission reference.",
        docLinks: [
            ...GRE_VERIFICATION_DOC_LINKS,
            BUILD_DOC_LINK,
            SYNC_DOC_LINK,
            EVAL_DOC_LINK,
        ],
    };
}
