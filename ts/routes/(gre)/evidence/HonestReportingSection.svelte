<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import HonestReportingPanel from "./HonestReportingPanel.svelte";
    import type { HonestReportingPresentation } from "./honest-reporting-presentation";

    export let model: HonestReportingPresentation;
</script>

<HonestReportingPanel {model} />
