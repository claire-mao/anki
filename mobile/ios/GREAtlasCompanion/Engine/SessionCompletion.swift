// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

struct PracticeAttemptRecord: Equatable {
    let topic: String
    let correct: Bool
}

struct SessionCompletionRow: Equatable {
    let label: String
    let value: String
}

struct SessionCompletionSummary: Equatable {
    let headline: String
    let subline: String
    let rows: [SessionCompletionRow]
    let nextActionLabel: String
    let nextActionDetail: String
    let nextActionTab: GreNavTab?
    let nextActionTopicId: String?
    let usesExtraStudy: Bool
    let secondaryActionLabel: String
    let secondaryActionTab: GreNavTab
}

enum SessionCompletionBuilder {
    static func practiceSummary(from attempts: [PracticeAttemptRecord]) -> SessionCompletionSummary {
        let total = attempts.count
        let correct = attempts.filter(\.correct).count
        let stats = topicStats(from: attempts)
        let strongest = strongestTopic(from: stats)
        let weakest = weakestTopic(from: stats, strongest: strongest)

        var rows: [SessionCompletionRow] = [
            SessionCompletionRow(label: "Questions answered", value: "\(total)"),
        ]
        if total > 0 {
            rows.append(
                SessionCompletionRow(
                    label: "Accuracy",
                    value: ScoreFormat.formatPercent(Double(correct) / Double(total) * 100)
                )
            )
        }
        if let strongest {
            rows.append(SessionCompletionRow(label: "Strongest topic", value: strongest))
        }
        if let weakest {
            rows.append(SessionCompletionRow(label: "Focus next", value: weakest))
        }

        var nextActionLabel = "Practice again"
        var nextActionTab: GreNavTab? = .practice
        let nextActionTopicId: String? = nil
        var nextActionDetail = "Run another short set while this material is fresh."

        if let weakest {
            nextActionDetail =
                "Your weakest area this session was \(weakest). Another short set there will help it stick."
        } else if total > 0, correct == total {
            nextActionLabel = "Review flashcards"
            nextActionTab = .study
            nextActionDetail = "Strong accuracy — reinforce recall with a quick flashcard review."
        }

        return SessionCompletionSummary(
            headline: "Session complete",
            subline: "Here's how this set went.",
            rows: rows,
            nextActionLabel: nextActionLabel,
            nextActionDetail: nextActionDetail,
            nextActionTab: nextActionTab,
            nextActionTopicId: nextActionTopicId,
            usesExtraStudy: false,
            secondaryActionLabel: nextActionLabel == "Practice again" ? "View study plan" : "Practice again",
            secondaryActionTab: nextActionLabel == "Practice again" ? .dashboard : .practice
        )
    }

    static func studyCaughtUpSummary(
        weakTopic: GreTopicInsightView?,
        recommendedTopics: [GreTopicInsightView],
        dueTotal: UInt,
        studiedCards: UInt,
        extraStudyAvailable: UInt = 0,
        nextReviewInDays: UInt? = nil
    ) -> SessionCompletionSummary {
        let weakest = weakTopic?.displayName
        let strongest = strongestDashboardTopic(recommendedTopics: recommendedTopics, weakTopic: weakTopic)

        var rows: [SessionCompletionRow] = [
            SessionCompletionRow(label: "Cards due now", value: "\(dueTotal)"),
            SessionCompletionRow(label: "Flashcards reviewed", value: "\(studiedCards)"),
        ]
        if let strongest {
            rows.append(SessionCompletionRow(label: "Strongest area", value: strongest))
        }
        if let weakest {
            rows.append(SessionCompletionRow(label: "Focus next", value: weakest))
        }
        if dueTotal == 0, extraStudyAvailable == 0, let nextReviewInDays {
            rows.append(
                SessionCompletionRow(
                    label: "Next flashcard review",
                    value: nextReviewScheduleLabel(days: nextReviewInDays)
                )
            )
        }

        if extraStudyAvailable > 0 {
            let label = extraStudyAvailable == 1
                ? "Study 1 more card"
                : "Study \(extraStudyAvailable) more cards"
            return SessionCompletionSummary(
                headline: dueTotal == 0 ? "Review complete" : "Session complete",
                subline: dueTotal == 0
                    ? "You're caught up on flashcards due right now."
                    : "Nice pause point — you can pick up remaining cards later.",
                rows: rows,
                nextActionLabel: label,
                nextActionDetail:
                    "Unlock up to \(extraStudyAvailable) new flashcard\(extraStudyAvailable == 1 ? "" : "s") today (20 max per day) to build memory evidence without cramming.",
                nextActionTab: .study,
                nextActionTopicId: nil,
                usesExtraStudy: true,
                secondaryActionLabel: "Practice questions",
                secondaryActionTab: .practice
            )
        }

        var nextActionLabel = "Practice questions"
        var nextActionTab: GreNavTab? = .practice
        var nextActionTopicId: String? = nil
        var nextActionDetail = "Keep momentum with a short practice set."

        if let weakTopic {
            nextActionLabel = "Practice weak area"
            nextActionTopicId = weakTopic.topicId
            nextActionDetail = "Your next best step is a few questions on \(weakTopic.displayName)."
        } else {
            nextActionLabel = "View study plan"
            nextActionTab = nil
            nextActionDetail = "Open your study plan to pick the next focus area."
        }

        return SessionCompletionSummary(
            headline: dueTotal == 0 ? "Review complete" : "Session complete",
            subline: dueTotal == 0
                ? "You're caught up on flashcards due right now."
                : "Nice pause point — you can pick up remaining cards later.",
            rows: rows,
            nextActionLabel: nextActionLabel,
            nextActionDetail: nextActionDetail,
            nextActionTab: nextActionTab,
            nextActionTopicId: nextActionTopicId,
            usesExtraStudy: false,
            secondaryActionLabel: "View study plan",
            secondaryActionTab: .dashboard
        )
    }

    private static func nextReviewScheduleLabel(days: UInt) -> String {
        switch days {
        case 0:
            return "Flashcards due today"
        case 1:
            return "Flashcards due tomorrow"
        default:
            return "Flashcards due in \(days) days"
        }
    }

    private static func topicStats(from attempts: [PracticeAttemptRecord]) -> [String: (correct: Int, total: Int)] {
        var stats: [String: (correct: Int, total: Int)] = [:]
        for attempt in attempts {
            let topic = attempt.topic.trimmingCharacters(in: .whitespacesAndNewlines)
            let key = topic.isEmpty ? "Unknown topic" : topic
            var current = stats[key] ?? (0, 0)
            current.total += 1
            if attempt.correct { current.correct += 1 }
            stats[key] = current
        }
        return stats
    }

    private static func strongestTopic(from stats: [String: (correct: Int, total: Int)]) -> String? {
        stats.max { lhs, rhs in
            let leftAccuracy = Double(lhs.value.correct) / Double(lhs.value.total)
            let rightAccuracy = Double(rhs.value.correct) / Double(rhs.value.total)
            if leftAccuracy == rightAccuracy {
                return lhs.value.correct < rhs.value.correct
            }
            return leftAccuracy < rightAccuracy
        }?.key
    }

    private static func weakestTopic(
        from stats: [String: (correct: Int, total: Int)],
        strongest: String?
    ) -> String? {
        guard !stats.isEmpty else { return nil }
        guard let worst = stats.min(by: { lhs, rhs in
            let leftAccuracy = Double(lhs.value.correct) / Double(lhs.value.total)
            let rightAccuracy = Double(rhs.value.correct) / Double(rhs.value.total)
            if leftAccuracy == rightAccuracy {
                return lhs.value.total < rhs.value.total
            }
            return leftAccuracy < rightAccuracy
        }) else { return nil }

        if stats.count == 1 {
            let accuracy = Double(worst.value.correct) / Double(worst.value.total)
            return accuracy < 1 ? worst.key : nil
        }
        if worst.key == strongest { return nil }
        return worst.key
    }

    private static func topicScore(_ topic: GreTopicInsightView) -> Double {
        if let accuracy = topic.practiceAccuracy { return accuracy }
        if let memory = topic.memoryScore { return memory }
        return topic.studyLabel.isEmpty ? 0 : 0.5
    }

    private static func strongestDashboardTopic(
        recommendedTopics: [GreTopicInsightView],
        weakTopic: GreTopicInsightView?
    ) -> String? {
        var candidates = recommendedTopics
        if let weakTopic { candidates.append(weakTopic) }
        return candidates.max(by: { topicScore($0) < topicScore($1) })?.displayName
    }
}
