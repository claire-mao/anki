// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// Loads GRE pages through the shared Rust backend (`mobile_bridge` → `Backend::run_service_method`).
@MainActor
final class AnkiMobileEngine: ObservableObject {
    @Published private(set) var dashboard: GreDashboardView?
    @Published private(set) var progress: GreProgressView?
    @Published private(set) var practice: GrePracticeBootstrapView?
    @Published private(set) var study: GreStudyView?
    @Published private(set) var isLoading = false
    @Published var lastError: String?

    private var bridge = MobileBridgeClient()
    private var bootstrapped = false

    func bootstrapIfNeeded() async {
        guard !bootstrapped else { return }
        await refreshAllPages()
    }

    func refreshAllPages() async {
        isLoading = true
        lastError = nil
        defer { isLoading = false }

        do {
            let pages = try await Task.detached(priority: .userInitiated) { [bridge] in
                try Self.loadPages(bridge: bridge)
            }.value
            dashboard = pages.dashboard
            progress = pages.progress
            practice = pages.practice
            study = pages.study
            bootstrapped = true
        } catch {
            lastError = error.localizedDescription
        }
    }

    nonisolated private static func loadPages(bridge: MobileBridgeClient) throws -> GrePageBundle {
        let paths = CollectionPaths.default
        try paths.ensureParentDirectoryExists()
        try bridge.createBackend()
        try bridge.openCollection(
            collectionPath: paths.collectionPath,
            mediaFolderPath: paths.mediaFolderPath,
            mediaDbPath: paths.mediaDbPath
        )
        return try bridge.loadGrePages()
    }
}

private struct CollectionPaths {
    let collectionPath: String
    let mediaFolderPath: String
    let mediaDbPath: String

    static var `default`: CollectionPaths {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let root = support.appendingPathComponent("BrainLift", isDirectory: true)
        let collection = root.appendingPathComponent("collection.anki2")
        return CollectionPaths(
            collectionPath: collection.path,
            mediaFolderPath: collection.deletingPathExtension().appendingPathExtension("media").path,
            mediaDbPath: collection.deletingPathExtension().appendingPathExtension("mdb").path
        )
    }

    func ensureParentDirectoryExists() throws {
        let url = URL(fileURLWithPath: collectionPath).deletingLastPathComponent()
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    }
}
