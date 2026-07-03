<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import GreSkeleton from "./GreSkeleton.svelte";
</script>

<div
    class="gre-page-skeleton"
    role="status"
    aria-live="polite"
    aria-label="Loading page"
>
    <div class="gre-page-skeleton-header">
        <GreSkeleton width="2.5rem" height="2.5rem" shape="md" />
        <div class="gre-page-skeleton-header-text">
            <GreSkeleton width="40%" height="1.75rem" shape="md" />
            <GreSkeleton width="62%" height="0.875rem" />
        </div>
    </div>

    <div class="gre-page-skeleton-grid">
        {#each [0, 1, 2] as _}
            <div class="gre-page-skeleton-card">
                <GreSkeleton width="55%" height="0.75rem" shape="pill" />
                <GreSkeleton width="45%" height="1.5rem" shape="md" />
                <GreSkeleton width="100%" height="0.375rem" shape="pill" />
            </div>
        {/each}
    </div>

    <GreSkeleton width="100%" height="10rem" shape="lg" />
    <GreSkeleton width="100%" height="8rem" shape="lg" />
</div>
