<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let description: string | null = null;
    export let value = 0;
    export let min: number | undefined = undefined;
    export let max: number | undefined = undefined;
    export let suffix: string | null = null;

    const dispatch = createEventDispatcher<{ change: number }>();
    const inputId = `settings-number-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
</script>

<div class="settings-number">
    <label class="settings-number-copy" for={inputId}>
        <span class="settings-number-label">{label}</span>
        {#if description}
            <span class="settings-number-description">{description}</span>
        {/if}
    </label>
    <div class="settings-number-input">
        <input
            id={inputId}
            type="number"
            bind:value
            {min}
            {max}
            on:change={() => dispatch("change", value)}
        />
        {#if suffix}
            <span class="settings-number-suffix">{suffix}</span>
        {/if}
    </div>
</div>
