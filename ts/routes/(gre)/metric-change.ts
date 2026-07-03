// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { type MetricChanges, presentMetricChanges } from "./metric-change-presentation";
import {
    extractGreMetricSnapshot,
    loadGreMetricSnapshot,
    type MetricSnapshotInput,
    saveGreMetricSnapshot,
} from "./metric-snapshot";

export type { MetricChangeEvidence, MetricChangePresentation, MetricChanges } from "./metric-change-presentation";
export type { GreMetricSnapshot, MetricSnapshotInput } from "./metric-snapshot";

export function commitMetricSnapshot(input: MetricSnapshotInput): MetricChanges {
    const previous = loadGreMetricSnapshot();
    const current = extractGreMetricSnapshot(input);
    const changes = presentMetricChanges(previous, current, input.recentActivity);
    saveGreMetricSnapshot(current);
    return changes;
}
