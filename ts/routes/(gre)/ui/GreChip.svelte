<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let active = false;
    export let type: "button" | "submit" = "button";
</script>

<button
    class="gre-ds-chip"
    class:gre-ds-chip-active={active}
    {type}
    aria-pressed={active}
    on:click
>
    <slot />
</button>
