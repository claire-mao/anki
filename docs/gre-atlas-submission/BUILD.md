# GRE Atlas — build guide

**Audit date:** 2026-07-05\
**Commit:** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

Related: [../development.md](../development.md) (full upstream prerequisites) · [INSTALL.md](./INSTALL.md) (installer) · [tests.md](./tests.md) · [artifacts.md](./artifacts.md) · [SUBMISSION.md](./SUBMISSION.md)

> **Note:** On macOS, `build.md` and `BUILD.md` refer to the same file.

---

## 1. Dev build (day-to-day)

From a configured Anki dev environment ([../development.md](../development.md)):

```bash
just build          # pylib + Qt (./ninja pylib qt)
just run            # GRE shell at /home
just check          # format + build + all tests (requires CONTRIBUTORS entry)
```

GRE pages are served during `just run` at `http://127.0.0.1:40000/_anki/pages/` (collection open lands on `/home`).

TypeScript live reload (separate terminal): `just web-watch`

---

## 2. Wednesday build evidence (automated proof)

### `just build` — result

| Field     | Value                                                           |
| --------- | --------------------------------------------------------------- |
| Command   | `just build`                                                    |
| Exit code | **0**                                                           |
| Targets   | `pylib`, `qt` (SvelteKit GRE pages, reviewer, editor, congrats) |

Raw log: [logs/wednesday-build.log](./logs/wednesday-build.log)

### iOS simulator build

| Field            | Value                                                                                                                                                                                               |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Bridge script    | `mobile/ios/scripts/build-mobile-bridge.sh` (`PLATFORM_NAME=iphonesimulator ARCHS=arm64`)                                                                                                           |
| Xcode command    | `xcodebuild -project GREAtlasCompanion.xcodeproj -scheme GREAtlasCompanion -destination 'platform=iOS Simulator,name=iPhone 17,OS=26.5' -derivedDataPath /tmp/GREAtlasCompanion-wednesday-DD build` |
| Result           | **BUILD SUCCEEDED**                                                                                                                                                                                 |
| Simulator `.app` | `/tmp/GREAtlasCompanion-wednesday-DD/Build/Products/Debug-iphonesimulator/GREAtlasCompanion.app`                                                                                                    |

Raw logs: [logs/wednesday-mobile-bridge.log](./logs/wednesday-mobile-bridge.log), [logs/wednesday-xcodebuild-simulator.log](./logs/wednesday-xcodebuild-simulator.log)

**Note:** Use an isolated `-derivedDataPath` if another `xcodebuild` holds a lock on default DerivedData.

### Desktop installer build

| Artifact  | Path                                          |
| --------- | --------------------------------------------- |
| macOS DMG | `out/installer/dist/anki-26.05-mac-apple.dmg` |

Build command: `tools/build-installer` (same as `RELEASE=2 ./ninja installer`). Details: [artifacts.md](./artifacts.md), [INSTALL.md](./INSTALL.md).

### Abstention constant verification

Memory abstention gate **`MIN_STUDIED_CARDS = 50`** in `rslib/src/gre_atlas/abstention.rs`, `ts/routes/(gre)/empty-states.ts`, `prediction-readiness-presentation.ts`.

---

## 3. Reproduce this audit

```bash
cd /path/to/anki
git rev-parse HEAD | tee docs/gre-atlas-submission/logs/wednesday-commit.txt
just build 2>&1 | tee docs/gre-atlas-submission/logs/wednesday-build.log
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
xcodebuild -project GREAtlasCompanion.xcodeproj -scheme GREAtlasCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 17,OS=26.5' \
  -derivedDataPath /tmp/GREAtlasCompanion-wednesday-DD build \
  2>&1 | tee ../../docs/gre-atlas-submission/logs/wednesday-xcodebuild-simulator.log
```
