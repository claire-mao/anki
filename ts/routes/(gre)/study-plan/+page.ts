// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { redirect } from "@sveltejs/kit";

import type { PageLoad } from "./$types";

export const load = (() => {
    redirect(302, "/home");
}) satisfies PageLoad;
