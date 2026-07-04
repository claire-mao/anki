<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { navigating, page } from "$app/stores";
    import { fade } from "svelte/transition";

    import { greMotionDuration } from "./motion";
    import GreIcon from "./GreIcon.svelte";
    import {
        greNavHref,
        grePrimaryNavItems,
        greUtilityNavItems,
        isGreNavActive,
        runGreNavAction,
        greNavAction,
        type GreNavItem,
    } from "./gre-navigation";
    import GrePageSkeleton from "./ui/GrePageSkeleton.svelte";

    import "./gre.scss";

    function onNavClick(item: GreNavItem, event: MouseEvent): void {
        runGreNavAction(greNavAction(item), event);
    }

    $: pageFadeIn = greMotionDuration(160);
    $: pageFadeOut = greMotionDuration(120);
    $: skeletonFade = greMotionDuration(120);
</script>

<svelte:head>
    <title>GRE Atlas</title>
</svelte:head>

<div class="gre-shell">
    <header class="gre-header">
        <div class="brand">
            <span class="brand-mark"><GreIcon name="compass" size="sm" /></span>
            GRE Atlas
        </div>
        <nav class="gre-nav" aria-label="GRE sections">
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
            <span class="gre-nav-divider" aria-hidden="true"></span>
            {#each greUtilityNavItems as item}
                <a
                    class="nav-link nav-link-utility"
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
