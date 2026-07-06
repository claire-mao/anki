<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreIcon from "../GreIcon.svelte";
    import { runGreNavAction, type GreNavAction } from "../gre-navigation";

    export let action: GreNavAction;
    export let label: string;
</script>

<button
    type="button"
    class="gre-info-button"
    aria-label={label}
    title={label}
    on:click={() => runGreNavAction(action)}
>
    <GreIcon name="info" size="sm" />
</button>

<style lang="scss">
    .gre-info-button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        border: none;
        background: none;
        cursor: pointer;
        color: var(--fg-subtle);
        border-radius: var(--gre-radius-sm);
    }

    .gre-info-button:hover {
        color: var(--fg-link);
    }

    .gre-info-button:focus-visible {
        outline: 2px solid var(--fg-focus);
        outline-offset: 2px;
    }
</style>
