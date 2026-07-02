<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let title: string;
    export let description: string | null = null;
</script>

<section class="gre-ds-panel settings-section">
    <header class="settings-section-header">
        <h2 class="settings-section-title">{title}</h2>
        {#if description}
            <p class="settings-section-description">{description}</p>
        {/if}
    </header>
    <div class="settings-section-body">
        <slot />
    </div>
</section>
