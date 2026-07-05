// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// Presentation helpers for practice explanations — kept in sync with
/// `ts/routes/(gre)/practice/practice-presentation.ts`.
enum PracticePresentation {
    static let provenanceOfflineTemplate = "offline_template"
    static let offlineTemplateNote = "Generated using offline templates."

    static func isOfflineTemplateProvenance(_ provenance: String) -> Bool {
        provenance.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            == provenanceOfflineTemplate
    }

    static func resolveExplanationProvenanceNote(
        provenance: String,
        provenanceNote: String
    ) -> String? {
        guard isOfflineTemplateProvenance(provenance) else { return nil }
        let note = provenanceNote.trimmingCharacters(in: .whitespacesAndNewlines)
        return note.isEmpty ? offlineTemplateNote : note
    }

    static func formatExplanationCitation(
        sourceName: String,
        sourceSection: String
    ) -> String? {
        let name = sourceName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return nil }
        let section = sourceSection.trimmingCharacters(in: .whitespacesAndNewlines)
        return section.isEmpty ? name : "\(name) — \(section)"
    }

    static func orderExplanationChoices(
        _ choices: [GreAnswerChoiceExplanationView]
    ) -> [GreAnswerChoiceExplanationView] {
        choices
            .filter { !$0.reasoning.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
            .sorted { lhs, rhs in
                if lhs.isCorrect != rhs.isCorrect {
                    return lhs.isCorrect && !rhs.isCorrect
                }
                return false
            }
    }
}
