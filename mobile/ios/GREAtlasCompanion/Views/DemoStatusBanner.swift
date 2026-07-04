// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

/// Compact banner showing live collection state prepared by the Rust backend.
struct DemoStatusBanner: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        if let demo = engine.demo {
            VStack(alignment: .leading, spacing: 4) {
                Text("GRE Atlas demo collection")
                    .font(.caption.weight(.semibold))
                Text(demoSummary(demo))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, GreTheme.pagePadding)
            .padding(.vertical, 6)
            .background(.ultraThinMaterial)
        } else if engine.isLoading {
            ProgressView("Loading Rust backend…")
                .font(.caption)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 8)
                .background(.ultraThinMaterial)
        }
    }

    private func demoSummary(_ demo: GreDemoCollectionView) -> String {
        var parts: [String] = [
            "\(demo.dueTotal) cards due for review",
        ]
        if let practice = engine.practice {
            parts.append("\(practice.questionCount) practice questions")
        }
        if demo.practiceAttemptsAdded > 0 {
            parts.append("\(demo.practiceAttemptsAdded) sample attempts logged")
        }
        if demo.cardsAdded > 0 {
            parts.append("\(demo.cardsAdded) flashcards seeded")
        }
        parts.append("Settings → GRE Atlas sync")
        return parts.joined(separator: " · ")
    }
}
