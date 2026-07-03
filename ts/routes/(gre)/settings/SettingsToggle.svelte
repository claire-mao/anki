<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let description: string | null = null;
    export let checked = false;

    const dispatch = createEventDispatcher<{ change: boolean }>();
</script>

<label class="settings-toggle">
    <span class="settings-toggle-copy">
        <span class="settings-toggle-label">{label}</span>
        {#if description}
            <span class="settings-toggle-description">{description}</span>
        {/if}
    </span>
    <input type="checkbox" bind:checked on:change={() => dispatch("change", checked)} />
</label>
