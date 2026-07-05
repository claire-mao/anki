# GRE Atlas — Wednesday build artifacts

**Audit date:** 2026-07-05\
**Commit:** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

Related: [build.md](./build.md), [../gre-atlas-release.md](../gre-atlas-release.md) (installer process).

---

## Desktop installer

| Artifact            | Path                                          | Size                       | Modified (local) |
| ------------------- | --------------------------------------------- | -------------------------- | ---------------- |
| macOS DMG           | `out/installer/dist/anki-26.05-mac-apple.dmg` | 214 MB (224,436,927 bytes) | 2026-07-05 14:22 |
| Unpacked app bundle | `out/installer/build/anki/macos/app/Anki.app` | ~659 MB                    | 2026-07-05 14:21 |
| Main binary         | `…/Anki.app/Contents/MacOS/Anki`              | 75 KB                      | 2026-07-05 14:22 |

**Build command:** `tools/build-installer` (equivalent: `RELEASE=2 ./ninja installer`)

**Build logs:** `out/installer/logs/briefcase.2026_07_05-14_18_27.build.log`, `briefcase.2026_07_05-14_19_28.package.log`

### Verify installer locally

```bash
open out/installer/dist/anki-26.05-mac-apple.dmg
# Drag Anki.app to Applications, launch, open a profile → GRE shell at /home
```

---

## iOS companion

| Artifact              | Path                                                                                             | Size                     | Modified (local) |
| --------------------- | ------------------------------------------------------------------------------------------------ | ------------------------ | ---------------- |
| Static bridge library | `mobile/ios/out/lib/libmobile_bridge.a`                                                          | 86 MB (90,581,904 bytes) | 2026-07-05 14:22 |
| Simulator app bundle  | `/tmp/GREAtlasCompanion-wednesday-DD/Build/Products/Debug-iphonesimulator/GREAtlasCompanion.app` | ~50 MB                   | 2026-07-05 14:20 |
| Simulator binary      | `…/GREAtlasCompanion.app/GREAtlasCompanion`                                                      | 57 KB                    | 2026-07-05 14:20 |
| Bundled demo DB       | `mobile/ios/GREAtlasCompanion/Resources/DemoBundle/greatlas.db`                                  | (bundled)                | in repo          |

**Build commands:**

```bash
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
xcodebuild -project GREAtlasCompanion.xcodeproj -scheme GREAtlasCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 17,OS=26.5' \
  -derivedDataPath /tmp/GREAtlasCompanion-wednesday-DD build
```

**Install on simulator:** Xcode → Product → Run (⌘R), or:

```bash
xcrun simctl install booted /tmp/GREAtlasCompanion-wednesday-DD/Build/Products/Debug-iphonesimulator/GREAtlasCompanion.app
```

See [../../mobile/ios/DEMO.md](../../mobile/ios/DEMO.md).

---

## Dev build outputs (not shipped)

| Artifact            | Path                                     | Notes                                                             |
| ------------------- | ---------------------------------------- | ----------------------------------------------------------------- |
| Python/Rust bridge  | `out/pylib/anki/_rsbridge.so`            | Built by `just build`                                             |
| SvelteKit GRE pages | `out/qt/_aqt/data/web/sveltekit/`        | Served at `http://127.0.0.1:40000/_anki/pages/` during `just run` |
| Sidecar schema      | `rslib/src/gre_atlas/storage/schema.sql` | Runtime: `greatlas.db` beside profile                             |

---

## Evaluation artifacts (read-only)

Pre-generated under [results/](./results/):

| File                                | Purpose                              |
| ----------------------------------- | ------------------------------------ |
| `gre-atlas-eval.json` / `.md`       | Full eval report                     |
| `performance-eval.md`               | Held-out performance metrics         |
| `gre-atlas-benchmark.{json,md,csv}` | API latency benchmark                |
| `gre-atlas-ai-eval.{json,md}`       | AI gold-set eval (release gate PASS) |

Regenerate: `just eval-gre-atlas /path/to/collection.anki2`, `just bench-gre-atlas`, `just eval-gre-atlas-ai`.

---

## UI evidence (manual — not in git)

| Type              | Location                       | Status                                                                                     |
| ----------------- | ------------------------------ | ------------------------------------------------------------------------------------------ |
| Screenshots       | [screenshots/](./screenshots/) | `08-eval-report.png`, `10-benchmark-output.png` ✅; `01`–`07` require `just run` capture ⚠️ |
| Screen recordings | [recordings/](./recordings/)   | Not committed (see [RECORDINGS.md](./RECORDINGS.md))                                       |
