# BrainLift iOS Companion

SwiftUI shell for the GRE mobile companion. **All scoring, review scheduling, dashboard, readiness, and BrainLift sync logic lives in `rslib/`** and is accessed through the same protobuf RPCs as desktop.

## Architecture

```
SwiftUI (mobile/ios)
    ↓ C FFI (mobile/mobile_bridge/include/anki_mobile.h)
Backend::run_service_method  ← same entry point as pylib/rsbridge
    ↓
rslib/brainlift/* + rslib/scheduler/* + rslib/sync/*
```

| Feature | RPC | Rust module |
| ------- | --- | ----------- |
| Review / FSRS | `SchedulerService.*` | `rslib/scheduler/` |
| Collection sync | `BackendSyncService.*` | `rslib/sync/` |
| GRE dashboard | `BackendBrainLiftService.get_dashboard` | `rslib/brainlift/dashboard.rs` |
| Readiness | `BackendBrainLiftService.get_readiness_calibration` | `rslib/brainlift/calibration.rs` |
| Practice sync | `BackendBrainLiftService.pull/push_brain_lift_changes` | `rslib/brainlift/sync.rs` |

## Offline operation

- Open collection locally via `BackendCollectionService.open_collection`.
- All BrainLift RPCs read/write local `brainlift.db` beside the profile.
- No network required for review, practice, dashboard, or readiness.
- Sync is explicit: exchange protobuf chunks when online.

## Conflict handling

- **Collection:** existing Anki sync (`mtime` / USN rules in `rslib/src/sync/collection/`).
- **BrainLift practice:** `PushBrainLiftChanges` keeps the row with newer `mtime_secs`; older remote rows return `BrainLiftSyncConflict`.

## Build Rust bridge for iOS

From repo root (requires Rust iOS targets):

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cd mobile/mobile_bridge
cargo build --release --target aarch64-apple-ios
```

Link `libmobile_bridge.a` in Xcode and add `mobile/mobile_bridge/include` to the header search path.

## Verify shared backend logic

```bash
./ninja rslib:proto
CARGO_TARGET_DIR=out/rust cargo test -p mobile_bridge
CARGO_TARGET_DIR=out/rust cargo test brainlift::sync::
./ninja check:pytest:pylib
```

Parity tests assert mobile FFI and direct `Backend::run_service_method` return identical protobuf bytes for `get_scores` and `get_dashboard`.

See also [brainlift-release.md](../../docs/brainlift-release.md).
