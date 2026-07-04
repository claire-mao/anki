// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

@MainActor
final class GREAtlasSyncSession: ObservableObject {
    @Published private(set) var status: GREAtlasSyncStatusView?
    @Published private(set) var lastResult: GREAtlasPerformSyncView?
    @Published private(set) var loadingStatus = false
    @Published private(set) var syncing = false
    @Published var error: String?

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

    /// Download remote changes, merge locally, and upload pending practice data.
    func syncNow(using engine: AnkiMobileEngine, credentials: GreAtlasSyncCredentials?) async {
        guard !syncing else { return }

        syncing = true
        error = nil
        lastResult = nil
        await Task.yield()
        defer { syncing = false }

        guard let credentials else {
            error = GreAtlasSyncCredentials.missingCredentialsMessage
            return
        }

        guard let endpoint = credentials.endpoint, !endpoint.isEmpty else {
            error = "Enter sync server URL (e.g. http://127.0.0.1:8080/)."
            return
        }

        do {
            let result = try await engine.performGREAtlasSync(
                auth: GREAtlasSyncAuthInput(
                    hkey: credentials.hkey,
                    endpoint: credentials.endpoint,
                    ioTimeoutSecs: credentials.ioTimeoutSecs
                )
            )
            lastResult = result
            status = try await engine.loadGREAtlasSyncStatus()
            await engine.refreshProgress()
            await engine.refreshDashboard()

            if result.success {
                error = nil
            } else {
                error = result.message.isEmpty ? "Sync failed." : result.message
            }
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// Best-effort background sync on launch/resume.
    func autoSyncIfConfigured(using engine: AnkiMobileEngine) async {
        guard let credentials = GreAtlasSyncCredentials.load() else { return }
        await syncNow(using: engine, credentials: credentials)
    }
}
