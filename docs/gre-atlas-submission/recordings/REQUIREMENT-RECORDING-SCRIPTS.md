# GRE Atlas — requirement recording scripts

Exact step-by-step scripts for Friday submission proof. Save finished files under `docs/gre-atlas-submission/recordings/` (gitignored). Filenames below are **required** unless noted as optional.

**Setup (all recordings):**

```bash
cd /path/to/anki
just check   # confirm green before demo
```

Use a **dev profile** (File → Switch Profile). Hide personal windows. macOS: QuickTime → New Screen Recording, or `Cmd+Shift+5` → Record Selected Portion.

---

## Requirement → recording map

| Requirement          | Primary file                                          | Alt / supplement                                |
| -------------------- | ----------------------------------------------------- | ----------------------------------------------- |
| AI evaluation        | `04-eval-artifacts.mov`                               | —                                               |
| Baseline comparison  | `04-eval-artifacts.mov` (same; scroll baseline table) | —                                               |
| Desktop review       | `06-desktop-review.mov`                               | `01-gre-shell-tour.mov` § Study                 |
| Phone review         | `07-phone-review.mov`                                 | —                                               |
| Desktop–phone sync   | `05-sync-friday-loop.mov`                             | terminal fallback below                         |
| Offline sync         | `05-sync-friday-loop.mov` (offline segment)           | automated test fallback                         |
| Three scores         | `03-study-plan-readiness.mov`                         | seeded profile required                         |
| Give-up rule         | `01-gre-shell-tour.mov` (Progress abstention)         | `02-abstention-demo.mov` optional               |
| Uncertainty (ranges) | `03-study-plan-readiness.mov` (confidence band)       | seeded profile                                  |
| Evidence             | `03-study-plan-readiness.mov` (evidence cards)        | —                                               |
| Installer            | `08-installer-smoke.mov`                              | optional if grader runs `tools/build-installer` |

---

## `01-gre-shell-tour.mov` — GRE shell + give-up rule (2–3 min)

**Covers:** Give-up rule (abstention), partial Evidence (checklist UI)

1. Terminal: `just run` (show command, wait for Anki window).
2. Confirm landing at **`/home`**: header shows Dashboard, Study, Practice, Progress, Settings.
3. Pan home: daily mission / study teaser if present.
4. Click **Progress** (`/progress`).
5. **Hold 5 s** on abstention checklist: FSRS, studied cards, coverage, practice attempts with counts.
6. Narrate (optional): “Three scores abstain rather than fabricate numbers.”
7. Click **Settings** briefly, show FSRS toggle location (Settings → Preferences path if needed).

**Profile:** Fresh or sparse dev profile (abstention visible).

---

## `02-practice-isolation.mov` — Practice + revlog isolation (2–3 min)

**Covers:** Performance evidence path (attempt count)

1. From GRE shell, open **Practice** (`/practice`).
2. Show one MCQ with four choices; select an answer; submit.
3. Show feedback / explanation panel.
4. Open **Progress** or **Readiness**; point to performance attempt count increment.
5. Verbal note: practice writes `greatlas.db` only, not revlog (no Deck Browser proof required).

**Profile:** Any profile with practice bank loaded.

---

## `03-study-plan-readiness.mov` — Three scores, evidence, uncertainty (2–3 min)

**Covers:** Three scores, Evidence cards, Uncertainty (ranges)

**Profile (seed before record):**

- FSRS on; GRE Atlas deck with 50+ reviewed tagged cards OR use demo seed path in [DEMO-CHECKLIST.md](../DEMO-CHECKLIST.md#demo-data-shortcuts).
- 50+ practice attempts if Performance/Readiness should unlock.

1. Open **Study plan** (`/study-plan`); scroll ranked recommendations and factor text (~20 s).
2. Open **Readiness** (`/readiness`).
3. **Hold 8 s** on three evidence cards: **Memory**, **Performance**, **Readiness**.
4. If unlocked: show projected score **and confidence band / range** (low–high on memory; readiness interval).
5. If still abstaining: show which cards abstain and unmet requirements (honest state OK).
6. Expand **Calibration** panel if data exists; otherwise show “insufficient history” message.

---

## `04-eval-artifacts.mov` — AI evaluation + baseline comparison (1–2 min)

**Covers:** AI evaluation, Baseline comparison

1. Terminal full screen or large pane.
2. Run:

```bash
cd /path/to/anki
just eval-gre-atlas-ai
```

3. Open `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md` in editor.
4. Scroll **Release gate** → **Benchmark comparison** table (keyword, BM25, TF-IDF, ai_retrieval, ai_generation_pipeline).
5. **Hold 5 s** on “AI beats all baselines” / verdict lines.
6. Optional: `just eval-gre-atlas mobile/ios/GREAtlasCompanion/Resources/DemoBundle/collection.anki2` and scroll Abstention section.

---

## `05-sync-friday-loop.mov` — Desktop–phone sync + offline reconnect (3–5 min)

**Covers:** Desktop–phone sync, Offline sync

### Option A — Live hardware (preferred if available)

Follow [SYNC-DEV-SETUP.md](../SYNC-DEV-SETUP.md):

1. Terminal 1: start SimpleServer with `SYNC_USER1=dev:dev`.
2. Desktop Anki: Settings → sync login to `http://127.0.0.1:8080`, user `dev`.
3. iOS Simulator or device: same server credentials in companion settings.
4. Desktop: record one practice attempt → Sync → show iOS Practice tab count match.
5. **Airplane mode ON** on phone; record attempt offline on phone.
6. Desktop offline: record attempt (or disable network).
7. Reconnect both → Sync → show merged counts.
8. State clearly: **practice sidecar only**; Anki card review does not sync cross-device today.

### Option B — Automated substitute (no iOS)

```bash
cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect -- --nocapture
```

Record terminal through `test ... ok` (~5 s hold). Say on-screen: “In-process HTTP E2E: desktop ↔ phone offline reconnect.”

---

## `06-desktop-review.mov` — Desktop memory review (1–2 min)

**Covers:** Desktop review

1. `just run` → GRE shell at `/home`.
2. Ensure **GRE Atlas** deck exists (create or use seeded profile).
3. Click **Study** in header → Anki reviewer opens.
4. Show one card front/back; rate (Again/Hard/Good/Easy).
5. Exit review → return to GRE shell without broken state.
6. Open **Progress** → studied count or memory evidence updated (or abstention checklist progress).

---

## `07-phone-review.mov` — iOS companion review (2–3 min)

**Covers:** Phone review

1. Xcode: open `mobile/ios/GREAtlasCompanion.xcodeproj`, run on Simulator (iPhone 17).
2. First launch: demo bundle installs (brief).
3. Tap **Study** tab → WebView reviewer shows a card.
4. Rate one card; return to tabs.
5. Show **Dashboard** or **Progress** reflecting review (memory signal).
6. Note: same FSRS engine via Rust bridge; not mock Swift data.

See [mobile/ios/DEMO.md](../../../mobile/ios/DEMO.md).

---

## `08-installer-smoke.mov` — Installer (optional, 1–2 min)

**Covers:** Installer

1. Show build (or prebuilt artifact):

```bash
tools/build-installer
ls -lh out/installer/dist/
```

2. Open `.dmg` (macOS), drag Anki to Applications.
3. Launch installed app → GRE shell at `/home`.
4. One practice attempt → confirm UI loads.

Reference: [docs/gre-atlas-release.md](../../gre-atlas-release.md), `qt/installer/`.

---

## `02-abstention-demo.mov` — Optional deep dive on give-up rule (1 min)

**Covers:** Give-up rule only

1. Fresh profile, FSRS off or 0 cards.
2. Progress → Readiness → Home score strip: every numeric score hidden; requirements listed.
3. Enable FSRS, review 3 cards, re-open Progress: checklist partial progress visible.

---

## Acceptance checklist

- [ ] All required `.mov` files saved under `recordings/`
- [ ] Filenames match this doc and [RECORDINGS.md](../RECORDINGS.md)
- [ ] No fabricated UI (abstention OK on sparse profile)
- [ ] Eval recording shows real `just eval-gre-atlas-ai` output from this repo
