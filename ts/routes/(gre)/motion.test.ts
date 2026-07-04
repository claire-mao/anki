// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, it } from "vitest";

import { greMotionDuration } from "./motion";

describe("greMotionDuration", () => {
    it("returns the requested duration in non-browser test env", () => {
        expect(greMotionDuration(160)).toBe(160);
    });
});
