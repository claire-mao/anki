<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { SessionCompletionSummary } from "../session-completion";
    import GreButton from "./GreButton.svelte";
    import GreButtonRow from "./GreButtonRow.svelte";
    import type { GreNavAction } from "../gre-navigation";
    import { runGreNavAction } from "../gre-navigation";

    export let summary: SessionCompletionSummary;
    export let onSecondary: (() => void) | undefined = undefined;
    export let secondaryLabel: string | undefined = undefined;

    function runAction(action: GreNavAction, event?: Event): void {
        runGreNavAction(action, event);
    }
</script>

<section class="session-complete" aria-labelledby="session-complete-title">
    <h2 class="session-complete-title" id="session-complete-title">{summary.headline}</h2>
    <p class="session-complete-subline">{summary.subline}</p>

    <dl class="session-complete-rows">
        {#each summary.rows as row}
            <div class="session-complete-row">
                <dt>{row.label}</dt>
                <dd>{row.value}</dd>
            </div>
        {/each}
    </dl>

    <p class="session-complete-next">{summary.nextActionDetail}</p>

    <GreButtonRow className="session-complete-actions">
        <GreButton
            variant="primary"
            size="lg"
            on:click={(event) => runAction(summary.nextAction, event)}
        >
            {summary.nextAction.label}
        </GreButton>
        {#if summary.secondaryAction && secondaryLabel}
            <GreButton
                on:click={(event) => {
                    if (onSecondary) {
                        onSecondary();
                        return;
                    }
                    runAction(summary.secondaryAction!, event);
                }}
            >
                {secondaryLabel}
            </GreButton>
        {/if}
    </GreButtonRow>
</section>
