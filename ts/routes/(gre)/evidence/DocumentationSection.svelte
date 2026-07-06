<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand } from "@tslib/bridgecommand";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import { documentationNavAction } from "../gre-navigation";
    import type { DocumentationEvidencePresentation } from "./build-sync-presentation";
    import { greVerificationDocBridgeCommand } from "../settings/verification-presentation";

    export let model: DocumentationEvidencePresentation;
</script>

<section class="evidence-section" aria-labelledby="evidence-documentation-heading">
    <header class="evidence-section-header">
        <h2 class="gre-section-title" id="evidence-documentation-heading">Documentation</h2>
        <p class="evidence-section-lead">{model.description}</p>
    </header>

    <GreButtonRow className="evidence-documentation-actions">
        <GreButton navAction={documentationNavAction("Open documentation")}>
            Open documentation
        </GreButton>
    </GreButtonRow>

    <div class="evidence-doc-links">
        <h3 class="evidence-subheading">Key documents</h3>
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
</section>
