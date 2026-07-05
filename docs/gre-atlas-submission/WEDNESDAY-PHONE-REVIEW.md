# GRE Atlas — Wednesday phone review recording instructions

Step-by-step guide to record a **simulator review session** for Wednesday proof. No fabricated data — the bundled demo collection shows real abstention until you review cards.

## Prerequisites

1. Repo at commit documented in [wednesday-release-proof.md](./wednesday-release-proof.md)
2. Xcode 16+ with iOS Simulator
3. Rust on PATH (`rustup`)

## One-time setup

```bash
cd /Users/clairemao/anki
./ninja rslib:proto
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
```

## Build and launch (Simulator)

1. Open the project:
   ```bash
   open mobile/ios/GREAtlasCompanion.xcodeproj
   ```
2. Scheme: **GREAtlasCompanion**
3. Destination: **iPhone 17** (or any available iPhone simulator)
4. **Product → Run** (⌘R)

   Or from terminal:
   ```bash
   cd mobile/ios
   xcodebuild -project GREAtlasCompanion.xcodeproj -scheme GREAtlasCompanion \
     -destination 'platform=iOS Simulator,name=iPhone 17' build
   xcrun simctl boot "iPhone 17" 2>/dev/null || true
   xcrun simctl install booted ~/Library/Developer/Xcode/DerivedData/GREAtlasCompanion-*/Build/Products/Debug-iphonesimulator/GREAtlasCompanion.app
   xcrun simctl launch booted org.ankitects.greatlas.companion
   ```

## Recording setup (macOS)

1. **Cmd+Shift+5** → Record Selected Portion (or QuickTime → New Screen Recording)
2. Frame the **Simulator window** only
3. Hide unrelated windows and personal Anki profiles

Suggested filename (save under `docs/gre-atlas-submission/recordings/` — gitignored):

`wednesday-phone-review.mov`

## Demo script (~3 minutes)

| Step | Action            | What to show                                                                                      |
| ---- | ----------------- | ------------------------------------------------------------------------------------------------- |
| 1    | App launch        | Splash → **Dashboard** tab loads (coverage, readiness bands from Rust)                            |
| 2    | Confirm demo deck | Banner or deck name **GRE Atlas**; Progress tab shows topic tree                                  |
| 3    | **Study** tab     | Tap **Start review**                                                                              |
| 4    | Review loop       | Question front → **Show answer** → tap **Again**, **Hard**, **Good**, or **Easy**                 |
| 5    | Second card       | Repeat Show answer → grade (proves loop continues)                                                |
| 6    | Exit review       | Return to Study tab; due count updates                                                            |
| 7    | Progress tab      | Topic mastery / memory metrics reflect reviews (may still abstain on readiness — that is correct) |

## Acceptance criteria

- [ ] Simulator shows **GRE Atlas** deck (from `DemoBundle/collection.anki2`)
- [ ] Review uses FSRS grade buttons (not practice MCQs)
- [ ] At least **2 cards** graded on camera
- [ ] No off-camera data seeding
- [ ] Recording saved as `recordings/wednesday-phone-review.mov`

## Reset demo (optional)

Delete app from simulator, or remove Application Support:

```
~/Library/Developer/CoreSimulator/Devices/<UDID>/data/Containers/Data/Application/*/Library/Application Support/GRE Atlas/
```

Re-launch to copy `DemoBundle` again.

## Automated verification (no recording)

These tests prove the same review path without Xcode UI:

```bash
cargo test -p mobile_bridge gre_study_review_matches_between_mobile_ffi_and_direct_backend
cargo test -p mobile_bridge grade_buttons_follow_scheduler_labels
cargo test -p anki gre_atlas::demo::
```

## Troubleshooting

| Issue                        | Fix                                                             |
| ---------------------------- | --------------------------------------------------------------- |
| `libmobile_bridge.a` missing | Run `./scripts/build-mobile-bridge.sh`                          |
| Wrong simulator name         | Use `xcodebuild -showdestinations` and pick an iPhone simulator |
| Study shows "Deck not found" | Reset app; ensure `PrepareDemoCollection` ran                   |
| Build DB locked              | Wait for other Xcode builds to finish, then rebuild             |
