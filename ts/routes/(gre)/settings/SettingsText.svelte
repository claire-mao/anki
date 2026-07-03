<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let description: string | null = null;
    export let value = "";

    const dispatch = createEventDispatcher<{ change: string }>();
    const inputId = `settings-text-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
</script>

<div class="settings-text">
    <label class="settings-text-copy" for={inputId}>
        <span class="settings-text-label">{label}</span>
        {#if description}
            <span class="settings-text-description">{description}</span>
        {/if}
    </label>
    <input
        id={inputId}
        type="text"
        bind:value
        on:change={() => dispatch("change", value)}
    />
</div>
