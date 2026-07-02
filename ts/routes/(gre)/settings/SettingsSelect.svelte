<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let description: string | null = null;
    export let value = "";
    export let options: { value: string; label: string }[] = [];

    const dispatch = createEventDispatcher<{ change: string }>();
    const selectId = `settings-select-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
</script>

<div class="settings-select">
    <label class="settings-select-copy" for={selectId}>
        <span class="settings-select-label">{label}</span>
        {#if description}
            <span class="settings-select-description">{description}</span>
        {/if}
    </label>
    <select id={selectId} bind:value on:change={() => dispatch("change", value)}>
        {#each options as option (option.value)}
            <option value={option.value}>{option.label}</option>
        {/each}
    </select>
</div>
