// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { goto } from "$app/navigation";
import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";

export type GreNavId =
    | "dashboard"
    | "studyPlan"
    | "study"
    | "practice"
    | "progress"
    | "settings";

export type GreNavIcon =
    | "dashboard"
    | "calendar"
    | "study"
    | "practice"
    | "progress"
    | "settings";

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
    studyPlan: {
        id: "studyPlan",
        label: "Study plan",
        page: "study-plan",
        bridge: "greOpenStudyPlan",
        icon: "calendar",
    },
    study: {
        id: "study",
        label: "Study",
        page: "review",
        bridge: "greOpenStudy",
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

/** Primary workflow nav; settings stays accessible but off the main learning path. */
export const grePrimaryNavItems: GreNavItem[] = [
    greNavById.dashboard,
    greNavById.studyPlan,
    greNavById.study,
    greNavById.practice,
    greNavById.progress,
];

export const greUtilityNavItems: GreNavItem[] = [greNavById.settings];

/** All nav items (primary + utility). */
export const greNavItems: GreNavItem[] = [
    ...grePrimaryNavItems,
    ...greUtilityNavItems,
];

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

/** Standard primary-action labels — keep wording consistent across GRE Atlas. */
export const GRE_CTA_REVIEW = "Review flashcards";
export const GRE_CTA_PRACTICE = "Practice questions";
export const GRE_CTA_STUDY_PLAN = "View study plan";
export const GRE_CTA_STUDY_TOPIC = "Study topic";
export const GRE_CTA_PRACTICE_TOPIC = "Practice topic";
export const GRE_CTA_BROWSE_DECK = "Browse GRE deck";

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

export function studyPlanNavAction(label = GRE_CTA_STUDY_PLAN): GreNavAction {
    return {
        label,
        bridge: greNavById.studyPlan.bridge,
        href: greNavHref(greNavById.studyPlan),
    };
}

export function greDeckOptionsAction(): GreNavAction {
    return {
        label: "Open GRE deck options",
        bridge: "greOpenDeckOptions",
    };
}

export function greBrowseDeckAction(label = GRE_CTA_BROWSE_DECK): GreNavAction {
    return {
        label,
        bridge: "greBrowseGreDeck",
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
