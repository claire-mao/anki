# BrainLift iOS Companion

SwiftUI shell for the GRE mobile companion. **All scoring, review scheduling, dashboard, readiness, and BrainLift sync logic lives in `rslib/`** and is accessed through the same protobuf RPCs as desktop.

## Architecture

```
SwiftUI (mobile/ios)
    ↓ MobileBridgeClient.swift (C FFI)
    ↓ anki_mobile_gre_*_json page loaders
Backend::run_service_method  ← same entry point as pylib/rsbridge
    ↓
rslib/brainlift/* + rslib/scheduler/* + rslib/sync/*
```

| Tab | FFI | Desktop page RPC bundle |
| --- | --- | --- |
| Dashboard | `anki_mobile_gre_dashboard_json` | `home/+page.ts` / dashboard |
| Study | `anki_mobile_gre_study_json` | `review/+page.ts` |
| Practice | `anki_mobile_gre_practice_bootstrap_json` | `practice/+page.ts` |
| Progress | `anki_mobile_gre_progress_json` | `progress/+page.ts` |

Page loaders live in `mobile/mobile_bridge/src/gre_pages.rs` and aggregate the same RPC calls as the Svelte `+page.ts` loaders.

## Offline operation

- Open collection locally via `anki_mobile_open_collection` (App Support `BrainLift/collection.anki2`).
- GRE tabs call `anki_mobile_gre_*_json`, which run the shared RPC bundles and return camelCase JSON matching `GrePageModels.swift`.
- No network required for review, practice bootstrap, dashboard, or progress.
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

Link `libmobile_bridge.a` in Xcode and add `mobile/mobile_bridge/include` to the header search path. Add all Swift files under `mobile/ios/BrainLiftCompanion/` to the app target.

## Verify shared backend logic

```bash
just rslib-proto  # or ./ninja rslib:proto
CARGO_TARGET_DIR=out/rust cargo test -p mobile_bridge
CARGO_TARGET_DIR=out/rust cargo test brainlift::sync::
```

Parity tests assert mobile FFI and direct `Backend::run_service_method` return identical protobuf bytes for each RPC, and identical JSON for Dashboard, Progress, Practice bootstrap, and Study page bundles.

See also [brainlift-mobile.md](../../docs/brainlift-mobile.md) and [brainlift-release.md](../../docs/brainlift-release.md).
