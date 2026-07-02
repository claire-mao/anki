<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let width = "100%";
    export let height = "0.875rem";
    export let shape: "sm" | "md" | "lg" | "pill" = "sm";
</script>

<span
    class="gre-skeleton"
    class:gre-skeleton-md={shape === "md"}
    class:gre-skeleton-lg={shape === "lg"}
    class:gre-skeleton-pill={shape === "pill"}
    style:width
    style:height
    aria-hidden="true"
></span>
