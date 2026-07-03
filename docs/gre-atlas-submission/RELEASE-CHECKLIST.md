# GRE Atlas — Pre-Release Checklist

Use this checklist the day before and morning of public release. Assumes macOS desktop + iOS companion; **Android is N/A** (no project in repo).

---

## Build & CI

- [ ] Clean checkout or tagged commit (avoid shipping from a dirty tree with ~100+ uncommitted files unless intentional)
- [ ] `just check` passes (format, build, Rust/Python/TS tests)
- [ ] `cargo test -p anki gre_atlas` — 107+ gre_atlas unit tests green
- [ ] `just eval-gre-atlas /path/to/collection.anki2` — eval report generates without error
- [ ] Desktop dev run: `just run` → GRE shell loads at `/home`
- [ ] Release build smoke: `just run-optimized` (optional but recommended for perf)
- [ ] iOS bridge: `mobile/ios/scripts/build-mobile-bridge.sh` succeeds
- [ ] iOS: open `mobile/ios/GREAtlasCompanion.xcodeproj`, build for Simulator + device arch
- [ ] Installer packaging: `just` release recipes (see `docs/gre-atlas-release.md`) — run on release machine outside sandbox

---

## Desktop — Core GRE flows

### Launch & navigation

- [ ] Collection open lands in GRE main shell (`/home`)
- [ ] Header nav: Home, Study, Practice, Progress, Settings
- [ ] GRE modal dialog (`GRE →` congrats CTAs) opens `/dashboard` with title **GRE Atlas**
- [ ] Bridge commands work: start review, open practice, study plan, readiness, sync login/logout

### Review (memory)

- [ ] Deck **GRE Atlas** (or legacy **BrainLift GRE**) detected
- [ ] Study → Review starts Anki reviewer; FSRS scheduling unchanged
- [ ] After review session, home dashboard refreshes (metric snapshot / due counts)
- [ ] Cards tagged `gre::…` contribute to topic mastery and coverage

### Practice

- [ ] `/practice` loads seeded MCQs without network
- [ ] Submit answer → feedback + explanation
- [ ] Score strip updates after each attempt (performance attempt count)
- [ ] Section filters (quant / verbal / awa) work; empty filter shows empty state
- [ ] Invalid session shows user-friendly error (not raw Rust traceback)

### Prediction dashboard

- [ ] `/home` and `/dashboard` show memory, performance, readiness, estimated GRE, coverage
- [ ] Abstention states show checklist requirements when data insufficient
- [ ] Weak topics and recommendations link to topic detail pages
- [ ] Metric change indicators behave after review + practice

### Readiness & calibration

- [ ] `/readiness` shows projected score or abstention
- [ ] Calibration panel shows bins / Brier when enough held-out predictions exist
- [ ] Progress charts load without console errors

### Coverage & study plan

- [ ] Coverage summary reflects tagged cards in GRE deck
- [ ] `/study-plan` ranks topics; daily mission tasks actionable

### Settings

- [ ] `/settings` loads preferences and deck config
- [ ] FSRS / scheduling toggles persist
- [ ] Deck options bridge opens for GRE Atlas deck
- [ ] Sync login/logout from settings (AnkiWeb collection sync — separate from GRE Atlas practice sync)

---

## Sidecar database (`greatlas.db`)

- [ ] Fresh profile creates `greatlas.db` beside `collection.anki2`
- [ ] Legacy `greatlas.db` renamed to `greatlas.db` on first open (data preserved)
- [ ] Schema v4 migration runs on older sidecars
- [ ] Practice attempts do **not** write to `revlog` or alter card scheduling
- [ ] **Known limitation:** standard Anki export/import does not bundle `greatlas.db`; back up the profile folder or copy the file manually

---

## Sync & offline

- [ ] **Offline:** all GRE pages work without network (seeded questions, local scores)
- [ ] **AnkiWeb sync:** collection sync unchanged; GRE sidecar stays local
- [ ] **GRE Atlas practice sync (iOS):** pull/push attempts via `GetBrainLiftSyncStatus` / Pull / Push — newer mtime wins
- [ ] Sync conflicts surfaced in iOS UI (not silent data loss)

---

## Recovery & integrity

- [ ] Missing sidecar: app creates empty DB and seeds questions
- [ ] Corrupt sidecar: verify graceful error (note path — manual recovery = delete `greatlas.db` to regenerate; practice history lost)
- [ ] Missing GRE deck: empty states + “Create GRE Atlas deck” guidance
- [ ] **Undo:** practice attempts are not undoable (by design); card review undo unchanged

---

## AI features

- [ ] **AI disabled (default):** template-based generation in rslib; UI uses seeded question bank only
- [ ] No API keys required for shipping GRE product paths
- [ ] `GenerateQuestion` RPC exists but is **not** exposed on web mediasrv — no user-facing AI toggle in GRE routes
- [ ] Eval harness: `just eval-gre-atlas-ai` (optional, dev-only)

---

## iOS companion (`GREAtlasCompanion`)

- [ ] First launch installs bundled demo from `Resources/DemoBundle/`
- [ ] Tabs: Dashboard, Study, Practice, Progress (+ sync in settings)
- [ ] Practice record + score strip match desktop semantics
- [ ] Study review renders cards via WebView
- [ ] Demo banner shows due counts / practice bank size
- [ ] Application Support path: `GRE Atlas/collection.anki2` + `greatlas.db`
- [ ] **Do not** open old `BrainLiftCompanion/` Xcode project — use `GREAtlasCompanion.xcodeproj` only
- [ ] Remove stale `mobile/ios/BrainLiftCompanion/` tree from release branch if still present locally

---

## UX & error handling

- [ ] No user-visible **BrainLift** strings in GRE Svelte routes (internal protobuf/API names OK)
- [ ] Legacy deck name **BrainLift GRE** still recognized for compatibility
- [ ] Practice errors: generic message, no backend exception text in UI
- [ ] **Known limitation:** mediasrv may return raw exception strings on 500 for unhandled backend errors (inherited Anki behavior)

---

## Documentation

- [ ] [README.md](../../README.md) — GRE Atlas quick start
- [ ] [docs/gre-atlas-release.md](../gre-atlas-release.md) — build & ship
- [ ] [mobile/ios/README.md](../../mobile/ios/README.md) + [DEMO.md](../../mobile/ios/DEMO.md)
- [ ] [docs/gre-atlas-submission/](./) — submission package complete
- [ ] Internal docs may still say `greatlas.db`; user-facing sidecar filename is **`greatlas.db`**

---

## Known limitations (document, don’t block)

| Item               | Notes                                                                      |
| ------------------ | -------------------------------------------------------------------------- |
| Android            | Not implemented                                                            |
| Sidecar backup     | User must copy `greatlas.db` with profile                                  |
| Practice undo      | Not supported                                                              |
| AI in UI           | Template engine only; no LLM in product UI                                 |
| Dual sidecar files | If both `greatlas.db` and `greatlas.db` exist, only `greatlas.db` is used  |
| Uncommitted WIP    | Large dirty tree risks non-reproducible builds — tag/commit before release |

---

## Release day commands

```bash
just check
just run                    # final desktop smoke
just eval-gre-atlas "$PROFILE/collection.anki2"

# iOS
cd mobile/ios && ./scripts/build-mobile-bridge.sh
# Then Xcode → Product → Archive
```

---

## Sign-off

| Role        | Name | Date | Ship? |
| ----------- | ---- | ---- | ----- |
| Engineering |      |      |       |
| QA          |      |      |       |
| Product     |      |      |       |
