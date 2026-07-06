<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";
    import { navigating, page } from "$app/stores";
    import { fade } from "svelte/transition";

    import { greMotionDuration } from "./motion";
    import GreIcon from "./GreIcon.svelte";
    import {
        greNavHref,
        grePrimaryNavItems,
        greSubmissionNavAction,
        greSubmissionNavItems,
        greUtilityNavItems,
        greNavAction,
        isGreNavActive,
        isGreSubmissionNavActive,
        runGreNavAction,
        type GreNavItem,
        type GreSubmissionNavItem,
    } from "./gre-navigation";
    import {
        initSubmissionMode,
        submissionMode,
        submissionModeShortcut,
    } from "./submission-mode";
    import GrePageSkeleton from "./ui/GrePageSkeleton.svelte";

    import "./gre.scss";

    function onNavClick(item: GreNavItem, event: MouseEvent): void {
        runGreNavAction(greNavAction(item), event);
    }

    function onSubmissionNavClick(item: GreSubmissionNavItem, event: MouseEvent): void {
        runGreNavAction(greSubmissionNavAction(item), event);
    }

    function onWindowKeydown(event: KeyboardEvent): void {
        submissionModeShortcut(event);
    }

    onMount(() => initSubmissionMode());

    $: pageFadeIn = greMotionDuration(160);
    $: pageFadeOut = greMotionDuration(120);
    $: skeletonFade = greMotionDuration(120);
</script>

<svelte:window on:keydown={onWindowKeydown} />

<svelte:head>
    <title>GRE Atlas</title>
</svelte:head>

<div class="gre-shell" class:gre-submission-mode={$submissionMode}>
    <header class="gre-header">
        <div class="brand">
            <span class="brand-mark"><GreIcon name="compass" size="sm" /></span>
            GRE Atlas
            {#if $submissionMode}
                <span class="gre-submission-mode-indicator">Submission mode</span>
            {/if}
        </div>
        <nav class="gre-nav" aria-label="GRE sections">
            {#if $submissionMode}
                {#each greSubmissionNavItems as item}
                    <a
                        class="nav-link"
                        class:nav-link-active={isGreSubmissionNavActive(
                            item,
                            $page.url.pathname,
                        )}
                        href="/{item.page}"
                        aria-current={isGreSubmissionNavActive(item, $page.url.pathname)
                            ? "page"
                            : undefined}
                        on:click={(event) => onSubmissionNavClick(item, event)}
                    >
                        <GreIcon name={item.icon} size="sm" />
                        <span class="nav-link-label">{item.label}</span>
                    </a>
                {/each}
            {:else}
                {#each grePrimaryNavItems as item}
                    <a
                        class="nav-link"
                        class:nav-link-active={isGreNavActive(item, $page.url.pathname)}
                        href={greNavHref(item)}
                        aria-current={isGreNavActive(item, $page.url.pathname)
                            ? "page"
                            : undefined}
                        on:click={(event) => onNavClick(item, event)}
                    >
                        <GreIcon name={item.icon} size="sm" />
                        <span class="nav-link-label">{item.label}</span>
                    </a>
                {/each}
                <span class="gre-nav-divider gre-dev-tools" aria-hidden="true"></span>
                {#each greUtilityNavItems as item}
                    <a
                        class="nav-link nav-link-utility gre-dev-tools"
                        class:nav-link-active={isGreNavActive(item, $page.url.pathname)}
                        href={greNavHref(item)}
                        aria-current={isGreNavActive(item, $page.url.pathname)
                            ? "page"
                            : undefined}
                        on:click={(event) => onNavClick(item, event)}
                    >
                        <GreIcon name={item.icon} size="sm" />
                        <span class="nav-link-label">{item.label}</span>
                    </a>
                {/each}
            {/if}
        </nav>
    </header>
    <main class="gre-main" aria-busy={$navigating ? "true" : undefined}>
        {#if $navigating}
            <div
                class="gre-page gre-page-loading"
                in:fade={{ duration: skeletonFade }}
                out:fade={{ duration: pageFadeOut }}
            >
                <GrePageSkeleton />
            </div>
        {:else}
            {#key $page.url.pathname}
                <div
                    class="gre-page"
                    in:fade={{ duration: pageFadeIn }}
                    out:fade={{ duration: pageFadeOut }}
                >
                    <slot />
                </div>
            {/key}
        {/if}
    </main>
</div>
