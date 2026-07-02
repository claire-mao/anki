<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreText from "./GreText.svelte";

    export let title: string | null = null;
    export let headingLevel: 2 | 3 = 2;
    export let interactive = false;
    export let className = "";
</script>

<section class="gre-ds-panel {className}" class:gre-ds-panel-interactive={interactive}>
    {#if title}
        <GreText variant="h2" tag={`h${headingLevel}`}>{title}</GreText>
    {/if}
    <div class="gre-ds-panel-body">
        <slot />
    </div>
</section>

<style lang="scss">
    .gre-ds-panel-body {
        margin-top: var(--gre-space-3);
    }

    .gre-ds-panel:not(:has(.gre-text-h2)) .gre-ds-panel-body {
        margin-top: 0;
    }
</style>
