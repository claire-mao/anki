// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

@MainActor
final class GREAtlasSyncSession: ObservableObject {
    @Published private(set) var status: GREAtlasSyncStatusView?
    @Published private(set) var lastResult: GREAtlasPerformSyncView?
    @Published private(set) var collectionResult: GRECollectionSyncView?
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

        let auth = GREAtlasSyncAuthInput(
            hkey: credentials.hkey,
            endpoint: credentials.endpoint,
            ioTimeoutSecs: credentials.ioTimeoutSecs
        )

        do {
            // 1. Real collection sync: cards, notes, revlog, FSRS scheduling,
            //    and statistics converge with the desktop.
            let collection = try await engine.syncCollection(auth: auth)
            collectionResult = collection

            // 2. Practice/calibration sidecar sync (question attempts, readiness).
            let result = try await engine.performGREAtlasSync(auth: auth)
            lastResult = result
            status = try await engine.loadGREAtlasSyncStatus()

            // Refresh every surface so freshly-synced review state is visible.
            await engine.refreshStudy()
            await engine.refreshProgress()
            await engine.refreshDashboard()

            if collection.fullSyncRequired {
                error = "This device needs a full sync. Choose upload or download to continue."
            } else if !collection.success {
                error = collection.serverMessage.isEmpty
                    ? "Collection sync failed."
                    : collection.serverMessage
            } else if result.success {
                error = nil
            } else {
                error = result.message.isEmpty ? "Practice sync failed." : result.message
            }
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// Resolve an ambiguous full sync by explicitly choosing a direction.
    func resolveFullSync(
        using engine: AnkiMobileEngine,
        credentials: GreAtlasSyncCredentials,
        upload: Bool
    ) async {
        guard !syncing else { return }
        syncing = true
        error = nil
        defer { syncing = false }

        let auth = GREAtlasSyncAuthInput(
            hkey: credentials.hkey,
            endpoint: credentials.endpoint,
            ioTimeoutSecs: credentials.ioTimeoutSecs
        )
        do {
            let collection = try await engine.syncCollection(
                auth: auth,
                fullSyncChoice: upload ? "upload" : "download"
            )
            collectionResult = collection
            await engine.refreshStudy()
            await engine.refreshProgress()
            await engine.refreshDashboard()
            error = collection.success ? nil : collection.serverMessage
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
