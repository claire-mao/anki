// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// Student-facing mission copy aligned with `ts/routes/(gre)/daily-mission.ts`.
enum GreMissionCopy {
    static func intro(taskCount: UInt) -> String {
        if taskCount == 1 {
            return "One focused action to move your score today."
        }
        return "\(taskCount) focused actions to move your score today."
    }

    static func title(for task: GreDailyTaskView) -> String {
        switch task.id {
        case "review_cards":
            return "Review flashcards"
        case "practice_questions":
            return "Practice questions"
        default:
            var title = task.title
            if title.hasPrefix("GRE ") {
                title.removeFirst(4)
            }
            return title
        }
    }

    static func description(for task: GreDailyTaskView) -> String {
        switch task.id {
        case "review_cards":
            return task.targetCount > 0
                ? "Work through due cards and lock in retention."
                : "You're caught up — keep momentum with focus topics."
        case "practice_questions":
            return "Answer exam-style questions to sharpen accuracy."
        default:
            if task.title.hasPrefix("Cover") {
                return "Answer practice questions to close a catalog gap."
            }
            if task.title.hasPrefix("Strengthen") {
                return "Rebuild mastery where memory is slipping."
            }
            return task.detail.isEmpty ? "Target this topic with focused practice." : task.detail
        }
    }

    static func progressLabel(for task: GreDailyTaskView, dueTotal: UInt) -> String {
        if task.id == "review_cards", task.targetCount == 0 {
            return "All caught up"
        }
        if task.targetCount == 0 {
            return "Ready to start"
        }
        let unit = task.id == "practice_questions"
            || task.title.hasPrefix("Practice")
            || task.title.hasPrefix("Cover")
            ? "questions"
            : "cards"
        return "Target: \(task.targetCount) \(unit)"
    }

    static func progressDetail(for task: GreDailyTaskView, dueTotal: UInt) -> String? {
        if task.id == "review_cards", task.targetCount == 0 {
            return "Nothing due right now."
        }
        if task.id == "review_cards", dueTotal > 0 {
            return "\(dueTotal) due now"
        }
        return nil
    }

    static func actionLabel(for task: GreDailyTaskView) -> String {
        switch task.id {
        case "review_cards":
            return task.targetCount > 0 ? "Review flashcards" : "View study plan"
        case "practice_questions":
            return "Practice questions"
        default:
            if task.title.hasPrefix("Cover") || task.title.hasPrefix("Strengthen") {
                return "Practice topic"
            }
            return "Practice questions"
        }
    }

    static func practiceTopicTitle(for topicId: String) -> String {
        let normalized = topicId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return "Practice" }

        let labels: [String: String] = [
            "gre::quant::data_interpretation": "Data interpretation",
            "gre::verbal::reading::function": "Function of a sentence",
            "gre::awa::argument": "Analyze an Argument",
            "gre::awa::issue": "Analyze an Issue",
        ]
        if let label = labels[normalized] {
            return label
        }
        let leaf = normalized.split(separator: "::").last.map(String.init) ?? normalized
        return leaf
            .replacingOccurrences(of: "_", with: " ")
            .split(separator: " ")
            .map { $0.prefix(1).uppercased() + $0.dropFirst() }
            .joined(separator: " ")
    }

    static func questionTypeLabel(for format: String) -> String {
        let normalized = format.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        let labels: [String: String] = [
            "mcq": "Multiple Choice",
            "multiple_choice": "Multiple Choice",
            "text_completion": "Text Completion",
            "sentence_equivalence": "Sentence Equivalence",
            "reading_comprehension": "Reading Comprehension",
            "data_interpretation": "Data Interpretation",
            "essay_prompt": "Essay Prompt",
        ]
        if let label = labels[normalized] {
            return label
        }
        guard !normalized.isEmpty else { return "Question" }
        return normalized
            .replacingOccurrences(of: "_", with: " ")
            .split(separator: " ")
            .map { $0.prefix(1).uppercased() + $0.dropFirst() }
            .joined(separator: " ")
    }
}
