<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreProgressBar from "./GreProgressBar.svelte";

    export let label: string;
    export let value: number | null = null;
    export let max = 100;

    function formatTopicMasteryValue(): string {
        if (value === null) {
            return "—";
        }
        if (max === 100) {
            return `${Math.round(value)}%`;
        }
        return String(value);
    }
</script>

<div class="gre-topic-mastery-bar">
    <span class="gre-topic-mastery-bar-label">{label}</span>
    <span class="gre-topic-mastery-bar-value">
        {formatTopicMasteryValue()}
    </span>
    <div class="gre-topic-mastery-bar-track">
        <GreProgressBar {label} {value} {max} showValue={false} compact />
    </div>
</div>

<style lang="scss">
    .gre-topic-mastery-bar-track :global(.gre-progress-bar-header) {
        display: none;
    }
</style>
