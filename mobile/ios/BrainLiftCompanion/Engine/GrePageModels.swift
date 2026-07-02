// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// JSON view models returned by `mobile_bridge` GRE page loaders (`gre_pages.rs`).
/// Field names match desktop Svelte page data derived from the same RPC bundles.

struct GreDashboardView: Codable, Equatable {
    let readinessProjected: Double?
    let readinessLow: Double?
    let readinessHigh: Double?
    let readinessSufficient: Bool
    let readinessSummary: String
    let estimatedGreCombined: UInt?
    let estimatedGreLow: UInt?
    let estimatedGreHigh: UInt?
    let estimatedGrePreliminary: Bool
    let dailyPlanHeadline: String
    let dailyPlanTaskCount: UInt
    let studyPlanSummary: String
    let weakTopicName: String?
    let deckExists: Bool
    let deckName: String
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
}

struct GreProgressView: Codable, Equatable {
    let memoryValue: Double?
    let memoryLow: Double?
    let memoryHigh: Double?
    let memorySufficient: Bool
    let performanceValue: Double?
    let performanceLow: Double?
    let performanceHigh: Double?
    let performanceSufficient: Bool
    let readinessProjected: Double?
    let readinessLow: Double?
    let readinessHigh: Double?
    let readinessSufficient: Bool
    let estimatedGreCombined: UInt?
    let estimatedGreLow: UInt?
    let estimatedGreHigh: UInt?
    let weightedCoverage: Double
    let studiedTopics: UInt
    let calibrationAssessment: String
}

struct GrePracticeBootstrapView: Codable, Equatable {
    let sessionId: String
    let questionCount: UInt
    let memoryValue: Double?
    let performanceValue: Double?
    let performanceSufficient: Bool
}

struct GreStudyView: Codable, Equatable {
    let deckExists: Bool
    let deckName: String
    let dueNew: UInt
    let dueLearn: UInt
    let dueReview: UInt
    let dueTotal: UInt
}

struct GrePageBundle: Equatable {
    let dashboard: GreDashboardView
    let progress: GreProgressView
    let practice: GrePracticeBootstrapView
    let study: GreStudyView
}
