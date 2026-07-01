// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

@main
struct BrainLiftCompanionApp: App {
    @StateObject private var engine = AnkiMobileEngine()

    var body: some Scene {
        WindowGroup {
            RootTabView()
                .environmentObject(engine)
        }
    }
}

struct RootTabView: View {
    @EnvironmentObject private var engine: AnkiMobileEngine

    var body: some View {
        TabView {
            DashboardView()
                .tabItem { Label("Dashboard", systemImage: "gauge") }
            ReadinessView()
                .tabItem { Label("Readiness", systemImage: "chart.line.uptrend.xyaxis") }
            ReviewView()
                .tabItem { Label("Review", systemImage: "rectangle.stack") }
            SyncView()
                .tabItem { Label("Sync", systemImage: "arrow.triangle.2.circlepath") }
        }
        .task {
            await engine.bootstrapIfNeeded()
        }
    }
}
