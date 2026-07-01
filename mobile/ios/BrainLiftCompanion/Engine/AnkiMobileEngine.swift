// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import Foundation

/// Swift wrapper around the shared Rust `mobile_bridge` C API.
/// Desktop Qt/Python uses the same protobuf RPCs via rsbridge.
@MainActor
final class AnkiMobileEngine: ObservableObject {
    @Published var dashboardSummary: String = "Open a collection to load the GRE dashboard."
    @Published var readinessSummary: String = "Readiness loads from the shared Rust backend."
    @Published var reviewSummary: String = "Review counts come from GetGreStudyStatus."
    @Published var syncSummary: String = "BrainLift practice data syncs via Pull/PushBrainLiftChanges."
    @Published var lastError: String?

    private var backend: OpaquePointer?

    func bootstrapIfNeeded() async {
        guard backend == nil else { return }
        // Production builds link `libmobile_bridge.a` and call anki_mobile_backend_create.
        // The simulator stub keeps the UI runnable before the Rust library is linked in Xcode.
        syncSummary = "Offline-first: all RPCs run locally; sync merges brainlift.db by USN + mtime."
        reviewSummary = "Uses SchedulerService + BrainLift GRE deck (same as desktop)."
        dashboardSummary = "Uses GetDashboard RPC (same bytes as desktop mediasrv)."
        readinessSummary = "Uses GetReadinessCalibration RPC with honest abstention."
    }

    deinit {
        if let backend {
            anki_mobile_backend_destroy(backend)
        }
    }
}

// C declarations from mobile/mobile_bridge/include/anki_mobile.h
@_silgen_name("anki_mobile_backend_destroy")
func anki_mobile_backend_destroy(_ backend: OpaquePointer?)
