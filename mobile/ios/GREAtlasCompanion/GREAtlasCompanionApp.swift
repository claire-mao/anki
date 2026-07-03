// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import SwiftUI

@main
struct GREAtlasCompanionApp: App {
    @StateObject private var engine = AnkiMobileEngine()

    var body: some Scene {
        WindowGroup {
            GreNavigationShell()
                .environmentObject(engine)
        }
    }
}
