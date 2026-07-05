// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PerformGreAtlasSyncResponse } from "@generated/anki/brainlift_pb";
import type { SyncAuth } from "@generated/anki/sync_pb";
import { getBrainLiftSyncStatus, performGreAtlasSync } from "@generated/backend";
import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";

/** Best-effort GRE Atlas sidecar sync. Never throws. */
export async function syncGreAtlasPractice(options?: {
    auth?: SyncAuth;
    silent?: boolean;
}): Promise<PerformGreAtlasSyncResponse | null> {
    try {
        if (bridgeCommandsAvailable()) {
            bridgeCommand("grePerformGreAtlasSync");
            return null;
        }
        return await performGreAtlasSync({ auth: options?.auth });
    } catch {
        if (!options?.silent) {
            // Caller may surface UI; auto-sync paths pass silent: true.
        }
        return null;
    }
}

export async function loadGreAtlasSyncStatus() {
    return getBrainLiftSyncStatus({});
}

/** Fire-and-forget sync used after practice/review sessions. */
export function scheduleGreAtlasAutoSync(): void {
    void syncGreAtlasPractice({ silent: true });
}
