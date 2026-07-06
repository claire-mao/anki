<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { renderMarkdown } from "@tslib/helpers";

    import GrePageHeader from "../GrePageHeader.svelte";
    import GreText from "../ui/GreText.svelte";
    import { documentationEntries } from "./documentation-content";
    import {
        DOCUMENTATION_PAGE_SUBTITLE,
        DOCUMENTATION_PAGE_TITLE,
        defaultDocumentationId,
        filterDocumentationEntries,
        highlightDocumentationExcerpt,
        isDocumentationId,
        isExternalDocumentationLink,
        resolveDocumentationLink,
        searchDocumentation,
        type DocumentationId,
    } from "./documentation-presentation";

    import "../gre.scss";
    import "./documentation.scss";

    const entries = documentationEntries();

    let searchQuery = "";
    let contentEl: HTMLElement | null = null;

    $: activeDocId = isDocumentationId($page.url.searchParams.get("doc"))
        ? ($page.url.searchParams.get("doc") as DocumentationId)
        : defaultDocumentationId();
    $: activeEntry = entries.find((entry) => entry.id === activeDocId) ?? entries[0]!;
    $: searchHits = searchDocumentation(entries, searchQuery);
    $: visibleEntries = filterDocumentationEntries(entries, searchQuery);
    $: renderedHtml = renderMarkdown(activeEntry.markdown);

    async function selectDoc(docId: DocumentationId): Promise<void> {
        const url = new URL($page.url);
        url.searchParams.set("doc", docId);
        await goto(`${url.pathname}?${url.searchParams.toString()}`, {
            keepFocus: true,
            noScroll: true,
        });
        contentEl?.scrollTo({ top: 0 });
    }

    function onSearchInput(event: Event): void {
        const target = event.currentTarget as HTMLInputElement;
        searchQuery = target.value;
    }

    function onMarkdownClick(event: MouseEvent): void {
        const target = event.target;
        if (!(target instanceof HTMLAnchorElement)) {
            return;
        }
        const href = target.getAttribute("href");
        if (!href) {
            return;
        }

        if (isExternalDocumentationLink(href)) {
            return;
        }

        const docId = resolveDocumentationLink(href, entries);
        if (docId) {
            event.preventDefault();
            void selectDoc(docId);
            return;
        }

        if (/\.md(?:$|[#?])/i.test(href)) {
            event.preventDefault();
        }
    }
</script>

<GrePageHeader
    title={DOCUMENTATION_PAGE_TITLE}
    subtitle={DOCUMENTATION_PAGE_SUBTITLE}
    icon="info"
/>

<div class="documentation-page">
    <aside class="documentation-sidebar" aria-label="Documentation navigation">
        <label class="documentation-search">
            <span class="documentation-search-label">Search documentation</span>
            <input
                type="search"
                placeholder="Search architecture, models, submission…"
                value={searchQuery}
                on:input={onSearchInput}
            />
        </label>

        {#if searchQuery.trim()}
            <section class="documentation-search-results" aria-label="Search results">
                <GreText variant="caption" muted className="documentation-search-meta">
                    {searchHits.length} document{searchHits.length === 1 ? "" : "s"} matched
                </GreText>
                {#if searchHits.length === 0}
                    <p class="documentation-empty">No matches for “{searchQuery.trim()}”.</p>
                {:else}
                    <ul class="documentation-search-list">
                        {#each searchHits as hit}
                            <li>
                                <button
                                    type="button"
                                    class="documentation-search-hit"
                                    class:documentation-search-hit-active={hit.docId === activeDocId}
                                    on:click={() => selectDoc(hit.docId)}
                                >
                                    <span class="documentation-search-hit-title">{hit.title}</span>
                                    <span class="documentation-search-hit-count">
                                        {hit.matchCount} match{hit.matchCount === 1 ? "" : "es"}
                                    </span>
                                    <span class="documentation-search-hit-excerpt">
                                        {@html highlightDocumentationExcerpt(hit.excerpt, searchQuery)}
                                    </span>
                                </button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </section>
        {/if}

        <nav class="documentation-nav" aria-label="Documents">
            <GreText variant="caption" muted className="documentation-nav-label">
                Documents
            </GreText>
            <ul class="documentation-nav-list">
                {#each visibleEntries as entry}
                    <li>
                        <button
                            type="button"
                            class="documentation-nav-link"
                            class:documentation-nav-link-active={entry.id === activeDocId}
                            aria-current={entry.id === activeDocId ? "page" : undefined}
                            on:click={() => selectDoc(entry.id)}
                        >
                            <span>{entry.title}</span>
                            <span class="documentation-nav-file">{entry.sourceFile}</span>
                        </button>
                    </li>
                {/each}
            </ul>
        </nav>
    </aside>

    <article class="documentation-content gre-ds-panel">
        <header class="documentation-content-header">
            <GreText variant="h2" tag="h2">{activeEntry.title}</GreText>
            <GreText variant="caption" muted>{activeEntry.sourceFile}</GreText>
        </header>
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            class="documentation-markdown"
            bind:this={contentEl}
            on:click={onMarkdownClick}
        >
            {@html renderedHtml}
        </div>
    </article>
</div>
