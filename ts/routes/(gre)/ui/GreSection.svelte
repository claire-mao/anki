<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let gap: "sm" | "md" | "lg" = "md";
</script>

<section class="gre-ds-section" class:gre-ds-section-sm={gap === "sm"} class:gre-ds-section-lg={gap === "lg"}>
    <slot />
</section>
