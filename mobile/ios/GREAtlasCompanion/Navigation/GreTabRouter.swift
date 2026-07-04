// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

/// Shared tab selection so dashboard CTAs can jump to Study or Practice.
@MainActor
final class GreTabRouter: ObservableObject {
    @Published var selectedTab: GreNavTab = .dashboard
    @Published var openStudyPlanOnDashboard = false
    @Published var pendingPracticeTopicId: String?
    @Published var pendingPracticeTopicTitle: String?

    func open(_ tab: GreNavTab) {
        selectedTab = tab
    }

    func openStudyPlan() {
        selectedTab = .dashboard
        openStudyPlanOnDashboard = true
    }

    func openPractice(topicId: String? = nil, topicTitle: String? = nil) {
        if let topicId {
            let trimmed = topicId.trimmingCharacters(in: .whitespacesAndNewlines)
            pendingPracticeTopicId = trimmed.isEmpty ? nil : trimmed
        } else {
            pendingPracticeTopicId = nil
        }
        if let topicTitle {
            let trimmed = topicTitle.trimmingCharacters(in: .whitespacesAndNewlines)
            pendingPracticeTopicTitle = trimmed.isEmpty ? nil : trimmed
        } else {
            pendingPracticeTopicTitle = nil
        }
        selectedTab = .practice
    }
}
