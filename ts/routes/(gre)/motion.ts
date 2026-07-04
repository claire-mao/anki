// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { browser } from "$app/environment";

/** Respect Reduce Motion for Svelte transitions (0 ms when reduced). */
export function greMotionDuration(ms: number): number {
    if (!browser) {
        return ms;
    }
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : ms;
}
