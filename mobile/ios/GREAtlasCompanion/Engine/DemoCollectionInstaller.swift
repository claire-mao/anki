// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum DemoCollectionInstallError: LocalizedError {
    case missingBundleResource(String)
    case missingBundledCollection

    var errorDescription: String? {
        switch self {
        case .missingBundleResource(let name):
            return "Bundled demo resource missing from app: \(name)"
        case .missingBundledCollection:
            return "Bundled demo collection.anki2 is missing from the app bundle."
        }
    }
}

enum DemoCollectionInstaller {
    /// Folder copied into the app bundle by Xcode (see `Resources/DemoBundle`).
    static let bundleSubdirectory = "DemoBundle"

    /// Sidecar SQLite filename beside the collection (matches Rust `GRE_ATLAS_DB_NAME`).
    static let greAtlasDbFileName = "greatlas.db"
    /// Legacy bundled sidecar filename; copied as `greatlas.db` when present.
    static let legacyGreAtlasDbFileName = "brainlift.db"

    /// Copies the bundled demo collection into Application Support when no local file exists yet.
    static func installIfNeeded(at paths: CollectionPaths, fileManager: FileManager = .default) throws {
        guard !fileManager.fileExists(atPath: paths.collectionPath) else { return }

        guard let bundleRoot = Bundle.main.url(
            forResource: bundleSubdirectory,
            withExtension: nil
        ) else {
            throw DemoCollectionInstallError.missingBundleResource(bundleSubdirectory)
        }

        try installBundledDemo(at: paths, bundleRoot: bundleRoot, fileManager: fileManager)
    }

    /// Test seam: copies demo files from a bundle root directory.
    static func installBundledDemo(
        at paths: CollectionPaths,
        bundleRoot: URL,
        fileManager: FileManager = .default
    ) throws {
        let destinationRoot = URL(fileURLWithPath: paths.collectionPath).deletingLastPathComponent()
        try fileManager.createDirectory(at: destinationRoot, withIntermediateDirectories: true)

        let bundledCollection = bundleRoot.appendingPathComponent("collection.anki2")
        guard fileManager.fileExists(atPath: bundledCollection.path) else {
            throw DemoCollectionInstallError.missingBundledCollection
        }

        try copyItem(
            from: bundledCollection,
            to: destinationRoot.appendingPathComponent("collection.anki2"),
            fileManager: fileManager
        )

        if fileManager.fileExists(atPath: destinationRoot.appendingPathComponent("collection.mdb").path) == false {
            let bundledMdb = bundleRoot.appendingPathComponent("collection.mdb")
            if fileManager.fileExists(atPath: bundledMdb.path) {
                try copyItem(
                    from: bundledMdb,
                    to: destinationRoot.appendingPathComponent("collection.mdb"),
                    fileManager: fileManager
                )
            }
        }

        try copyGreAtlasDb(from: bundleRoot, to: destinationRoot, fileManager: fileManager)

        let bundledMedia = bundleRoot.appendingPathComponent("collection.media", isDirectory: true)
        if fileManager.fileExists(atPath: bundledMedia.path) {
            try copyItem(
                from: bundledMedia,
                to: URL(fileURLWithPath: paths.mediaFolderPath, isDirectory: true),
                fileManager: fileManager
            )
        } else {
            try fileManager.createDirectory(
                atPath: paths.mediaFolderPath,
                withIntermediateDirectories: true
            )
        }
    }

    private static func copyGreAtlasDb(
        from bundleRoot: URL,
        to destinationRoot: URL,
        fileManager: FileManager
    ) throws {
        let destination = destinationRoot.appendingPathComponent(greAtlasDbFileName)
        for sourceName in [greAtlasDbFileName, legacyGreAtlasDbFileName] {
            let source = bundleRoot.appendingPathComponent(sourceName)
            if fileManager.fileExists(atPath: source.path) {
                try copyItem(from: source, to: destination, fileManager: fileManager)
                return
            }
        }
    }

    private static func copyItem(from source: URL, to destination: URL, fileManager: FileManager) throws {
        if fileManager.fileExists(atPath: destination.path) {
            try fileManager.removeItem(at: destination)
        }
        try fileManager.copyItem(at: source, to: destination)
    }
}
