// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

struct DashboardView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text("GRE Dashboard")
                        .font(.title.bold())
                    Text(engine.dashboardSummary)
                        .foregroundStyle(.secondary)
                    Text("Memory, performance, and readiness are computed in rslib/brainlift — not reimplemented in Swift.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                .padding()
            }
            .navigationTitle("Dashboard")
        }
    }
}

struct ReadinessView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Readiness & calibration")
                        .font(.title.bold())
                    Text(engine.readinessSummary)
                        .foregroundStyle(.secondary)
                    Text("When evidence is insufficient, the backend abstains and returns structured missing requirements — identical to desktop.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                .padding()
            }
            .navigationTitle("Readiness")
        }
    }
}

struct ReviewView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 16) {
                Text("Memory review")
                    .font(.title.bold())
                Text(engine.reviewSummary)
                    .foregroundStyle(.secondary)
                Text("Card scheduling uses SchedulerService in rslib (FSRS). Practice attempts never write revlog.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                Spacer()
            }
            .padding()
            .navigationTitle("Review")
        }
    }
}

struct SyncView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 16) {
                Text("Sync")
                    .font(.title.bold())
                Text(engine.syncSummary)
                    .foregroundStyle(.secondary)
                Text("Collection: SyncService (existing Anki sync). BrainLift practice: PullBrainLiftChanges / PushBrainLiftChanges with mtime conflict handling.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                if let error = engine.lastError {
                    Text(error)
                        .foregroundStyle(.red)
                        .font(.footnote)
                }
                Spacer()
            }
            .padding()
            .navigationTitle("Sync")
        }
    }
}
