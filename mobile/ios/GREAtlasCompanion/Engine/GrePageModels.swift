// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// JSON view models returned by `mobile_bridge` GRE page loaders (`gre_pages.rs`).
/// Field names match desktop Svelte page data derived from the same RPC bundles.

struct GreCoverageSectionView: Codable, Equatable {
    let label: String
    let percent: UInt
}

struct GreCoverageView: Codable, Equatable {
    let weightedRatio: Double
    let unweightedRatio: Double
    let catalogLeafCount: UInt
    let coveredLeafCount: UInt
    let sections: [GreCoverageSectionView]
    let uncoveredStudyLabels: [String]
    let readinessAvailable: Bool
    let readinessReason: String
}

struct GreTopicInsightView: Codable, Equatable {
    let topicId: String
    let displayName: String
    let section: String
    let examWeight: Double
    let memoryScore: Double?
    let practiceAccuracy: Double?
    let studiedCards: UInt
    let covered: Bool
    let reason: String
    let studyLabel: String
}

struct GreAttemptView: Codable, Equatable {
    let topic: String
    let correct: Bool
    let responseTimeMs: UInt
    let answeredAtSecs: Int64
}

struct GreDailyTaskView: Codable, Equatable {
    let id: String
    let title: String
    let detail: String
    let targetCount: UInt
    let topicId: String?
    let topicDisplayName: String?
}

struct GreQuestionView: Codable, Equatable {
    let id: String
    let topic: String
    let section: String
    let format: String
    let stem: String
    let choices: [String]
}

struct GreTopicMasteryView: Codable, Equatable {
    let topicId: String
    let displayName: String
    let avgRetrievability: Double
}

struct GreAbstentionRequirementView: Codable, Equatable {
    let id: String
    let label: String
    let status: String
    let nextStep: String
    let met: Bool
}

struct GreDashboardView: Codable, Equatable {
    let computedAtMillis: Int64
    let readinessProjected: Double?
    let readinessLow: Double?
    let readinessHigh: Double?
    let readinessSufficient: Bool
    let readinessSummary: String
    let readinessEvidenceSummary: String
    let readinessAbstainReason: String
    let readinessAbstentionRequirements: [GreAbstentionRequirementView]
    let readinessConfidenceLevel: String
    let memoryValue: Double?
    let memoryLow: Double?
    let memoryHigh: Double?
    let memorySufficient: Bool
    let memoryDetail: String
    let memoryAbstainReason: String
    let memoryAbstentionRequirements: [GreAbstentionRequirementView]
    let memoryStudiedCards: UInt
    let performanceValue: Double?
    let performanceLow: Double?
    let performanceHigh: Double?
    let performanceSufficient: Bool
    let performanceDetail: String
    let performanceAbstainReason: String
    let performanceAbstentionRequirements: [GreAbstentionRequirementView]
    let performanceAttemptCount: UInt
    let estimatedGreCombined: UInt?
    let estimatedGreLow: UInt?
    let estimatedGreHigh: UInt?
    let estimatedGrePreliminary: Bool
    let coverage: GreCoverageView
    let dailyPlanHeadline: String
    let dailyPlanTaskCount: UInt
    let dailyPlanRationale: String
    let dailyPlanTasks: [GreDailyTaskView]
    let studyPlanSummary: String
    let weakTopic: GreTopicInsightView?
    let weakTopicName: String?
    let recommendedTopics: [GreTopicInsightView]
    let recentActivity: [GreAttemptView]
    let recentAccuracyTrend: [Double]
    let deckExists: Bool
    let deckName: String
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
    let dueTotal: UInt
}

struct GreProgressView: Codable, Equatable {
    let computedAtMillis: Int64
    let memoryValue: Double?
    let memoryLow: Double?
    let memoryHigh: Double?
    let memorySufficient: Bool
    let memoryDetail: String
    let performanceValue: Double?
    let performanceLow: Double?
    let performanceHigh: Double?
    let performanceSufficient: Bool
    let performanceDetail: String
    let performanceAttemptCount: UInt
    let readinessProjected: Double?
    let readinessLow: Double?
    let readinessHigh: Double?
    let readinessSufficient: Bool
    let readinessSummary: String
    let readinessConfidenceLevel: String
    let estimatedGreCombined: UInt?
    let estimatedGreLow: UInt?
    let estimatedGreHigh: UInt?
    let estimatedGrePreliminary: Bool
    let estimatedGreConfidence: String
    let weightedCoverage: Double
    let unweightedCoverage: Double
    let catalogLeafCount: UInt
    let coveredLeafCount: UInt
    let coverage: GreCoverageView
    let studiedCards: UInt
    let topicCount: UInt
    let masteredCards: UInt
    let calibrationAssessment: String
    let calibrationWellCalibrated: Bool
    let practiceTrend: [Double]
    let recentActivity: [GreAttemptView]
    let weakTopics: [GreTopicInsightView]
    let topicMastery: [GreTopicMasteryView]
}

struct GrePracticeQueuesView: Codable, Equatable {
    let all: [String]
    let quant: [String]
    let verbal: [String]
    let awa: [String]
}

struct GrePracticeScoreStripView: Codable, Equatable {
    let memoryValue: Double?
    let memoryLow: Double?
    let memoryHigh: Double?
    let memorySufficient: Bool
    let memoryDetail: String
    let performanceValue: Double?
    let performanceLow: Double?
    let performanceHigh: Double?
    let performanceSufficient: Bool
    let performanceDetail: String
    let performanceAttemptCount: UInt
}

struct GreRecordAttemptInput: Codable, Equatable {
    let questionId: String
    let answer: String
    let responseTimeMs: UInt
    let sessionId: String
}

struct GreRecordAttemptResultView: Codable, Equatable {
    let correct: Bool
    let explanation: String
    let topic: String
}

struct GreExplainAnswerInput: Codable, Equatable {
    let questionId: String
    let selectedAnswer: String
}

struct GreAnswerChoiceExplanationView: Codable, Equatable {
    let choice: String
    let isCorrect: Bool
    let reasoning: String
}

struct GreAnswerExplanationView: Codable, Equatable {
    let summary: String
    let choices: [GreAnswerChoiceExplanationView]
    let correctAnswer: String
    let citationSourceName: String
    let citationSourceSection: String
    let citationExcerpt: String
    let provenance: String
    let provenanceNote: String
    let modelVersion: String
}

struct GrePracticeBootstrapView: Codable, Equatable {
    let sessionId: String
    let questionCount: UInt
    let questions: [GreQuestionView]
    let queue: [String]
    let queuesBySection: GrePracticeQueuesView
    let quantCount: UInt
    let verbalCount: UInt
    let awaCount: UInt
    let memoryValue: Double?
    let memoryLow: Double?
    let memoryHigh: Double?
    let memorySufficient: Bool
    let memoryDetail: String
    let performanceValue: Double?
    let performanceLow: Double?
    let performanceHigh: Double?
    let performanceSufficient: Bool
    let performanceDetail: String
    let performanceAttemptCount: UInt
}

struct GreStudyView: Codable, Equatable {
    let deckExists: Bool
    let deckName: String
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
    let dueTotal: UInt
    let availableNewCount: UInt
    let extraStudyAvailable: UInt
    let nextReviewInDays: UInt?
}

struct GreStudyGradeButtonView: Codable, Equatable {
    let rating: UInt
    let label: String
}

struct GreStudyCardView: Codable, Equatable {
    let cardId: Int64
    let queue: String
    let questionHtml: String
    let answerHtml: String
    let css: String
    let buttons: [GreStudyGradeButtonView]
}

struct GreStudyReviewView: Codable, Equatable {
    let deckExists: Bool
    let deckName: String
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
    let dueTotal: UInt
    let sessionActive: Bool
    let sessionComplete: Bool
    let card: GreStudyCardView?
}

struct GreStudyAnswerInput: Codable, Equatable {
    let cardId: Int64
    let rating: UInt
    let millisecondsTaken: UInt
}

struct GREAtlasSyncStatusView: Codable, Equatable {
    let currentUsn: Int32
    let pendingUploadCount: UInt
    let lastModifiedSecs: Int64
}

struct GREAtlasSyncAttemptView: Codable, Equatable {
    let id: Int64
    let questionId: String
    let topic: String
    let difficulty: Float?
    let answeredAtSecs: Int64
    let answer: String
    let correct: Bool
    let responseTimeMs: UInt
    let confidence: UInt?
    let sessionId: String?
    let usn: Int32
    let mtimeSecs: Int64
}

struct GREAtlasSyncPullInput: Codable, Equatable {
    let afterUsn: Int32
    let limit: UInt
}

struct GREAtlasSyncPullView: Codable, Equatable {
    let attempts: [GREAtlasSyncAttemptView]
    let currentUsn: Int32
}

struct GREAtlasSyncPushInput: Codable, Equatable {
    let attempts: [GREAtlasSyncAttemptView]
}

struct GREAtlasSyncConflictView: Codable, Equatable, Identifiable {
    var id: Int64 { attemptId }
    let attemptId: Int64
    let reason: String
    let kept: GREAtlasSyncAttemptView
    let rejected: GREAtlasSyncAttemptView
}

struct GREAtlasSyncPushView: Codable, Equatable {
    let currentUsn: Int32
    let appliedCount: UInt
    let conflicts: [GREAtlasSyncConflictView]
}

struct GREAtlasSyncAuthInput: Codable, Equatable {
    let hkey: String
    let endpoint: String?
    let ioTimeoutSecs: UInt32?
}

struct GREAtlasPerformSyncInput: Codable, Equatable {
    let auth: GREAtlasSyncAuthInput?
}

struct GREAtlasPerformSyncView: Codable, Equatable {
    let success: Bool
    let message: String
    let downloadedCount: UInt
    let uploadedCount: UInt
    let appliedCount: UInt
    let conflicts: [GREAtlasSyncConflictView]
    let status: GREAtlasSyncStatusView?
}

/// Input for a real Anki collection sync (cards, notes, revlog, scheduling).
struct GRECollectionSyncInput: Codable, Equatable {
    let auth: GREAtlasSyncAuthInput
    /// "upload" or "download"; only needed when the server reports FULL_SYNC.
    let fullSyncChoice: String?
}

struct GRECollectionSyncView: Codable, Equatable {
    let success: Bool
    /// "noChanges", "normalSync", "fullUpload", "fullDownload", "fullSyncRequired".
    let outcome: String
    let fullSyncRequired: Bool
    /// When true, a full up/download ran and the collection must be reopened.
    let reopenRequired: Bool
    let serverMessage: String
    let hostNumber: UInt
}

struct GrePageBundle: Equatable {
    let dashboard: GreDashboardView
    let progress: GreProgressView
    let practice: GrePracticeBootstrapView
    let study: GreStudyView
}

struct GreDemoCollectionView: Codable, Equatable {
    let deckName: String
    let deckCreated: Bool
    let cardsAdded: UInt
    let practiceAttemptsAdded: UInt
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
    let dueTotal: UInt
}

struct GreVerificationDocLinkView: Codable, Equatable {
    let id: String
    let label: String
    let relativePath: String
}

struct GreVerificationRowView: Codable, Equatable {
    let label: String
    let value: String
}

struct GreVerificationView: Codable, Equatable {
    let rows: [GreVerificationRowView]
    let docLinks: [GreVerificationDocLinkView]
}
