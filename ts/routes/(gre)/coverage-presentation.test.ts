// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, expect, test } from "vitest";

import { COVERAGE_EXPLANATION } from "./coverage-presentation";

describe("coverage explanation", () => {
    test("describes what counts as covered and how weighting works", () => {
        expect(COVERAGE_EXPLANATION).toMatch(/reviewed its flashcards/i);
        expect(COVERAGE_EXPLANATION).toMatch(/practice alone/i);
        expect(COVERAGE_EXPLANATION).toMatch(/weighted/i);
        expect(COVERAGE_EXPLANATION).toMatch(/47%/);
        expect(COVERAGE_EXPLANATION).toMatch(/simple average/i);
        expect(COVERAGE_EXPLANATION).toMatch(/three section numbers/i);
    });
});
