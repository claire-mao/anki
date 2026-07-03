// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

@MainActor
final class GREAtlasSyncSession: ObservableObject {
    @Published private(set) var status: GREAtlasSyncStatusView?
    @Published private(set) var lastPull: GREAtlasSyncPullView?
    @Published private(set) var lastPush: GREAtlasSyncPushView?
    @Published private(set) var loadingStatus = false
    @Published private(set) var pulling = false
    @Published private(set) var pushing = false
    @Published var error: String?

    var exportedAttempts: [GREAtlasSyncAttemptView] {
        lastPull?.attempts ?? []
    }

    func refreshStatus(using engine: AnkiMobileEngine) async {
        loadingStatus = true
        error = nil
        defer { loadingStatus = false }

        do {
            status = try await engine.loadGREAtlasSyncStatus()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func pull(using engine: AnkiMobileEngine) async {
        pulling = true
        error = nil
        defer { pulling = false }

        do {
            let current = try await engine.loadGREAtlasSyncStatus()
            status = current
            let afterUsn = max(0, current.currentUsn - Int32(current.pendingUploadCount))
            lastPull = try await engine.pullGREAtlasChanges(
                afterUsn: afterUsn,
                limit: 500
            )
            status = try await engine.loadGREAtlasSyncStatus()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func pushExported(using engine: AnkiMobileEngine) async {
        let attempts = exportedAttempts
        guard !attempts.isEmpty else {
            error = "Pull practice changes before pushing them to another device."
            return
        }
        await push(attempts: attempts, using: engine)
    }

    func push(attempts: [GREAtlasSyncAttemptView], using engine: AnkiMobileEngine) async {
        pushing = true
        error = nil
        defer { pushing = false }

        do {
            lastPush = try await engine.pushGREAtlasChanges(attempts: attempts)
            status = try await engine.loadGREAtlasSyncStatus()
            await engine.refreshProgress()
            await engine.refreshDashboard()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
