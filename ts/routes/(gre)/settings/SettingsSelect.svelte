<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    import Select from "$lib/components/Select.svelte";

    export let label: string;
    export let description: string | null = null;
    export let value = "";
    export let options: { value: string; label: string }[] = [];

    const dispatch = createEventDispatcher<{ change: string }>();
    const selectId = `settings-select-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;

    $: selectedLabel =
        options.find((option) => option.value === value)?.label ?? options[0]?.label ?? "";
</script>

<div class="settings-select">
    <div class="settings-select-copy">
        <span class="settings-select-label" id="{selectId}-label">{label}</span>
        {#if description}
            <span class="settings-select-description">{description}</span>
        {/if}
    </div>
    <Select
        id={selectId}
        class="settings-select-control"
        bind:value
        list={options}
        label={selectedLabel}
        parser={(option) => ({ content: option.label, value: option.value })}
        on:change={({ detail }) => dispatch("change", detail.value)}
    />
</div>

<style lang="scss">
    /* Class is forwarded to the child <Select>; scope globally so it applies.
       Height uses the shared control token so it matches button heights. */
    :global(.settings-select-control) {
        min-width: 12rem;
        max-width: min(100%, 20rem);
        height: var(--gre-btn-height-md);
    }
</style>
