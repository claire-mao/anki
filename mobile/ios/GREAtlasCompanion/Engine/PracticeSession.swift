// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

enum PracticeSectionFilter: String, CaseIterable, Identifiable {
    case all
    case quant
    case verbal
    case awa

    var id: String { rawValue }

    var label: String {
        switch self {
        case .all: "All sections"
        case .quant: "Quant"
        case .verbal: "Verbal"
        case .awa: "AWA"
        }
    }
}

extension GrePracticeScoreStripView {
    init(from bootstrap: GrePracticeBootstrapView) {
        self.init(
            memoryValue: bootstrap.memoryValue,
            memoryLow: bootstrap.memoryLow,
            memoryHigh: bootstrap.memoryHigh,
            memorySufficient: bootstrap.memorySufficient,
            memoryDetail: bootstrap.memoryDetail,
            performanceValue: bootstrap.performanceValue,
            performanceLow: bootstrap.performanceLow,
            performanceHigh: bootstrap.performanceHigh,
            performanceSufficient: bootstrap.performanceSufficient,
            performanceDetail: bootstrap.performanceDetail,
            performanceAttemptCount: bootstrap.performanceAttemptCount
        )
    }
}

@MainActor
final class PracticeSession: ObservableObject {
    @Published private(set) var scoreStrip: GrePracticeScoreStripView?
    @Published var sectionFilter: PracticeSectionFilter = .all
    @Published private(set) var queue: [GreQuestionView] = []
    @Published private(set) var questionIndex = 0
    @Published var selectedAnswer = ""
    @Published private(set) var attemptResult: GreRecordAttemptResultView?
    @Published private(set) var responseTimeMs: UInt = 0
    @Published private(set) var sessionComplete = false
    @Published private(set) var attemptsRecorded = 0
    @Published private(set) var submitting = false
    @Published var submitError: String?

    private var sessionId = ""
    private var questionById: [String: GreQuestionView] = [:]
    private var startedAt = Date()

    var currentQuestion: GreQuestionView? {
        guard !sessionComplete, questionIndex < queue.count else { return nil }
        return queue[questionIndex]
    }

    var progressPercent: Int {
        guard !queue.isEmpty else { return 0 }
        if sessionComplete { return 100 }
        return Int((Double(questionIndex + 1) / Double(queue.count) * 100).rounded())
    }

    var progressLabel: String {
        guard !queue.isEmpty else { return "No questions for this filter" }
        if sessionComplete { return "Session complete" }
        return "Question \(questionIndex + 1) of \(queue.count)"
    }

    func syncBootstrap(_ bootstrap: GrePracticeBootstrapView) {
        guard bootstrap.sessionId != sessionId else { return }
        sessionId = bootstrap.sessionId
        questionById = Dictionary(uniqueKeysWithValues: bootstrap.questions.map { ($0.id, $0) })
        scoreStrip = GrePracticeScoreStripView(from: bootstrap)
        sectionFilter = .all
        attemptsRecorded = 0
        applySectionFilter(.all, queues: bootstrap.queuesBySection)
    }

    func applySectionFilter(_ filter: PracticeSectionFilter, queues: GrePracticeQueuesView) {
        sectionFilter = filter
        let ids: [String]
        switch filter {
        case .all: ids = queues.all
        case .quant: ids = queues.quant
        case .verbal: ids = queues.verbal
        case .awa: ids = queues.awa
        }
        queue = ids.compactMap { questionById[$0] }
        questionIndex = 0
        sessionComplete = queue.isEmpty
        resetQuestionState()
    }

    func applySectionFilter(_ filter: PracticeSectionFilter, from bootstrap: GrePracticeBootstrapView) {
        applySectionFilter(filter, queues: bootstrap.queuesBySection)
    }

    func resetQuestionState() {
        selectedAnswer = ""
        startedAt = Date()
        attemptResult = nil
        responseTimeMs = 0
        submitError = nil
    }

    func submit(using engine: AnkiMobileEngine) async {
        guard let question = currentQuestion,
              !selectedAnswer.isEmpty,
              !submitting,
              attemptResult == nil else { return }

        submitting = true
        submitError = nil
        let elapsedMs = UInt(max(0, Date().timeIntervalSince(startedAt) * 1000))
        defer { submitting = false }

        do {
            let result = try await engine.recordPracticeAttempt(
                questionId: question.id,
                answer: selectedAnswer,
                responseTimeMs: elapsedMs,
                sessionId: sessionId
            )
            attemptResult = result
            responseTimeMs = elapsedMs
            attemptsRecorded += 1
            scoreStrip = try await engine.refreshPracticeScoreStrip()
        } catch {
            submitError = error.localizedDescription
        }
    }

    func nextQuestion() {
        resetQuestionState()
        let nextIndex = questionIndex + 1
        if nextIndex >= queue.count {
            sessionComplete = true
            return
        }
        questionIndex = nextIndex
    }

    func restart(from bootstrap: GrePracticeBootstrapView) {
        syncBootstrap(bootstrap)
    }
}
