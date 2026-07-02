<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ComponentProps } from "svelte";

    import GreIcon from "../GreIcon.svelte";
    import GreText from "./GreText.svelte";

    export let title: string;
    export let icon: ComponentProps<GreIcon>["name"] | null = null;
</script>

<div class="gre-ds-score-card">
    <GreText variant="h3" tag="h2" className="gre-text-label-row">
        {#if icon}
            <GreIcon name={icon} size="sm" />
        {/if}
        {title}
    </GreText>
    <div class="gre-ds-score-card-body">
        <slot />
    </div>
</div>

<style lang="scss">
    .gre-ds-score-card-body {
        margin-top: var(--gre-space-3);
    }
</style>
