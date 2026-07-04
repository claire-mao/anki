// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

private enum PracticeSessionLimits {
    /// Matches `DAILY_PRACTICE_TARGET` in `rslib/src/gre_atlas/study_plan.rs`.
    static let topicSessionSize = 5
}

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
    @Published private(set) var topicFilter = ""
    @Published private(set) var topicTitle = ""
    @Published private(set) var queue: [GreQuestionView] = []
    @Published private(set) var questionIndex = 0
    @Published private(set) var questionsCompleted = 0
    @Published var selectedAnswer = ""
    @Published private(set) var attemptResult: GreRecordAttemptResultView?
    @Published private(set) var responseTimeMs: UInt = 0
    @Published private(set) var sessionComplete = false
    @Published private(set) var attemptsRecorded = 0
    @Published private(set) var sessionAttempts: [PracticeAttemptRecord] = []
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
        let number = min(questionsCompleted + 1, queue.count)
        return Int((Double(number) / Double(queue.count) * 100).rounded())
    }

    var progressLabel: String {
        guard !queue.isEmpty else { return "No questions here" }
        if sessionComplete { return "Session complete" }
        let number = min(questionsCompleted + 1, queue.count)
        return "Question \(number) of \(queue.count)"
    }

    func syncBootstrap(_ bootstrap: GrePracticeBootstrapView) {
        guard bootstrap.sessionId != sessionId else { return }
        sessionId = bootstrap.sessionId
        questionById = Dictionary(uniqueKeysWithValues: bootstrap.questions.map { ($0.id, $0) })
        scoreStrip = GrePracticeScoreStripView(from: bootstrap)
        sectionFilter = .all
        attemptsRecorded = 0
        sessionAttempts = []
        rebuildQueue(from: bootstrap.queuesBySection)
    }

    func applyTopicFocus(
        topicId: String,
        topicTitle: String?,
        from bootstrap: GrePracticeBootstrapView
    ) {
        topicFilter = topicId.trimmingCharacters(in: .whitespacesAndNewlines)
        self.topicTitle = topicTitle?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        sectionFilter = .all
        rebuildQueue(from: bootstrap.queuesBySection)
    }

    func clearTopicFocus(from bootstrap: GrePracticeBootstrapView) {
        topicFilter = ""
        topicTitle = ""
        rebuildQueue(from: bootstrap.queuesBySection)
    }

    func applySectionFilter(_ filter: PracticeSectionFilter, queues: GrePracticeQueuesView) {
        sectionFilter = filter
        rebuildQueue(from: queues)
    }

    private func rebuildQueue(from queues: GrePracticeQueuesView) {
        let ids: [String]
        switch sectionFilter {
        case .all: ids = queues.all
        case .quant: ids = queues.quant
        case .verbal: ids = queues.verbal
        case .awa: ids = queues.awa
        }
        var nextQueue = ids.compactMap { questionById[$0] }
        if !topicFilter.isEmpty {
            nextQueue = nextQueue.filter { question in
                question.topic == topicFilter || question.topic.hasPrefix("\(topicFilter)::")
            }
            nextQueue = Self.buildTopicPracticeQueue(nextQueue)
        }
        queue = nextQueue
        questionIndex = 0
        questionsCompleted = 0
        sessionComplete = queue.isEmpty
        attemptsRecorded = 0
        sessionAttempts = []
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
            questionsCompleted += 1
            sessionAttempts.append(
                PracticeAttemptRecord(topic: result.topic, correct: result.correct)
            )
            scoreStrip = try await engine.refreshPracticeScoreStrip()
        } catch {
            submitError = "Could not record this attempt. Please try again."
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

    func skipQuestion() {
        questionsCompleted += 1
        nextQuestion()
    }

    func restart(from bootstrap: GrePracticeBootstrapView) {
        syncBootstrap(bootstrap)
    }

    private static func buildTopicPracticeQueue(
        _ questions: [GreQuestionView],
        sessionSize: Int = PracticeSessionLimits.topicSessionSize
    ) -> [GreQuestionView] {
        guard !questions.isEmpty, sessionSize > 0 else { return [] }
        if questions.count >= sessionSize {
            return Array(questions.prefix(sessionSize))
        }

        var queue: [GreQuestionView] = []
        queue.reserveCapacity(sessionSize)
        for index in 0..<sessionSize {
            queue.append(questions[index % questions.count])
        }
        return queue
    }
}
