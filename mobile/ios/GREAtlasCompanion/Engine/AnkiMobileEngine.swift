// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

struct CollectionPaths {
    let collectionPath: String
    let mediaFolderPath: String
    let mediaDbPath: String

    static var `default`: CollectionPaths {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let root = support.appendingPathComponent("GRE Atlas", isDirectory: true)
        let collection = root.appendingPathComponent("collection.anki2")
        return CollectionPaths(
            collectionPath: collection.path,
            mediaFolderPath: collection.deletingPathExtension().appendingPathExtension("media").path,
            mediaDbPath: collection.deletingPathExtension().appendingPathExtension("mdb").path
        )
    }

    init(collectionPath: String, mediaFolderPath: String, mediaDbPath: String) {
        self.collectionPath = collectionPath
        self.mediaFolderPath = mediaFolderPath
        self.mediaDbPath = mediaDbPath
    }

    func ensureParentDirectoryExists() throws {
        let url = URL(fileURLWithPath: collectionPath).deletingLastPathComponent()
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    }
}

/// Serializes access to the C FFI backend (not thread-safe).
actor GreBridgeLoader {
    private let client = MobileBridgeClient()
    private var ready = false

    private func ensureReady() throws {
        if ready { return }
        let paths = CollectionPaths.default
        try paths.ensureParentDirectoryExists()
        try DemoCollectionInstaller.installIfNeeded(at: paths)
        try client.createBackend()
        try client.openCollection(
            collectionPath: paths.collectionPath,
            mediaFolderPath: paths.mediaFolderPath,
            mediaDbPath: paths.mediaDbPath
        )
        ready = true
    }

    func loadDashboard() throws -> GreDashboardView {
        try ensureReady()
        return try client.loadDashboard()
    }

    func loadProgress() throws -> GreProgressView {
        try ensureReady()
        return try client.loadProgress()
    }

    func loadPracticeBootstrap() throws -> GrePracticeBootstrapView {
        try ensureReady()
        return try client.loadPracticeBootstrap()
    }

    func recordPracticeAttempt(_ input: GreRecordAttemptInput) throws -> GreRecordAttemptResultView {
        try ensureReady()
        return try client.recordPracticeAttempt(input)
    }

    func explainAnswer(_ input: GreExplainAnswerInput) throws -> GreAnswerExplanationView {
        try ensureReady()
        return try client.explainAnswer(input)
    }

    func loadPracticeScoreStrip() throws -> GrePracticeScoreStripView {
        try ensureReady()
        return try client.loadPracticeScoreStrip()
    }

    func loadStudy() throws -> GreStudyView {
        try ensureReady()
        return try client.loadStudy()
    }

    func loadVerification() throws -> GreVerificationView {
        try ensureReady()
        return try client.loadVerification()
    }

    func loadStudyReview() throws -> GreStudyReviewView {
        try ensureReady()
        return try client.loadStudyReview()
    }

    func answerStudyCard(_ input: GreStudyAnswerInput) throws -> GreStudyReviewView {
        try ensureReady()
        return try client.answerStudyCard(input)
    }

    func loadGREAtlasSyncStatus() throws -> GREAtlasSyncStatusView {
        try ensureReady()
        return try client.loadGREAtlasSyncStatus()
    }

    func pullGREAtlasChanges(_ input: GREAtlasSyncPullInput) throws -> GREAtlasSyncPullView {
        try ensureReady()
        return try client.pullGREAtlasChanges(input)
    }

    func pushGREAtlasChanges(_ input: GREAtlasSyncPushInput) throws -> GREAtlasSyncPushView {
        try ensureReady()
        return try client.pushGREAtlasChanges(input)
    }

    func performGREAtlasSync(_ input: GREAtlasPerformSyncInput) throws -> GREAtlasPerformSyncView {
        try ensureReady()
        return try client.performGREAtlasSync(input)
    }

    func syncCollection(_ input: GRECollectionSyncInput) throws -> GRECollectionSyncView {
        try ensureReady()
        let result = try client.syncCollection(input)
        // A full up/download closes the collection; reopen it in place so the
        // freshly-synced review state is visible without restarting the app.
        if result.reopenRequired {
            let paths = CollectionPaths.default
            try client.openCollection(
                collectionPath: paths.collectionPath,
                mediaFolderPath: paths.mediaFolderPath,
                mediaDbPath: paths.mediaDbPath
            )
        }
        return result
    }

    func prepareDemoCollection() throws -> GreDemoCollectionView {
        try ensureReady()
        return try client.prepareDemoCollection()
    }
}

/// Loads GRE pages through the shared Rust backend (`mobile_bridge` → `Backend::run_service_method`).
@MainActor
final class AnkiMobileEngine: ObservableObject {
    @Published private(set) var dashboard: GreDashboardView?
    @Published private(set) var progress: GreProgressView?
    @Published private(set) var practice: GrePracticeBootstrapView?
    @Published private(set) var study: GreStudyView?
    @Published private(set) var demo: GreDemoCollectionView?

    @Published private(set) var dashboardLoading = false
    @Published private(set) var progressLoading = false
    @Published private(set) var practiceLoading = false
    @Published private(set) var studyLoading = false

    @Published private(set) var dashboardError: String?
    @Published private(set) var progressError: String?
    @Published private(set) var practiceError: String?
    @Published private(set) var studyError: String?

    var isLoading: Bool {
        dashboardLoading || progressLoading || practiceLoading || studyLoading
    }

    private let loader = GreBridgeLoader()
    private var bootstrapped = false

    func bootstrapIfNeeded() async {
        guard !bootstrapped else { return }
        await prepareDemoCollection()
        await refreshAllPages()
        bootstrapped = true
    }

    func prepareDemoCollection() async {
        do {
            demo = try await loader.prepareDemoCollection()
        } catch {
            dashboardError = error.localizedDescription
        }
    }

    func refreshAllPages() async {
        async let dashboardTask: Void = refreshDashboard()
        async let progressTask: Void = refreshProgress()
        async let practiceTask: Void = refreshPractice()
        async let studyTask: Void = refreshStudy()
        _ = await (dashboardTask, progressTask, practiceTask, studyTask)
    }

    func refreshAfterStudyGrade() async {
        async let dashboardTask: Void = refreshDashboard()
        async let studyTask: Void = refreshStudy()
        _ = await (dashboardTask, studyTask)
    }

    func refreshDashboard() async {
        dashboardLoading = true
        dashboardError = nil
        defer { dashboardLoading = false }

        do {
            dashboard = try await loader.loadDashboard()
        } catch {
            dashboardError = error.localizedDescription
        }
    }

    func refreshProgress() async {
        progressLoading = true
        progressError = nil
        defer { progressLoading = false }

        do {
            progress = try await loader.loadProgress()
        } catch {
            progressError = error.localizedDescription
        }
    }

    func refreshPractice() async {
        practiceLoading = true
        practiceError = nil
        defer { practiceLoading = false }

        do {
            practice = try await loader.loadPracticeBootstrap()
        } catch {
            practiceError = error.localizedDescription
        }
    }

    func recordPracticeAttempt(
        questionId: String,
        answer: String,
        responseTimeMs: UInt,
        sessionId: String
    ) async throws -> GreRecordAttemptResultView {
        try await loader.recordPracticeAttempt(
            GreRecordAttemptInput(
                questionId: questionId,
                answer: answer,
                responseTimeMs: responseTimeMs,
                sessionId: sessionId
            )
        )
    }

    func explainAnswer(questionId: String, selectedAnswer: String) async throws -> GreAnswerExplanationView {
        try await loader.explainAnswer(
            GreExplainAnswerInput(questionId: questionId, selectedAnswer: selectedAnswer)
        )
    }

    func refreshPracticeScoreStrip() async throws -> GrePracticeScoreStripView {
        try await loader.loadPracticeScoreStrip()
    }

    func refreshStudy() async {
        studyLoading = true
        studyError = nil
        defer { studyLoading = false }

        do {
            study = try await loader.loadStudy()
        } catch {
            studyError = error.localizedDescription
        }
    }

    func startStudyReview() async throws -> GreStudyReviewView {
        try await loader.loadStudyReview()
    }

    func startExtraStudyReview() async throws -> GreStudyReviewView {
        try await loader.loadStudyExtraReview()
    }

    func answerStudyCard(
        cardId: Int64,
        rating: UInt,
        millisecondsTaken: UInt
    ) async throws -> GreStudyReviewView {
        try await loader.answerStudyCard(
            GreStudyAnswerInput(
                cardId: cardId,
                rating: rating,
                millisecondsTaken: millisecondsTaken
            )
        )
    }

    func loadGREAtlasSyncStatus() async throws -> GREAtlasSyncStatusView {
        try await loader.loadGREAtlasSyncStatus()
    }

    func pullGREAtlasChanges(afterUsn: Int32, limit: UInt = 500) async throws -> GREAtlasSyncPullView {
        try await loader.pullGREAtlasChanges(
            GREAtlasSyncPullInput(afterUsn: afterUsn, limit: limit)
        )
    }

    func pushGREAtlasChanges(attempts: [GREAtlasSyncAttemptView]) async throws -> GREAtlasSyncPushView {
        try await loader.pushGREAtlasChanges(
            GREAtlasSyncPushInput(attempts: attempts)
        )
    }

    func performGREAtlasSync(auth: GREAtlasSyncAuthInput?) async throws -> GREAtlasPerformSyncView {
        try await loader.performGREAtlasSync(GREAtlasPerformSyncInput(auth: auth))
    }

    func syncCollection(
        auth: GREAtlasSyncAuthInput,
        fullSyncChoice: String? = nil
    ) async throws -> GRECollectionSyncView {
        try await loader.syncCollection(
            GRECollectionSyncInput(auth: auth, fullSyncChoice: fullSyncChoice)
        )
    }

    func reloadDemoCollection() async {
        await prepareDemoCollection()
        await refreshAllPages()
    }
}
