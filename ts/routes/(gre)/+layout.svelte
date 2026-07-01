<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import "./gre.scss";

    const nav = [
        { href: "/dashboard", label: "Dashboard" },
        { href: "/review", label: "Review" },
        { href: "/practice", label: "Practice" },
        { href: "/study-plan", label: "Study Plan" },
        { href: "/readiness", label: "Readiness" },
    ];
</script>

<div class="gre-shell">
    <header class="gre-header">
        <div class="brand">GRE</div>
        <nav class="gre-nav">
            {#each nav as item}
                <a class="nav-link" href={item.href}>{item.label}</a>
            {/each}
        </nav>
    </header>
    <main class="gre-main">
        <slot />
    </main>
</div>
