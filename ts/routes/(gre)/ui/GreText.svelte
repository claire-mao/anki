<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let variant: "hero" | "h1" | "h2" | "h3" | "body" | "caption" = "body";
    export let tag: "p" | "h1" | "h2" | "h3" | "span" | "div" = "p";
    export let id: string | undefined = undefined;
    export let muted = false;
    export let strong = false;
    export let className = "";
</script>

<svelte:element
    this={tag}
    class="gre-text gre-text-{variant} {className}"
    class:gre-text-muted={muted}
    class:gre-text-strong={strong}
    {id}
>
    <slot />
</svelte:element>
