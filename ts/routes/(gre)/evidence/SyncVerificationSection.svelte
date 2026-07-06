<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand } from "@tslib/bridgecommand";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import type { SyncVerificationPresentation } from "./build-sync-presentation";
    import { greVerificationDocBridgeCommand } from "../settings/verification-presentation";

    export let model: SyncVerificationPresentation;
</script>

<section class="evidence-section" aria-labelledby="evidence-sync-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-sync-heading">Sync Verification</h2>
        <p class="evidence-section-lead">
            Live sync state from the GRE Atlas sidecar and documented cross-device
            verification scenarios.
        </p>
    </header>

    {#if !model.available}
        <p class="evidence-section-empty">{model.emptyMessage}</p>
    {:else}
        <div class="evidence-metrics">
            {#each model.rows as row (row.label)}
                <GreMetricRow label={row.label} value={row.value} />
            {/each}
        </div>

        {#if model.docLinks.length > 0}
            <div class="evidence-doc-links">
                <h3 class="evidence-subheading">Reference</h3>
                <ul class="evidence-doc-link-list">
                    {#each model.docLinks as link (link.id)}
                        <li>
                            <button
                                type="button"
                                class="evidence-doc-link"
                                on:click={() => bridgeCommand(greVerificationDocBridgeCommand(link))}
                            >
                                {link.label}
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}
    {/if}
</section>
