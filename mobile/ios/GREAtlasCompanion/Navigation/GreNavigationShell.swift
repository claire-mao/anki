// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

/// Root GRE shell: five primary sections matching desktop `greNavItems`.
struct GreNavigationShell: View {
    @EnvironmentObject private var engine: AnkiMobileEngine
    @State private var selectedTab: GreNavTab = .dashboard

    var body: some View {
        TabView(selection: $selectedTab) {
            ForEach(GreNavTab.allCases) { tab in
                tabContent(for: tab)
                    .tabItem {
                        Label(tab.label, systemImage: tab.systemImage)
                    }
                    .tag(tab)
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            DemoStatusBanner()
        }
        .tint(.accentColor)
        .task {
            await engine.bootstrapIfNeeded()
        }
    }

    @ViewBuilder
    private func tabContent(for tab: GreNavTab) -> some View {
        switch tab {
        case .dashboard:
            DashboardView()
        case .study:
            StudyView()
        case .practice:
            PracticeView()
        case .progress:
            GreProgressScreen()
        case .settings:
            SettingsView()
        }
    }
}
