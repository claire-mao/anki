# BrainLift Mobile Companion

The GRE mobile companion uses the **same Rust engine** as desktop Anki. Swift UI is a thin client; scoring, abstention, calibration, scheduling, and sync rules are not duplicated in mobile code.

## Shared backend guarantee

| Client | Bridge | RPC transport |
| ------ | ------ | ------------- |
| Desktop Qt | `pylib/rsbridge` → Python `_backend` | mediasrv HTTP + protobuf |
| Mobile iOS | `mobile/mobile_bridge` C FFI | in-process protobuf bytes |
| (future Android) | same `mobile_bridge` | in-process protobuf bytes |

Both call `Backend::run_service_method(service, method, input_bytes)` with **Backend\*** service indices from `anki_proto_gen`.

Verification:

- `mobile/mobile_bridge` tests compare FFI output to direct `Backend` calls for `get_scores` and `get_dashboard`.
- `rslib/src/brainlift/sync.rs` tests cover BrainLift pull/push and mtime conflicts.

## Features

### Review

Uses existing `SchedulerService` RPCs (same FSRS path as desktop reviewer). GRE deck selection uses `GetGreStudyStatus` / `BrainLift GRE` deck name constant in Rust.

### Collection synchronization

Uses existing `BackendSyncService` (`SyncCollection`, `FullUploadOrDownload`, etc.) — same conflict rules as desktop (`rslib/src/sync/collection/chunks.rs`, `changes.rs`).

### BrainLift practice synchronization

New RPCs on `BrainLiftService`:

- `GetBrainLiftSyncStatus` — current USN, pending upload count
- `PullBrainLiftChanges(after_usn)` — incremental attempt export
- `PushBrainLiftChanges(attempts)` — merge with **newer mtime wins**; stale rows become `BrainLiftSyncConflict`

Practice rows receive monotonic `usn` on each local change in `brainlift.db`.

### Dashboard & readiness (offline)

- `GetDashboard` — memory, performance, readiness, coverage, weak topics
- `GetReadinessCalibration` — Brier score, calibration curve, abstention requirements

Identical protobuf messages as desktop GRE Svelte pages (`ts/routes/(gre)/`).

## Repository layout

```
mobile/
  mobile_bridge/          # C FFI → Backend (link into iOS/Android)
  ios/BrainLiftCompanion/ # SwiftUI shell
docs/brainlift-mobile.md  # this file
```

## Build & test

```bash
./ninja rslib:proto pylib:anki:proto ts:generated:proto
CARGO_TARGET_DIR=out/rust cargo test -p mobile_bridge
CARGO_TARGET_DIR=out/rust cargo test brainlift::sync::
./ninja check:pytest:pylib
```

iOS app: see [mobile/ios/README.md](../mobile/ios/README.md).

## Related docs

- [brainlift-release.md](./brainlift-release.md) — desktop release
- [brainlift-architecture.md](./brainlift-architecture.md) — codebase map
- [brainlift-product-architecture.md](./brainlift-product-architecture.md) — product design
