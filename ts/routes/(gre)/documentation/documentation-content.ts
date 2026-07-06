// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import architectureMarkdown from "../../../../docs/gre-atlas-submission/ARCHITECTURE.md?raw";
import aiMarkdown from "../../../../docs/gre-atlas-submission/AI.md?raw";
import memoryModelMarkdown from "../../../../docs/models/memory-model.md?raw";
import performanceModelMarkdown from "../../../../docs/models/performance-model.md?raw";
import readinessModelMarkdown from "../../../../docs/models/readiness-model.md?raw";
import submissionMarkdown from "../../../../docs/gre-atlas-submission/SUBMISSION.md?raw";

import type { DocumentationEntry, DocumentationId } from "./documentation-presentation";

const documentationById: Record<DocumentationId, DocumentationEntry> = {
    architecture: {
        id: "architecture",
        title: "Architecture",
        sourceFile: "ARCHITECTURE.md",
        markdown: architectureMarkdown,
    },
    ai: {
        id: "ai",
        title: "AI question generation",
        sourceFile: "AI.md",
        markdown: aiMarkdown,
    },
    "memory-model": {
        id: "memory-model",
        title: "Memory model",
        sourceFile: "memory-model.md",
        markdown: memoryModelMarkdown,
    },
    "performance-model": {
        id: "performance-model",
        title: "Performance model",
        sourceFile: "performance-model.md",
        markdown: performanceModelMarkdown,
    },
    "readiness-model": {
        id: "readiness-model",
        title: "Readiness model",
        sourceFile: "readiness-model.md",
        markdown: readinessModelMarkdown,
    },
    submission: {
        id: "submission",
        title: "Submission summary",
        sourceFile: "SUBMISSION.md",
        markdown: submissionMarkdown,
    },
};

export function documentationEntries(): DocumentationEntry[] {
    return [
        documentationById.architecture,
        documentationById.ai,
        documentationById["memory-model"],
        documentationById["performance-model"],
        documentationById["readiness-model"],
        documentationById.submission,
    ];
}

export function documentationEntry(id: DocumentationId): DocumentationEntry {
    return documentationById[id];
}
