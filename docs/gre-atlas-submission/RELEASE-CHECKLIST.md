# GRE Atlas — Release Checklist

Use this checklist before public release **and** for the **Wednesday submission bundle**. Assumes macOS desktop + iOS companion; **Android is N/A** (no project in repo).

**Your personal pre-upload checklist:** [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md)

**Related docs:** [build.md](./build.md) · [tests.md](./tests.md) · [artifacts.md](./artifacts.md) · [release.md](./release.md) · [BUILD.md](./BUILD.md) · [INSTALL.md](./INSTALL.md) · [../gre-atlas-release.md](../gre-atlas-release.md) · [FRIDAY-DELIVERABLES.md](./FRIDAY-DELIVERABLES.md) · [ARTIFACT-AUDIT-2026-07-05.md](./ARTIFACT-AUDIT-2026-07-05.md)

**Release commit (2026-07-05 verification):** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

---

## 1. Pre-release build steps

- [ ] Clean checkout or tagged commit (avoid shipping from a dirty tree with large uncommitted diffs)
- [ ] `just check` passes (format, build, Rust/Python/TS tests)
- [x] `just build` passes — [build.md](./build.md) (2026-07-05)
- [x] `cargo test -p anki` — 709 passed, 178 `gre_atlas` tests — [tests.md](./tests.md)
- [x] `just test-py` / `just test-ts` green — [tests.md](./tests.md)
- [ ] `just eval-gre-atlas /path/to/collection.anki2` — eval report generates without error
- [ ] `just eval-gre-atlas-ai` — AI release gate passes (optional but recommended)
- [ ] Proto bindings regenerated after any `.proto` change (`./ninja rslib:proto ts:generated:proto pylib:anki:proto`)
- [ ] No untracked GRE source files (`rslib/src/gre_atlas/*`, `ts/routes/(gre)/*` committed)

**Build logs (Wednesday bundle):** see [logs/README.md](./logs/README.md). Summaries: [build.md](./build.md), [tests.md](./tests.md).

| Step                    | Log                                                                                                                                    |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `just build` / pylib+qt | [logs/wednesday-build.log](./logs/wednesday-build.log)                                                                                 |
| `cargo test -p anki`    | [logs/wednesday-cargo-test.log](./logs/wednesday-cargo-test.log)                                                                       |
| `just test-py`          | [logs/wednesday-pytest-direct.log](./logs/wednesday-pytest-direct.log), [logs/wednesday-pytest-qt.log](./logs/wednesday-pytest-qt.log) |
| `just test-ts`          | [logs/wednesday-vitest-direct.log](./logs/wednesday-vitest-direct.log)                                                                 |
| iOS simulator           | [logs/wednesday-xcodebuild-simulator.log](./logs/wednesday-xcodebuild-simulator.log)                                                   |

---

## 2. Desktop installer verification

### Build

- [ ] `just check` green on release branch
- [x] `tools/build-installer` artifact present (`RELEASE=2 ./ninja installer`) — [artifacts.md](./artifacts.md)
- [x] Artifact present in `out/installer/dist/` — `anki-26.05-mac-apple.dmg`
- [ ] Build log archived: [logs/wednesday-installer-build.log](./logs/wednesday-installer-build.log) (Briefcase logs in `out/installer/logs/`)

**Verified 2026-07-05 (macOS Apple Silicon):**

| Artifact       | Path                                          | Size   | Modified         |
| -------------- | --------------------------------------------- | ------ | ---------------- |
| DMG            | `out/installer/dist/anki-26.05-mac-apple.dmg` | 214 MB | 2026-07-05 14:35 |
| App bundle     | `out/installer/build/anki/macos/app/Anki.app` | 646 MB | 2026-07-05 14:34 |
| Briefcase logs | `out/installer/logs/briefcase.*.log`          | —      | —                |

> **Note:** `out/installer/` is gitignored. Copy the DMG to submission storage or attach to release; do not commit binaries.

### Clean-machine smoke test

Follow [INSTALL.md](./INSTALL.md). Minimum checks:

- [ ] Install from `out/installer/dist/` on a machine **without** the source tree
- [ ] Collection open → GRE main shell at `/home`
- [ ] Header nav: Home, Study, Practice, Progress, Settings
- [ ] One practice MCQ → performance count rises; `greatlas.db` created
- [ ] Readiness shows abstention checklist on fresh profile
- [ ] Study → Review uses standard Anki reviewer (FSRS unchanged)

Dev smoke (before packaging):

- [ ] `just run` → GRE shell loads at `/home`
- [ ] `just run-optimized` (optional perf check)

---

## 3. Mobile companion verification

### Build

- [ ] `./ninja rslib:proto` (once per clone)
- [x] `mobile/ios/scripts/build-mobile-bridge.sh` succeeds → `mobile/ios/out/lib/libmobile_bridge.a` — [artifacts.md](./artifacts.md)
- [x] Xcode: `GREAtlasCompanion.xcodeproj` builds for Simulator — [build.md](./build.md)

**Verified 2026-07-05:**

| Step                 | Log / evidence                                                                                             |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| mobile_bridge        | [logs/wednesday-mobile-bridge.log](./logs/wednesday-mobile-bridge.log)                                     |
| xcodebuild Simulator | [logs/wednesday-xcodebuild-simulator.log](./logs/wednesday-xcodebuild-simulator.log) — **BUILD SUCCEEDED** |

### Runtime checks

- [ ] First launch installs bundled demo from `Resources/DemoBundle/`
- [ ] Tabs: Dashboard, Study, Practice, Progress (+ sync in settings)
- [ ] Practice record + score strip match desktop semantics
- [ ] Study review renders cards via WebView
- [ ] GRE Atlas sync: pull/push/status; conflicts surfaced in UI
- [ ] Application Support: `GRE Atlas/collection.anki2` + `greatlas.db`
- [ ] Use `GREAtlasCompanion.xcodeproj` only (not legacy `BrainLiftCompanion/`)

Walkthrough: [../../mobile/ios/DEMO.md](../../mobile/ios/DEMO.md)

---

## 4. Wednesday submission proof / deliverables

Cross-check [FRIDAY-DELIVERABLES.md](./FRIDAY-DELIVERABLES.md) and [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md).

### Automated evidence (complete)

| Deliverable             | Location                                                                       | Status        |
| ----------------------- | ------------------------------------------------------------------------------ | ------------- |
| Eval report             | `results/gre-atlas-eval.{json,md}`, `performance-eval.md`                      | ✅ 2026-07-05 |
| AI eval + baselines     | `results/gre-atlas-ai-eval.{json,md}`, [AI.md](./AI.md)                        | ✅ PASS       |
| Benchmark               | `results/gre-atlas-benchmark.{json,md,csv}`                                    | ✅            |
| Sync verification       | [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md), `friday_sync_loop` test        | ✅            |
| Build/test logs         | `logs/wednesday-*.log`                                                         | ✅            |
| Desktop installer built | `out/installer/dist/anki-26.05-mac-apple.dmg`                                  | ✅ local only |
| Installer docs          | [INSTALL.md](./INSTALL.md), [../gre-atlas-release.md](../gre-atlas-release.md) | ✅            |

### Manual evidence (capture before submission)

| Deliverable                     | Doc                                                                                              | Status                    |
| ------------------------------- | ------------------------------------------------------------------------------------------------ | ------------------------- |
| UI screenshots 01–07            | [SCREENSHOTS.md](./SCREENSHOTS.md)                                                               | ⚠️ capture from `just run` |
| Eval/benchmark PNGs             | `screenshots/08-*.png`, `10-*.png`                                                               | ✅ partial                |
| Screen recordings               | [RECORDINGS.md](./RECORDINGS.md)                                                                 | ⚠️ `.mov` not in repo      |
| Clean-machine install recording | [recordings/REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md) §08 | ⚠️ optional                |
| Live demo script                | [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)                                                         | ✅                        |

---

## 5. Desktop — core GRE flows

### Launch & navigation

- [ ] Collection open lands in GRE main shell (`/home`)
- [ ] Header nav: Home, Study, Practice, Progress, Settings
- [ ] GRE modal dialog opens `/dashboard` with title **GRE Atlas**
- [ ] Bridge commands: start review, open practice, study plan, readiness, sync login/logout

### Review (memory)

- [ ] Deck **GRE Atlas** (or legacy **BrainLift GRE**) detected
- [ ] Study → Review starts Anki reviewer; FSRS scheduling unchanged
- [ ] After review session, home dashboard refreshes
- [ ] Cards tagged `gre::…` contribute to topic mastery and coverage

### Practice

- [ ] `/practice` loads seeded MCQs without network
- [ ] Submit answer → feedback + explanation
- [ ] Score strip updates after each attempt
- [ ] Section filters work; empty filter shows empty state

### Prediction dashboard

- [ ] `/home` and `/dashboard` show memory, performance, readiness, estimated GRE, coverage
- [ ] Abstention states show checklist requirements when data insufficient
- [ ] Weak topics and recommendations link to topic detail pages

### Readiness & calibration

- [ ] `/readiness` shows projected score or abstention
- [ ] Calibration panel shows bins / Brier when enough held-out predictions exist

### Sidecar database (`greatlas.db`)

- [ ] Fresh profile creates `greatlas.db` beside `collection.anki2`
- [ ] Practice attempts do **not** write to `revlog` or alter card scheduling
- [ ] **Known limitation:** Anki export/import does not bundle `greatlas.db`

---

## 6. Sync & offline

- [ ] **Offline:** all GRE pages work without network
- [ ] **AnkiWeb sync:** collection sync unchanged; GRE sidecar stays local
- [ ] **GRE Atlas practice sync (iOS):** pull/push via sync RPCs; newer mtime wins

---

## 7. AI features

- [ ] **AI disabled (default):** template-based generation; seeded question bank in UI
- [ ] No API keys required for shipping GRE product paths
- [ ] `GenerateQuestion` RPC not exposed on web mediasrv

---

## 8. Documentation index

| Doc                                                      | Purpose                                                      |
| -------------------------------------------------------- | ------------------------------------------------------------ |
| [README.md](./README.md)                                 | Submission package index                                     |
| [BUILD.md](./BUILD.md)                                   | Dev build + Wednesday build evidence (`just build`, iOS sim) |
| [INSTALL.md](./INSTALL.md)                               | Installer build + clean-machine smoke test                   |
| [../gre-atlas-release.md](../gre-atlas-release.md)       | Release & architecture overview                              |
| [../../README.md](../../README.md)                       | Repo root GRE Atlas quick start                              |
| [../../mobile/ios/README.md](../../mobile/ios/README.md) | iOS companion layout                                         |
| [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)                 | Live demo for graders                                        |

---

## 9. Known limitations (document, don't block)

| Item               | Notes                                             |
| ------------------ | ------------------------------------------------- |
| Android            | Not implemented                                   |
| Sidecar backup     | User must copy `greatlas.db` with profile         |
| Practice undo      | Not supported                                     |
| AI in UI           | Template engine only                              |
| Uncommitted WIP    | Tag/commit before release for reproducible builds |
| Installer binaries | Local `out/installer/` not in git                 |

---

## 10. Release day commands

```bash
just check
just run                              # final desktop dev smoke
just eval-gre-atlas "$PROFILE/collection.anki2"
just eval-gre-atlas-ai                # optional AI gate

tools/build-installer                 # desktop DMG/MSI/tarball → out/installer/dist/

# iOS
cd mobile/ios && ./scripts/build-mobile-bridge.sh
# Xcode → Product → Run (Simulator) or Archive (device)
```

Public multi-platform release (signed, CI): `just release::help` — see [../releasing.md](../releasing.md).

---

## Sign-off

| Role        | Name | Date | Ship? |
| ----------- | ---- | ---- | ----- |
| Engineering |      |      |       |
| QA          |      |      |       |
| Product     |      |      |       |
