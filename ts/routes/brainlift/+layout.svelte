<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import "./brainlift.scss";

    const nav = [
        { href: "/brainlift/review", label: "Review" },
        { href: "/brainlift/practice", label: "Practice" },
        { href: "/brainlift/readiness", label: "Readiness" },
        { href: "/brainlift/study-plan", label: "Study Plan" },
    ];
</script>

<div class="brainlift-shell">
    <header class="brainlift-header">
        <div class="brand">BrainLift</div>
        <nav class="brainlift-nav">
            {#each nav as item}
                <a class="nav-link" href={item.href}>{item.label}</a>
            {/each}
        </nav>
    </header>
    <main class="brainlift-main">
        <slot />
    </main>
</div>
