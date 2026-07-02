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
                .tabItem { Label("Dashboard", systemImage: "gauge.with.dots.needle.67percent") }
            StudyView()
                .tabItem { Label("Study", systemImage: "rectangle.stack") }
            PracticeView()
                .tabItem { Label("Practice", systemImage: "checkmark.circle") }
            GreProgressScreen()
                .tabItem { Label("Progress", systemImage: "chart.line.uptrend.xyaxis") }
        }
        .tint(.accentColor)
        .task {
            await engine.bootstrapIfNeeded()
        }
    }
}
