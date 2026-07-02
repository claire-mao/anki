<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { runGreNavAction, type GreNavAction } from "../gre-navigation";

    export let variant: "primary" | "secondary" | "ghost" = "secondary";
    export let size: "sm" | "md" | "lg" = "md";
    export let href: string | undefined = undefined;
    export let bridge: string | undefined = undefined;
    export let navAction: GreNavAction | undefined = undefined;
    export let type: "button" | "submit" = "button";
    export let disabled = false;
    export let loading = false;
    export let className = "";
    export let ariaDescribedby: string | undefined = undefined;

    $: linkHref = href ?? navAction?.href;

    function handleLinkClick(event: MouseEvent): void {
        if (navAction) {
            runGreNavAction(navAction, event);
            return;
        }
        if (bridge && linkHref) {
            runGreNavAction({ href: linkHref, bridge, label: "" }, event);
        }
    }
</script>

{#if linkHref}
    <a
        class="btn gre-ds-btn {className}"
        class:btn-primary={variant === "primary"}
        class:gre-ds-btn-sm={size === "sm"}
        class:btn-lg={size === "lg"}
        class:gre-ds-btn-loading={loading}
        href={linkHref}
        on:click={navAction || bridge ? handleLinkClick : undefined}
    >
        <slot />
    </a>
{:else}
    <button
        class="btn gre-ds-btn {className}"
        class:btn-primary={variant === "primary"}
        class:gre-ds-btn-sm={size === "sm"}
        class:btn-lg={size === "lg"}
        class:gre-ds-btn-loading={loading}
        {type}
        disabled={disabled || loading}
        aria-describedby={ariaDescribedby}
        on:click
    >
        <slot />
    </button>
{/if}
