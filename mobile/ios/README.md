# GRE Atlas iOS Companion

SwiftUI shell for the GRE mobile companion. **All scoring, review scheduling, dashboard, readiness, and GRE Atlas sync logic lives in `rslib/`** and is accessed through the same protobuf RPCs as desktop.

## Repository layout

```
mobile/
  mobile_bridge/                 # C FFI → Backend (staticlib)
    include/anki_mobile.h        # Public C ABI
    src/lib.rs                   # FFI entry points
    src/gre_pages.rs             # GRE page JSON loaders
  ios/
    GREAtlasCompanion.xcodeproj # Xcode project (open this)
    GREAtlasCompanion/          # SwiftUI sources
    scripts/build-mobile-bridge.sh
    out/                         # Generated Rust build artifacts (gitignored)
```

## Demo

See [DEMO.md](DEMO.md) for a full walkthrough (Simulator, physical iPhone, and tab-by-tab demo script).

On first launch the app copies a bundled demo collection from `Resources/DemoBundle/` into Application Support, then opens it via `anki_mobile_open_collection()`. `PrepareDemoCollection` still runs as an idempotent fallback to seed or upgrade demo data through the shared backend.

## Swift source files

| File                                                 | Role                                          |
| ---------------------------------------------------- | --------------------------------------------- |
| `GREAtlasCompanion/GREAtlasCompanionApp.swift`       | `@main` app + tab shell                       |
| `GREAtlasCompanion/Engine/AnkiMobileEngine.swift`    | Loads GRE pages via FFI                       |
| `GREAtlasCompanion/Engine/MobileBridgeClient.swift`  | Swift wrapper + C FFI bindings                |
| `GREAtlasCompanion/Engine/GrePageModels.swift`       | JSON view models from `gre_pages.rs`          |
| `GREAtlasCompanion/Engine/ProtobufEncoding.swift`    | Hand-encoded `BackendInit` for backend create |
| `GREAtlasCompanion/Engine/ScoreFormat.swift`         | Score formatting helpers                      |
| `GREAtlasCompanion/Engine/PracticeSession.swift`     | Interactive practice session UI state         |
| `GREAtlasCompanion/Engine/StudySession.swift`        | Interactive GRE review session UI state       |
| `GREAtlasCompanion/Engine/GREAtlasSyncSession.swift` | GRE Atlas practice sync UI state              |
| `GREAtlasCompanion/Views/StudyCardWebView.swift`     | WKWebView card rendering                      |
| `GREAtlasCompanion/Views/GREViews.swift`             | Dashboard, Study, Practice, Progress tabs     |

## FFI bindings

Swift declares the C ABI with `@_silgen_name` in `MobileBridgeClient.swift`:

- `anki_mobile_backend_create` / `anki_mobile_backend_destroy`
- `anki_mobile_open_collection`
- `anki_mobile_gre_dashboard_json`
- `anki_mobile_gre_progress_json`
- `anki_mobile_gre_practice_bootstrap_json`
- `anki_mobile_gre_record_attempt_json`
- `anki_mobile_gre_practice_scores_json`
- `anki_mobile_gre_study_json`
- `anki_mobile_gre_study_review_json`
- `anki_mobile_gre_study_answer_json`
- `anki_mobile_prepare_demo_json`
- `anki_mobile_brainlift_sync_status_json`
- `anki_mobile_brainlift_sync_pull_json`
- `anki_mobile_brainlift_sync_push_json`
- `anki_mobile_bytes_free` / `anki_mobile_last_error`

The full C header (including generic `anki_mobile_backend_command`) is at `mobile/mobile_bridge/include/anki_mobile.h`.

## mobile_bridge interface

| C function                                               | Purpose                                                    |
| -------------------------------------------------------- | ---------------------------------------------------------- |
| `anki_mobile_buildhash()`                                | Build identifier string                                    |
| `anki_mobile_backend_create(init_msg, …)`                | Create `Backend` from protobuf `BackendInit`               |
| `anki_mobile_backend_destroy(backend)`                   | Tear down backend                                          |
| `anki_mobile_open_collection(…)`                         | Open `.anki2` + media paths                                |
| `anki_mobile_backend_command(service, method, input, …)` | Generic protobuf RPC (parity tests; not yet used in Swift) |
| `anki_mobile_gre_*_json(…)`                              | GRE page loaders → camelCase JSON                          |
| `anki_mobile_gre_study_answer_json(input, …)`            | `AnswerCard` RPC → next card / completion                  |
| `anki_mobile_gre_record_attempt_json(input, …)`          | `RecordAttempt` RPC → result JSON                          |
| `anki_mobile_prepare_demo_json(…)`                       | `PrepareDemoCollection` → demo deck/cards/attempts         |
| `anki_mobile_brainlift_sync_status_json(…)`              | `GetBrainLiftSyncStatus` → USN / pending count             |
| `anki_mobile_brainlift_sync_pull_json(input, …)`         | `PullBrainLiftChanges` → export attempts                   |
| `anki_mobile_brainlift_sync_push_json(input, …)`         | `PushBrainLiftChanges` → merge + conflicts                 |
| `anki_mobile_gre_practice_scores_json(…)`                | `GetScores` memory/performance strip after attempts        |
| `anki_mobile_bytes_free(ptr, len)`                       | Free Rust-allocated output buffers                         |
| `anki_mobile_last_error(out)`                            | Panic message after `ANKI_MOBILE_PANIC`                    |

Return codes: `ANKI_MOBILE_OK`, `ANKI_MOBILE_BACKEND_ERROR`, `ANKI_MOBILE_INVALID_INPUT`, `ANKI_MOBILE_PANIC`.

## Build the iOS app

### Prerequisites

- macOS with **Xcode** and iOS SDK installed (`xcode-select -p`)
- **Rust** toolchain (`rustup`)
- iOS Rust targets (the build script installs these automatically):
  - `aarch64-apple-ios` (device)
  - `aarch64-apple-ios-sim` (Apple Silicon simulator)
  - `x86_64-apple-ios` (Intel simulator)

### Option A — Xcode (recommended)

1. Open the project:

   ```bash
   open mobile/ios/GREAtlasCompanion.xcodeproj
   ```

2. Select the **GREAtlasCompanion** scheme and an iPhone simulator (or device).

3. Set **Signing & Capabilities → Team** to your Apple Developer team (required for device builds).

4. **Product → Build** (⌘B) or **Run** (⌘R).

The **Build mobile_bridge** run script phase invokes `scripts/build-mobile-bridge.sh`, which:

1. Detects the active SDK (`iphoneos` vs `iphonesimulator`) and architecture.
2. Runs `cargo build --release -p mobile_bridge --target …` from the repo root.
3. Copies `libmobile_bridge.a` to `mobile/ios/out/lib/libmobile_bridge.a`.

Xcode links that static library and `Security.framework` (required by Rust TLS).

### Option B — Command line

```bash
cd mobile/ios
xcodebuild \
  -project GREAtlasCompanion.xcodeproj \
  -scheme GREAtlasCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  build
```

### Verify Rust bridge (no Xcode required)

```bash
./ninja rslib:proto   # or ensure protos are generated
CARGO_TARGET_DIR=out/rust cargo test -p mobile_bridge
CARGO_TARGET_DIR=out/rust cargo test -p mobile_bridge ios_demo_bundle
```

Regenerate bundled demo files after seed changes:

```bash
mobile/ios/scripts/generate-bundled-demo-collection.sh
```

## Architecture

```
SwiftUI (mobile/ios)
    ↓ MobileBridgeClient.swift (C FFI)
    ↓ anki_mobile_gre_*_json page loaders
Backend::run_service_method  ← same entry point as pylib/rsbridge
    ↓
rslib/gre_atlas/* + rslib/scheduler/* + rslib/sync/*
```

| Tab       | FFI                                       | Desktop page RPC bundle     |
| --------- | ----------------------------------------- | --------------------------- |
| Dashboard | `anki_mobile_gre_dashboard_json`          | `home/+page.ts` / dashboard |
| Study     | `anki_mobile_gre_study_json`              | `review/+page.ts`           |
| Practice  | `anki_mobile_gre_practice_bootstrap_json` | `practice/+page.ts`         |
| Progress  | `anki_mobile_gre_progress_json`           | `progress/+page.ts`         |

## Offline operation

- Copy bundled demo collection on first launch (`DemoCollectionInstaller` → App Support `GRE Atlas/`), then open via `anki_mobile_open_collection`.
- GRE tabs call `anki_mobile_gre_*_json`, which run the shared RPC bundles and return camelCase JSON matching `GrePageModels.swift`.
- No network required for dashboard, progress, practice bootstrap, or study due counts.

## Related docs

- [gre-atlas-mobile.md](../../docs/gre-atlas-mobile.md)
- [gre-atlas-release.md](../../docs/gre-atlas-release.md)
