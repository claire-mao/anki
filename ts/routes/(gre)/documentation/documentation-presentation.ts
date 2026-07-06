// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export type DocumentationId =
    | "architecture"
    | "ai"
    | "memory-model"
    | "performance-model"
    | "readiness-model"
    | "submission";

export type DocumentationEntry = {
    id: DocumentationId;
    title: string;
    sourceFile: string;
    markdown: string;
};

export type DocumentationSearchHit = {
    docId: DocumentationId;
    title: string;
    excerpt: string;
    matchCount: number;
};

export const DOCUMENTATION_PAGE_TITLE = "Documentation";
export const DOCUMENTATION_PAGE_SUBTITLE =
    "Architecture, models, and submission reference for GRE Atlas.";

const documentationIds = new Set<string>([
    "architecture",
    "ai",
    "memory-model",
    "performance-model",
    "readiness-model",
    "submission",
]);

const sourceFileToId: Record<string, DocumentationId> = {
    "architecture.md": "architecture",
    "ai.md": "ai",
    "memory-model.md": "memory-model",
    "performance-model.md": "performance-model",
    "readiness-model.md": "readiness-model",
    "submission.md": "submission",
};

export function isDocumentationId(value: string | null | undefined): value is DocumentationId {
    return value != null && documentationIds.has(value);
}

export function defaultDocumentationId(): DocumentationId {
    return "architecture";
}

function escapeRegExp(value: string): string {
    return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function normalizedQuery(query: string): string {
    return query.trim().toLowerCase();
}

export function countDocumentationMatches(text: string, query: string): number {
    const normalized = normalizedQuery(query);
    if (!normalized) {
        return 0;
    }
    const matches = text.toLowerCase().match(new RegExp(escapeRegExp(normalized), "g"));
    return matches?.length ?? 0;
}

export function buildDocumentationExcerpt(
    text: string,
    query: string,
    radius = 90,
): string {
    const normalized = normalizedQuery(query);
    if (!normalized) {
        return "";
    }
    const lower = text.toLowerCase();
    const index = lower.indexOf(normalized);
    if (index < 0) {
        return "";
    }

    const start = Math.max(0, index - radius);
    const end = Math.min(text.length, index + normalized.length + radius);
    let excerpt = text.slice(start, end).replace(/\s+/g, " ").trim();
    if (start > 0) {
        excerpt = `…${excerpt}`;
    }
    if (end < text.length) {
        excerpt = `${excerpt}…`;
    }
    return excerpt;
}

export function highlightDocumentationExcerpt(excerpt: string, query: string): string {
    const normalized = normalizedQuery(query);
    if (!normalized) {
        return excerpt;
    }
    const pattern = new RegExp(`(${escapeRegExp(normalized)})`, "gi");
    return excerpt.replace(pattern, "<mark>$1</mark>");
}

export function searchDocumentation(
    entries: DocumentationEntry[],
    query: string,
): DocumentationSearchHit[] {
    const normalized = normalizedQuery(query);
    if (!normalized) {
        return [];
    }

    return entries
        .map((entry) => {
            const matchCount = countDocumentationMatches(entry.markdown, normalized);
            if (matchCount === 0) {
                return null;
            }
            return {
                docId: entry.id,
                title: entry.title,
                excerpt: buildDocumentationExcerpt(entry.markdown, normalized),
                matchCount,
            };
        })
        .filter((hit): hit is DocumentationSearchHit => hit != null)
        .sort((left, right) => right.matchCount - left.matchCount || left.title.localeCompare(right.title));
}

export function filterDocumentationEntries(
    entries: DocumentationEntry[],
    query: string,
): DocumentationEntry[] {
    const normalized = normalizedQuery(query);
    if (!normalized) {
        return entries;
    }
    const matchingIds = new Set(searchDocumentation(entries, normalized).map((hit) => hit.docId));
    return entries.filter((entry) => matchingIds.has(entry.id));
}

function sourceBasename(href: string): string {
    const withoutHash = href.split("#")[0] ?? href;
    const segments = withoutHash.split(/[/\\]/);
    return segments[segments.length - 1]?.toLowerCase() ?? "";
}

export function resolveDocumentationLink(
    href: string,
    entries: DocumentationEntry[],
): DocumentationId | null {
    const trimmed = href.trim();
    if (!trimmed || trimmed.startsWith("#")) {
        return null;
    }
    if (/^[a-z]+:/i.test(trimmed)) {
        return null;
    }

    const basename = sourceBasename(trimmed);
    const mapped = sourceFileToId[basename];
    if (mapped) {
        return mapped;
    }

    const entry = entries.find(
        (candidate) =>
            candidate.sourceFile.toLowerCase() === basename ||
            candidate.id === trimmed.replace(/\.md$/i, ""),
    );
    return entry?.id ?? null;
}

export function isExternalDocumentationLink(href: string): boolean {
    const trimmed = href.trim();
    return /^https?:\/\//i.test(trimmed);
}
