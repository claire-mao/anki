// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import XCTest
@testable import GREAtlasCompanion

/// End-to-end review flow through the shared Rust backend (mobile_bridge FFI).
final class MobileBridgeReviewTests: XCTestCase {
    func testStudyReviewDecrementsDueCountsAndRefreshesDashboard() throws {
        let fileManager = FileManager.default
        let tempRoot = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        defer { try? fileManager.removeItem(at: tempRoot) }

        let supportRoot = tempRoot.appendingPathComponent("GRE Atlas", isDirectory: true)
        try fileManager.createDirectory(at: supportRoot, withIntermediateDirectories: true)

        guard let bundleRoot = Bundle.main.url(forResource: "DemoBundle", withExtension: nil) else {
            XCTFail("DemoBundle missing from app bundle")
            return
        }

        let paths = CollectionPaths(
            collectionPath: supportRoot.appendingPathComponent("collection.anki2").path,
            mediaFolderPath: supportRoot.appendingPathComponent("collection.media").path,
            mediaDbPath: supportRoot.appendingPathComponent("collection.mdb").path
        )
        try DemoCollectionInstaller.installBundledDemo(at: paths, bundleRoot: bundleRoot, fileManager: fileManager)

        let client = MobileBridgeClient()
        try client.createBackend()
        try client.openCollection(
            collectionPath: paths.collectionPath,
            mediaFolderPath: paths.mediaFolderPath,
            mediaDbPath: paths.mediaDbPath
        )
        _ = try client.prepareDemoCollection()

        let dashboardBefore = try client.loadDashboard()
        XCTAssertTrue(dashboardBefore.deckExists)
        XCTAssertEqual(dashboardBefore.deckName, "GRE Atlas")
        XCTAssertGreaterThanOrEqual(dashboardBefore.dueTotal, 1)

        let studyBefore = try client.loadStudy()
        XCTAssertTrue(studyBefore.deckExists)
        XCTAssertGreaterThanOrEqual(studyBefore.dueTotal, 1)

        let reviewStart = try client.loadStudyReview()
        XCTAssertTrue(reviewStart.sessionActive)
        XCTAssertFalse(reviewStart.sessionComplete)
        guard let card = reviewStart.card else {
            XCTFail("Expected a queued study card")
            return
        }
        XCTAssertFalse(card.questionHtml.isEmpty)
        XCTAssertFalse(card.answerHtml.isEmpty)
        XCTAssertFalse(card.buttons.isEmpty)

        let reviewAfter = try client.answerStudyCard(
            GreStudyAnswerInput(
                cardId: card.cardId,
                rating: 2,
                millisecondsTaken: 1200
            )
        )
        let queuesChanged =
            reviewAfter.dueNew != reviewStart.dueNew
            || reviewAfter.dueLearn != reviewStart.dueLearn
            || reviewAfter.dueReview != reviewStart.dueReview
            || reviewAfter.sessionComplete
            || reviewAfter.card?.cardId != card.cardId
        XCTAssertTrue(queuesChanged, "Expected scheduler to advance after grading")

        let dashboardAfter = try client.loadDashboard()
        XCTAssertTrue(dashboardAfter.deckExists)
        let dashboardChanged =
            dashboardAfter.dueNew != dashboardBefore.dueNew
            || dashboardAfter.dueLearn != dashboardBefore.dueLearn
            || dashboardAfter.dueReview != dashboardBefore.dueReview
            || dashboardAfter.memoryStudiedCards > dashboardBefore.memoryStudiedCards
        XCTAssertTrue(dashboardChanged, "Expected dashboard due counts to refresh after a review")
    }

    func testExplainAnswerReturnsStructuredExplanationWithCitation() throws {
        let fileManager = FileManager.default
        let tempRoot = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        defer { try? fileManager.removeItem(at: tempRoot) }

        let supportRoot = tempRoot.appendingPathComponent("GRE Atlas", isDirectory: true)
        try fileManager.createDirectory(at: supportRoot, withIntermediateDirectories: true)

        guard let bundleRoot = Bundle.main.url(forResource: "DemoBundle", withExtension: nil) else {
            XCTFail("DemoBundle missing from app bundle")
            return
        }

        let paths = CollectionPaths(
            collectionPath: supportRoot.appendingPathComponent("collection.anki2").path,
            mediaFolderPath: supportRoot.appendingPathComponent("collection.media").path,
            mediaDbPath: supportRoot.appendingPathComponent("collection.mdb").path
        )
        try DemoCollectionInstaller.installBundledDemo(at: paths, bundleRoot: bundleRoot, fileManager: fileManager)

        let client = MobileBridgeClient()
        try client.createBackend()
        try client.openCollection(
            collectionPath: paths.collectionPath,
            mediaFolderPath: paths.mediaFolderPath,
            mediaDbPath: paths.mediaDbPath
        )
        _ = try client.prepareDemoCollection()

        let bootstrap = try client.loadPracticeBootstrap()
        guard let question = bootstrap.questions.first else {
            XCTFail("Expected at least one practice question")
            return
        }
        let selected = question.choices.first ?? "A"

        let explanation = try client.explainAnswer(
            GreExplainAnswerInput(questionId: question.id, selectedAnswer: selected)
        )
        XCTAssertFalse(explanation.summary.isEmpty)
        XCTAssertEqual(explanation.choices.filter(\.isCorrect).count, 1)
        XCTAssertFalse(explanation.citationSourceName.isEmpty)
        if PracticePresentation.isOfflineTemplateProvenance(explanation.provenance) {
            XCTAssertEqual(
                PracticePresentation.resolveExplanationProvenanceNote(
                    provenance: explanation.provenance,
                    provenanceNote: explanation.provenanceNote
                ),
                PracticePresentation.offlineTemplateNote
            )
        }
    }
}
