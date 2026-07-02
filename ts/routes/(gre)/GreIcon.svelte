<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    export let name:
        | "dashboard"
        | "study"
        | "practice"
        | "progress"
        | "settings"
        | "memory"
        | "performance"
        | "readiness"
        | "score"
        | "calendar"
        | "topic"
        | "check"
        | "alert";
    export let size: number | "sm" | "md" | "lg" | "xl" = "md";

    const sizeMap = {
        sm: 16,
        md: 18,
        lg: 20,
        xl: 22,
    } as const;

    $: pixelSize = typeof size === "number" ? size : sizeMap[size];
</script>

<svg
    class="gre-icon gre-icon-{typeof size === 'number' ? 'md' : size}"
    width={pixelSize}
    height={pixelSize}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.75"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
>
    {#if name === "dashboard"}
        <rect x="3" y="3" width="7" height="7" rx="1.5" />
        <rect x="14" y="3" width="7" height="7" rx="1.5" />
        <rect x="3" y="14" width="7" height="7" rx="1.5" />
        <rect x="14" y="14" width="7" height="7" rx="1.5" />
    {:else if name === "study"}
        <path d="M4 6.5A2.5 2.5 0 0 1 6.5 4H18v16H6.5A2.5 2.5 0 0 1 4 17.5z" />
        <path d="M8 4v16" />
    {:else if name === "practice"}
        <path d="M9 11l2 2 4-4" />
        <path d="M7 3h10a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" />
    {:else if name === "progress"}
        <path d="M4 18V6" />
        <path d="M10 18V10" />
        <path d="M16 18V4" />
        <path d="M22 18H2" />
    {:else if name === "settings"}
        <circle cx="12" cy="12" r="3" />
        <path
            d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"
        />
    {:else if name === "memory"}
        <path d="M12 3a5 5 0 0 0-5 5v2a5 5 0 1 0 10 0V8a5 5 0 0 0-5-5z" />
        <path d="M8 14h8" />
    {:else if name === "performance"}
        <circle cx="12" cy="12" r="8" />
        <circle cx="12" cy="12" r="3" />
        <path d="M12 4v2M12 18v2" />
    {:else if name === "readiness"}
        <path d="M5 20V8l7-4 7 4v12" />
        <path d="M9 20v-6h6v6" />
    {:else if name === "score"}
        <path d="M7 7h10v10H7z" />
        <path d="M9 12h6M12 9v6" />
    {:else if name === "calendar"}
        <rect x="3" y="5" width="18" height="16" rx="2" />
        <path d="M16 3v4M8 3v4M3 10h18" />
    {:else if name === "topic"}
        <path d="M4 6h16v12H4z" />
        <path d="M8 10h8M8 14h5" />
    {:else if name === "check"}
        <path d="M20 6 9 17l-5-5" />
    {:else if name === "alert"}
        <path d="M12 8v4M12 16h.01" />
        <path d="M10.3 4.3 2.6 18a1 1 0 0 0 .9 1.5h16a1 1 0 0 0 .9-1.5L13.7 4.3a1 1 0 0 0-1.8 0z" />
    {/if}
</svg>
