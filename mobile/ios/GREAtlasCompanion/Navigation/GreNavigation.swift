// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

/// Keep in sync with `ts/routes/(gre)/gre-navigation.ts`.
enum GreNavTab: String, CaseIterable, Identifiable {
    case dashboard
    case study
    case practice
    case progress
    case settings

    var id: String { rawValue }

    var label: String {
        switch self {
        case .dashboard: "Dashboard"
        case .study: "Study"
        case .practice: "Practice"
        case .progress: "Progress"
        case .settings: "Settings"
        }
    }

    /// SF Symbol chosen to mirror desktop `GreIcon` shapes.
    var systemImage: String {
        switch self {
        case .dashboard: "square.grid.2x2"
        case .study: "book.closed"
        case .practice: "checkmark.rectangle"
        case .progress: "chart.bar"
        case .settings: "gearshape"
        }
    }
}
