# GRE Atlas — installer & clean-machine verification

Use this guide to build the desktop installer on a release machine and smoke-test a **clean install** (no prior dev checkout on the target Mac). For day-to-day development builds, see [BUILD.md](./BUILD.md) § Dev build and [../development.md](../development.md).

Related: [../gre-atlas-release.md](../gre-atlas-release.md) · [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md)

---

## Prerequisites (build machine)

Same as upstream Anki — see [../development.md](../development.md#building-from-source):

- macOS with Xcode command-line tools (macOS builds)
- Rust via `rustup`, `just`, N2/Ninja, Node/Yarn, Python 3.9+
- Full dependency fetch on first build (`just check` or `./ninja pylib qt`)

Public multi-platform CI releases use `just release::build` (GitHub Actions). **Local submission builds** use the same Briefcase pipeline as upstream:

```bash
tools/build-installer
# equivalent: RELEASE=2 ./ninja installer
```

There is no separate `just build-installer` recipe; the authoritative local command is `tools/build-installer`.

---

## Build the installer

From a clean or release branch checkout:

```bash
just check                    # recommended gate before packaging
tools/build-installer         # ~7 min on Apple Silicon after pylib/qt built
```

### Output artifacts

| Path                                          | Platform | Notes                         |
| --------------------------------------------- | -------- | ----------------------------- |
| `out/installer/dist/*.dmg`                    | macOS    | Redistributable disk image    |
| `out/installer/dist/*.msi`                    | Windows  | MSI installer                 |
| `out/installer/dist/*.tar.*`                  | Linux    | Tarball                       |
| `out/installer/build/anki/macos/app/Anki.app` | macOS    | Unpacked app bundle (pre-DMG) |
| `out/installer/logs/briefcase.*.log`          | all      | Briefcase build/package logs  |

Version comes from `.version` at repo root (e.g. `26.05` → `anki-26.05-mac-apple.dmg`).

Build verification log (2026-07-05): [logs/wednesday-installer-build.log](./logs/wednesday-installer-build.log).

---

## Clean-machine install (macOS)

Perform these steps on a Mac **without** the Anki source tree — e.g. a fresh user account, VM, or second machine.

### 1. Copy the installer

Transfer the DMG from the build machine:

```bash
# on build machine
ls -lh out/installer/dist/
# e.g. out/installer/dist/anki-26.05-mac-apple.dmg  (~214 MB)
```

Copy via AirDrop, shared drive, or USB. Do **not** commit `out/installer/` to git.

### 2. Install

1. Double-click the `.dmg`.
2. Drag **Anki** to **Applications**.
3. First launch: if macOS blocks the app (unsigned local build), open **System Settings → Privacy & Security** and allow Anki, or right-click → **Open**.

### 3. Smoke test (GRE Atlas)

| Step                          | Expected result                                                         |
| ----------------------------- | ----------------------------------------------------------------------- |
| Launch Anki                   | App opens; profile picker or new profile                                |
| Create / open profile         | Main window enters **GRE shell** at `/home`                             |
| Header nav                    | Home, Study, Practice, Progress, Settings visible                       |
| **Practice** → answer one MCQ | Feedback + explanation; performance count increases                     |
| **Progress** / **Readiness**  | Abstention checklist on fresh profile (FSRS, cards, coverage, attempts) |
| **Study** → Review            | Standard Anki reviewer starts (FSRS unchanged)                          |
| Sidecar                       | `{profile}/greatlas.db` created beside `collection.anki2`               |

Optional: finish a review session and confirm congrats links open GRE Practice / Dashboard modal.

### 4. Uninstall (clean retest)

```bash
rm -rf "/Applications/Anki.app"
rm -rf ~/Library/"Application Support"/Anki2   # profile data — destructive
```

---

## Clean-machine install (Windows / Linux)

| Platform | Artifact                                  | Install                    | GRE smoke test                 |
| -------- | ----------------------------------------- | -------------------------- | ------------------------------ |
| Windows  | `out/installer/dist/anki-*-windows-*.msi` | Run MSI wizard             | Same GRE shell checks as macOS |
| Linux    | `out/installer/dist/anki-*-linux-*.tar.*` | Extract and run `bin/anki` | Same GRE shell checks          |

See [../development.md](../development.md#installer) for upstream notes on Linux library dependencies.

---

## iOS companion (separate from desktop DMG)

The desktop installer does **not** include the iOS app. Build and run the companion separately:

```bash
cd mobile/ios
./scripts/build-mobile-bridge.sh
open GREAtlasCompanion.xcodeproj   # Product → Run on Simulator or device
```

Full walkthrough: [../../mobile/ios/DEMO.md](../../mobile/ios/DEMO.md).

Simulator build log: [logs/wednesday-xcodebuild-simulator.log](./logs/wednesday-xcodebuild-simulator.log).

---

## Troubleshooting

| Symptom                             | Fix                                                                                 |
| ----------------------------------- | ----------------------------------------------------------------------------------- |
| `tools/build-installer` fails early | Run `just check`; ensure Xcode CLT + Rust installed                                 |
| DMG missing after success           | Check `out/installer/logs/briefcase.*.package.log`                                  |
| GRE pages blank after install       | Confirm collection opened; GRE shell requires profile load                          |
| Practice errors                     | Check `{profile}/greatlas.db` exists; delete to regenerate (loses practice history) |
| iOS link errors                     | Re-run `./scripts/build-mobile-bridge.sh`; `./ninja rslib:proto` if protos stale    |

---

## Release engineer sign-off

Record after clean-machine smoke test:

| Check                                    | Pass? | Date | Notes |
| ---------------------------------------- | ----- | ---- | ----- |
| `tools/build-installer` exit 0           |       |      |       |
| DMG/MSI/tarball in `out/installer/dist/` |       |      |       |
| Clean install launches                   |       |      |       |
| GRE `/home` shell loads                  |       |      |       |
| One practice attempt recorded            |       |      |       |
| iOS Simulator build (optional)           |       |      |       |

Full pre-release matrix: [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md).
