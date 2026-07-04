// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export type GreMetricStructuredValue = {
    headline: string;
    details?: string[];
    /** Section breakdowns read better as inline chips; other metrics use a vertical list. */
    detailLayout?: "stack" | "chips";
};
