<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ComponentProps } from "svelte";

    import GreIcon from "./GreIcon.svelte";
    import GreText from "./ui/GreText.svelte";

    export let title: string;
    export let subtitle: string | null = null;
    export let meta: string | null = null;
    export let icon: ComponentProps<GreIcon>["name"] | null = null;
</script>

<header class="gre-page-header">
    <div class="gre-page-header-main">
        {#if icon}
            <span class="gre-ds-icon-box" aria-hidden="true">
                <GreIcon name={icon} size="xl" />
            </span>
        {/if}
        <div>
            <GreText variant="h1" tag="h1">{title}</GreText>
            {#if subtitle}
                <GreText variant="body" tag="p" muted className="gre-page-subtitle">
                    {subtitle}
                </GreText>
            {/if}
        </div>
    </div>
    {#if meta}
        <GreText variant="caption" tag="p" muted className="gre-page-meta">
            {meta}
        </GreText>
    {/if}
    <slot />
</header>
