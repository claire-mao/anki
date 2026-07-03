// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import XCTest
@testable import GREAtlasCompanion

final class DemoCollectionInstallerTests: XCTestCase {
    func testInstallBundledDemoCopiesCollectionAndSidecarDb() throws {
        let fileManager = FileManager.default
        let tempRoot = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        defer { try? fileManager.removeItem(at: tempRoot) }

        let bundleRoot = tempRoot.appendingPathComponent("DemoBundle", isDirectory: true)
        let supportRoot = tempRoot.appendingPathComponent("Support/GRE Atlas", isDirectory: true)
        try fileManager.createDirectory(at: bundleRoot, withIntermediateDirectories: true)
        try fileManager.createDirectory(at: supportRoot, withIntermediateDirectories: true)

        let bundledCollection = bundleRoot.appendingPathComponent("collection.anki2")
        let bundledGreAtlasDb = bundleRoot.appendingPathComponent("greatlas.db")
        try Data("demo-collection".utf8).write(to: bundledCollection)
        try Data("demo-greatlas".utf8).write(to: bundledGreAtlasDb)

        let paths = CollectionPaths(
            collectionPath: supportRoot.appendingPathComponent("collection.anki2").path,
            mediaFolderPath: supportRoot.appendingPathComponent("collection.media").path,
            mediaDbPath: supportRoot.appendingPathComponent("collection.mdb").path
        )

        try DemoCollectionInstaller.installBundledDemo(at: paths, bundleRoot: bundleRoot, fileManager: fileManager)

        XCTAssertTrue(fileManager.fileExists(atPath: paths.collectionPath))
        XCTAssertTrue(fileManager.fileExists(atPath: supportRoot.appendingPathComponent("greatlas.db").path))
        XCTAssertTrue(fileManager.fileExists(atPath: paths.mediaFolderPath))
    }

    func testInstallBundledDemoCopiesLegacySidecarDbAsGreatlas() throws {
        let fileManager = FileManager.default
        let tempRoot = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        defer { try? fileManager.removeItem(at: tempRoot) }

        let bundleRoot = tempRoot.appendingPathComponent("DemoBundle", isDirectory: true)
        let supportRoot = tempRoot.appendingPathComponent("Support/GRE Atlas", isDirectory: true)
        try fileManager.createDirectory(at: bundleRoot, withIntermediateDirectories: true)
        try fileManager.createDirectory(at: supportRoot, withIntermediateDirectories: true)

        let bundledCollection = bundleRoot.appendingPathComponent("collection.anki2")
        let bundledLegacyDb = bundleRoot.appendingPathComponent("brainlift.db")
        try Data("demo-collection".utf8).write(to: bundledCollection)
        try Data("demo-legacy".utf8).write(to: bundledLegacyDb)

        let paths = CollectionPaths(
            collectionPath: supportRoot.appendingPathComponent("collection.anki2").path,
            mediaFolderPath: supportRoot.appendingPathComponent("collection.media").path,
            mediaDbPath: supportRoot.appendingPathComponent("collection.mdb").path
        )

        try DemoCollectionInstaller.installBundledDemo(at: paths, bundleRoot: bundleRoot, fileManager: fileManager)

        let destDb = supportRoot.appendingPathComponent("greatlas.db")
        XCTAssertTrue(fileManager.fileExists(atPath: destDb.path))
        XCTAssertEqual(try String(contentsOf: destDb), "demo-legacy")
    }
}
