// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { goto } from "$app/navigation";
import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";

export type GreNavId = "dashboard" | "study" | "practice" | "progress" | "settings";

export type GreNavIcon = "dashboard" | "study" | "practice" | "progress" | "settings";

export type GreNavItem = {
    id: GreNavId;
    label: string;
    page: string;
    bridge: string;
    icon: GreNavIcon;
};

/** Slugs passed to Qt `load_sveltekit_page()` — keep in sync with `open_gre_page()`. */
const greNavById: Record<GreNavId, GreNavItem> = {
    dashboard: {
        id: "dashboard",
        label: "Dashboard",
        page: "home",
        bridge: "greOpenDashboard",
        icon: "dashboard",
    },
    study: {
        id: "study",
        label: "Study",
        page: "review",
        bridge: "greStartReview",
        icon: "study",
    },
    practice: {
        id: "practice",
        label: "Practice",
        page: "practice",
        bridge: "greOpenPractice",
        icon: "practice",
    },
    progress: {
        id: "progress",
        label: "Progress",
        page: "progress",
        bridge: "greOpenProgress",
        icon: "progress",
    },
    settings: {
        id: "settings",
        label: "Settings",
        page: "settings",
        bridge: "greOpenSettings",
        icon: "settings",
    },
};

export const greNavItems: GreNavItem[] = Object.values(greNavById);

export function greNavItem(id: GreNavId): GreNavItem {
    return greNavById[id];
}

export function greNavHref(item: Pick<GreNavItem, "page">): string {
    return `/${item.page}`;
}

export function isGreNavActive(item: GreNavItem, pathname: string): boolean {
    if (item.id === "dashboard") {
        return pathname === "/home" || pathname === "/dashboard";
    }
    const href = greNavHref(item);
    return pathname === href || pathname.startsWith(`${href}/`);
}

export type GreNavAction = {
    label: string;
    bridge?: string;
    href?: string;
};

export function greNavAction(item: GreNavItem): GreNavAction {
    return {
        label: item.label,
        bridge: item.bridge,
        href: greNavHref(item),
    };
}

export function settingsNavAction(): GreNavAction {
    return greNavAction(greNavItem("settings"));
}

export function greDeckOptionsAction(): GreNavAction {
    return {
        label: "Open GRE deck options",
        bridge: "greOpenDeckOptions",
    };
}

/** Prefer Qt bridge navigation (full page load) over SvelteKit client routing in Anki. */
export function runGreNavAction(action: GreNavAction, event?: Event): void {
    event?.preventDefault();
    event?.stopPropagation();

    if (action.bridge && bridgeCommandsAvailable()) {
        bridgeCommand(action.bridge);
        return;
    }

    if (action.href) {
        void goto(action.href);
    }
}
