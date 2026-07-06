// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import {
    greSubmissionNavAction,
    greSubmissionNavItems,
    isGreSubmissionNavActive,
} from "./gre-navigation";

describe("submission navigation", () => {
    test("exposes the grader-facing nav set", () => {
        expect(greSubmissionNavItems.map((item) => item.label)).toEqual([
            "Evidence",
            "Documentation",
            "Practice",
            "Analytics",
            "Readiness",
            "Study",
        ]);
    });

    test("maps analytics to the progress page", () => {
        const analytics = greSubmissionNavItems.find((item) => item.id === "analytics");
        expect(analytics?.page).toBe("progress");
        expect(isGreSubmissionNavActive(analytics!, "/progress")).toBe(true);
        expect(greSubmissionNavAction(analytics!).href).toBe("/progress");
    });

    test("maps documentation to the documentation page", () => {
        const documentation = greSubmissionNavItems.find(
            (item) => item.id === "documentation",
        );
        expect(documentation?.page).toBe("documentation");
        expect(isGreSubmissionNavActive(documentation!, "/documentation")).toBe(true);
    });
});
