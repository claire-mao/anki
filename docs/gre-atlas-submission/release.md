# GRE Atlas — Wednesday release & install guide

**Audit date:** 2026-07-05\
**Commit:** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

Cross-references:

- [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md) — full pre-release manual QA
- [BUILD.md](./BUILD.md) — dev build + Wednesday evidence
- [build.md](./build.md) / [tests.md](./tests.md) / [artifacts.md](./artifacts.md) — Wednesday automated evidence
- [../gre-atlas-release.md](../gre-atlas-release.md) — upstream-style release doc
- [SUBMISSION.md](./SUBMISSION.md) — grader one-pager

---

## Release checklist (Wednesday proof)

Automated gates captured 2026-07-05:

- [x] Record commit hash (`1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`)
- [x] `just build` succeeds — [build.md](./build.md)
- [x] `cargo test -p anki` — 709 passed, 0 failed — [tests.md](./tests.md)
- [x] `just test-py` green (pytest stamps valid; 218 tests on direct re-run)
- [x] `just test-ts` green (111 Vitest tests on direct re-run)
- [x] macOS installer artifact present — [artifacts.md](./artifacts.md)
- [x] iOS simulator build succeeds — [build.md](./build.md)
- [x] `MIN_STUDIED_CARDS = 50` in Rust + TS abstention code
- [ ] `just check` full gate (format + all checks) — run before ship if tree is dirty
- [ ] Manual desktop smoke (`just run` → GRE `/home`, one practice attempt)
- [ ] Manual iOS device/simulator tour — see recording section below
- [ ] UI screenshots `01`–`07` — [SCREENSHOTS.md](./SCREENSHOTS.md)
- [ ] Screen recordings — [RECORDINGS.md](./RECORDINGS.md)

Full manual checklist: [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md).

---

## Build & package (release machine)

```bash
# 1. Full automated gate
just check

# 2. Desktop installer (macOS)
tools/build-installer
# Output: out/installer/dist/anki-26.05-mac-apple.dmg

# 3. iOS companion
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
# Xcode → Product → Archive (device release)
# Or simulator CLI build — see build.md
```

---

## Clean-machine desktop install

On a Mac **without** the dev tree:

1. Copy `out/installer/dist/anki-26.05-mac-apple.dmg` to the test machine.
2. Open the DMG and drag **Anki.app** to **Applications**.
3. First launch: macOS may prompt to allow the app (right-click → Open if Gatekeeper blocks).
4. Create or open a **dev profile** (File → Switch Profile) — avoid personal collections.
5. **Verify GRE Atlas:**
   - Collection open lands at GRE **`/home`** (header: Dashboard, Study, Practice, Progress, Settings).
   - Open **Practice** → answer one MCQ → confirm performance attempt count updates.
   - Open **Progress** → on a fresh profile, confirm **abstention** lists requirements including **50 reviewed cards** (not 20).
   - Optional: finish a review session → congrats links open GRE modal at `/dashboard`.

**Sidecar note:** GRE practice data lives in `greatlas.db` beside `collection.anki2`. Standard Anki export does not bundle this file — back up the profile folder manually.

---

## iOS companion — simulator & phone review

### Simulator (quick demo)

```bash
open mobile/ios/GREAtlasCompanion.xcodeproj
# Scheme: GREAtlasCompanion → iPhone 17 simulator → ⌘R
```

First launch installs bundled demo from `Resources/DemoBundle/`. Tabs: Dashboard, Study, Practice, Progress.

### Physical iPhone

1. Connect device; set **Signing & Capabilities → Team** in Xcode.
2. Select device as run destination → **Product → Run**.
3. Verify practice record + score strip match desktop semantics.
4. Optional sync: configure GRE Atlas practice sync in Settings (see [SYNC-DEV-SETUP.md](./SYNC-DEV-SETUP.md)).

### Phone review recording instructions

Record a **2–3 minute** screen capture for submission (filename suggestions in [RECORDINGS.md](./RECORDINGS.md)):

| Step | Action                                                                                         |
| ---- | ---------------------------------------------------------------------------------------------- |
| 1    | Launch **GREAtlasCompanion** on simulator or device (demo profile).                            |
| 2    | **Dashboard tab** — show memory / performance / readiness cards (abstention OK on fresh demo). |
| 3    | **Practice tab** — answer one MCQ; show feedback and score strip update.                       |
| 4    | **Study tab** — open review (WebView card); complete or skip one card.                         |
| 5    | **Progress tab** — scroll evidence / weak topics.                                              |
| 6    | _(Optional)_ Settings → sync status if demonstrating Friday sync loop.                         |

**Recording tips:**

- macOS: QuickTime → New Screen Recording, or `Cmd+Shift+5`.
- Hide unrelated notifications; use demo profile only.
- Save to `docs/gre-atlas-submission/recordings/` (gitignored) — see [recordings/README.md](./recordings/README.md).
- If no physical device: record terminal passing `cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect -- --nocapture` as sync alternative ([RECORDINGS.md §5](./RECORDINGS.md)).

Demo script alignment: [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md), [../../mobile/ios/DEMO.md](../../mobile/ios/DEMO.md).

---

## Abstention thresholds (shipping values)

| Gate              | Value               | Source                                               |
| ----------------- | ------------------- | ---------------------------------------------------- |
| Studied cards     | **50**              | `rslib/src/gre_atlas/abstention.rs`, TS empty-states |
| Topic coverage    | **50%**             | same                                                 |
| Practice attempts | **50**              | same                                                 |
| FSRS              | enabled on GRE deck | same                                                 |

Do **not** document 20- or 200-card thresholds for GRE Atlas memory abstention in new submission docs.

---

## Sign-off

| Role        | Name | Date | Ship? |
| ----------- | ---- | ---- | ----- |
| Engineering |      |      |       |
| QA          |      |      |       |
| Product     |      |      |       |
