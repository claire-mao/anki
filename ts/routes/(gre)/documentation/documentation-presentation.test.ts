// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import {
    buildDocumentationExcerpt,
    countDocumentationMatches,
    filterDocumentationEntries,
    isDocumentationId,
    resolveDocumentationLink,
    searchDocumentation,
    type DocumentationEntry,
} from "./documentation-presentation";

const sampleEntries: DocumentationEntry[] = [
    {
        id: "architecture",
        title: "Architecture",
        sourceFile: "ARCHITECTURE.md",
        markdown: "GRE Atlas architecture with BrainLiftService and greatlas.db.",
    },
    {
        id: "memory-model",
        title: "Memory model",
        sourceFile: "memory-model.md",
        markdown: "Memory score uses FSRS retrievability across GRE topic tags.",
    },
];

describe("documentation search", () => {
    it("counts matches case-insensitively", () => {
        expect(countDocumentationMatches("GRE Atlas and gre atlas", "gre atlas")).toBe(2);
    });

    it("builds an excerpt around the first match", () => {
        const excerpt = buildDocumentationExcerpt(
            "Alpha beta gamma FSRS retrievability delta",
            "fsrs",
            8,
        );
        expect(excerpt).toContain("FSRS");
        expect(excerpt.startsWith("…")).toBe(true);
    });

    it("returns ranked search hits", () => {
        const hits = searchDocumentation(sampleEntries, "gre");
        expect(hits).toHaveLength(2);
        expect(hits[0]?.docId).toBe("architecture");
    });

    it("filters the sidebar list to matching docs", () => {
        const filtered = filterDocumentationEntries(sampleEntries, "fsrs");
        expect(filtered.map((entry) => entry.id)).toEqual(["memory-model"]);
    });
});

describe("documentation links", () => {
    it("recognizes documentation ids", () => {
        expect(isDocumentationId("architecture")).toBe(true);
        expect(isDocumentationId("missing")).toBe(false);
    });

    it("resolves relative markdown links to in-app docs", () => {
        expect(resolveDocumentationLink("./ARCHITECTURE.md", sampleEntries)).toBe(
            "architecture",
        );
        expect(resolveDocumentationLink("../models/memory-model.md", sampleEntries)).toBe(
            "memory-model",
        );
    });

    it("ignores external links", () => {
        expect(resolveDocumentationLink("https://example.com", sampleEntries)).toBeNull();
    });
});
