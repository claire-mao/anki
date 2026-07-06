// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { goto } from "$app/navigation";
import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";

export type GreNavId =
    | "dashboard"
    | "study"
    | "practice"
    | "progress"
    | "evidence"
    | "settings";

export type GreNavIcon =
    | "dashboard"
    | "study"
    | "practice"
    | "progress"
    | "readiness"
    | "info"
    | "settings";

export type GreSubmissionNavId =
    | "evidence"
    | "analytics"
    | "readiness"
    | "practice"
    | "study"
    | "documentation";

export type GreSubmissionNavItem = {
    id: GreSubmissionNavId;
    label: string;
    page: string;
    bridge: string;
    icon: GreNavIcon;
};

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
    evidence: {
        id: "evidence",
        label: "Evidence",
        page: "evidence",
        bridge: "greOpenEvidence",
        icon: "info",
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
    greNavById.study,
    greNavById.practice,
    greNavById.progress,
];

export const greUtilityNavItems: GreNavItem[] = [
    greNavById.evidence,
    greNavById.settings,
];

/** All nav items (primary + utility). */
export const greNavItems: GreNavItem[] = [
    ...grePrimaryNavItems,
    ...greUtilityNavItems,
];

/**
 * Grader/demo navigation shown in submission mode, ordered as the intended
 * review journey: Evidence → Documentation → Practice → Analytics → Readiness.
 * Study is kept at the end (not part of the core grader flow).
 */
export const greSubmissionNavItems: GreSubmissionNavItem[] = [
    {
        id: "evidence",
        label: "Evidence",
        page: "evidence",
        bridge: "greOpenEvidence",
        icon: "info",
    },
    {
        id: "documentation",
        label: "Documentation",
        page: "documentation",
        bridge: "greOpenDocumentation",
        icon: "study",
    },
    {
        id: "practice",
        label: "Practice",
        page: "practice",
        bridge: "greOpenPractice",
        icon: "practice",
    },
    {
        id: "analytics",
        label: "Analytics",
        page: "progress",
        bridge: "greOpenProgress",
        icon: "progress",
    },
    {
        id: "readiness",
        label: "Readiness",
        page: "readiness",
        bridge: "greOpenReadiness",
        icon: "readiness",
    },
    {
        id: "study",
        label: "Study",
        page: "review",
        bridge: "greOpenStudy",
        icon: "study",
    },
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

export function isGreSubmissionNavActive(
    item: GreSubmissionNavItem,
    pathname: string,
): boolean {
    const href = `/${item.page}`;
    return pathname === href || pathname.startsWith(`${href}/`);
}

export function greSubmissionNavAction(item: GreSubmissionNavItem): GreNavAction {
    return {
        label: item.label,
        bridge: item.bridge,
        href: `/${item.page}`,
    };
}

export type GreNavAction = {
    label: string;
    bridge?: string;
    href?: string;
};

/** Standard primary-action labels — keep wording consistent across GRE Atlas. */
export const GRE_CTA_REVIEW = "Review flashcards";
export const GRE_CTA_STUDY_AHEAD = "Study ahead";
export const GRE_CTA_PRACTICE = "Practice questions";
export const GRE_CTA_STUDY_PLAN = "View dashboard";
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
        ...greNavAction(greNavById.dashboard),
        label,
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

export function methodologyNavAction(label = "How GRE Atlas estimates your score"): GreNavAction {
    return {
        label,
        bridge: "greOpenMethodology",
        href: "/methodology",
    };
}

export function documentationNavAction(label = "Documentation"): GreNavAction {
    return {
        label,
        bridge: "greOpenDocumentation",
        href: "/documentation",
    };
}

export function studyFeatureExperimentNavAction(
    label = "Study Feature Experiment",
): GreNavAction {
    return {
        label,
        bridge: "greOpenStudyFeatureExperiment",
        href: "/study-feature-experiment",
    };
}

export function readinessNavAction(label = "Readiness score details"): GreNavAction {
    return {
        label,
        bridge: "greOpenReadiness",
        href: "/readiness",
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
