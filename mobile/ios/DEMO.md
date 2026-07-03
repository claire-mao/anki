# GRE Atlas iOS Demo Guide

Run the GRE Atlas companion on **iOS Simulator** or a **physical iPhone**. All tab data comes from the shared Rust backend (`rslib/`) through `mobile_bridge` — no Swift mock data.

## Prerequisites

1. **macOS** with Xcode 16+ and iOS SDK
2. **Rust** (`rustup`) on your PATH
3. **Protos generated** (once per clone):

   ```bash
   cd /path/to/anki
   ./ninja rslib:proto
   ```

4. **Physical device only:** open `GREAtlasCompanion.xcodeproj` → target **GREAtlasCompanion** → **Signing & Capabilities** → set your **Team**.

## Build and run

### Simulator (recommended for demos)

```bash
open mobile/ios/GREAtlasCompanion.xcodeproj
```

1. Select scheme **GREAtlasCompanion**
2. Choose an **iPhone simulator** (e.g. iPhone 16)
3. **Product → Run** (⌘R)

Or from the command line:

```bash
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
xcodebuild \
  -project GREAtlasCompanion.xcodeproj \
  -scheme GREAtlasCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  build
```

### Physical iPhone

1. Connect your iPhone and trust the Mac
2. Set **Signing & Capabilities → Team** in Xcode
3. Select your device as the run destination
4. **Product → Run** (⌘R)

For device builds, Xcode sets `PLATFORM_NAME=iphoneos` and builds `aarch64-apple-ios` automatically via the **Build mobile_bridge** run script.

## What happens on first launch

1. **Copy bundled demo collection** — if `Application Support/GRE Atlas/collection.anki2` is missing, the app copies `DemoBundle/collection.anki2`, `greatlas.db`, and `collection.media/` from the app bundle (see `DemoCollectionInstaller.swift`).
2. **Open collection** — `anki_mobile_open_collection()` opens the copied files.
3. **Prepare demo (idempotent)** — `PrepareDemoCollection` in Rust ensures deck/cards/attempts exist (no-op when the bundle is already seeded):

| Action                                  | Rust source                                              |
| --------------------------------------- | -------------------------------------------------------- |
| Create **GRE Atlas** deck               | `rslib/src/gre_atlas/demo.rs`                            |
| Seed **8 flashcards** with `gre::` tags | `rslib/src/gre_atlas/flashcards/seed_gre.json`           |
| Seed **9 practice questions**           | `rslib/src/gre_atlas/questions/seed_gre.json` (existing) |
| Log **4 sample practice attempts**      | `greatlas.db` via `RecordAttempt` storage                |
| Load all GRE pages                      | Same RPC bundles as desktop                              |

Subsequent launches skip the copy step if the collection file already exists. `PrepareDemoCollection` remains idempotent.

### Regenerate the bundled collection

After changing demo seed data or `PrepareDemoCollection`:

```bash
mobile/ios/scripts/generate-bundled-demo-collection.sh
```

Verification: `cargo test -p mobile_bridge ios_demo_bundle`

## Demo walkthrough (~5 minutes)

| Step | Tab                           | What to show                                                                                    |
| ---- | ----------------------------- | ----------------------------------------------------------------------------------------------- |
| 1    | **Dashboard**                 | Coverage, daily plan, due counts, readiness bands (real abstention or partial scores from Rust) |
| 2    | **Study** → **Start review**  | FSRS-scheduled flashcard review; **Show answer** → Again/Hard/Good/Easy                         |
| 3    | **Practice**                  | Answer GRE MCQs; scores update via `RecordAttempt`                                              |
| 4    | **Progress**                  | Topic mastery tree, performance attempt count, estimated GRE                                    |
| 5    | **Settings → GRE Atlas sync** | Sync status (USN), **Pull** exports attempts, **Push** merges with conflict reporting           |

**Review** is the interactive flashcard flow inside the **Study** tab (same scheduler as desktop Anki).

**Sync** is GRE Atlas practice-attempt sync (`PullBrainLiftChanges` / `PushBrainLiftChanges`), not AnkiWeb collection sync.

## Architecture

```
SwiftUI tabs
  → AnkiMobileEngine / GreBridgeLoader
  → mobile_bridge C FFI (anki_mobile_*_json)
  → Backend::run_service_method
  → rslib/ (BrainLiftService, SchedulerService, …)
```

## Reset demo collection

Delete the app from the simulator/device, or remove:

```
~/Library/Developer/CoreSimulator/.../Application Support/GRE Atlas/
```

Re-launch to copy the bundled demo again (or re-run `PrepareDemoCollection` if you kept the directory but deleted only `greatlas.db`).

## Verify Rust bridge (no Xcode)

```bash
cd /path/to/anki
./ninja rslib:proto
cargo test -p mobile_bridge
cargo test -p anki gre_atlas::demo::
```

## Troubleshooting

| Issue                        | Fix                                                                    |
| ---------------------------- | ---------------------------------------------------------------------- |
| `libmobile_bridge.a` missing | Run `./scripts/build-mobile-bridge.sh` or build once in Xcode          |
| Study shows "Deck not found" | Reset app data; ensure `PrepareDemoCollection` ran (check demo banner) |
| Device build signing error   | Set **Team** in Signing & Capabilities                                 |
| Proto compile errors         | Run `./ninja rslib:proto`                                              |
