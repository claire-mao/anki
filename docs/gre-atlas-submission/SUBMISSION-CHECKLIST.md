# GRE Atlas — final submission checklist (your actions)

**Engineering status:** complete (Wednesday / Friday / Sunday automated requirements pass).

**Commit (verified 2026-07-05):** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

**Start here for graders:** [SUBMISSION.md](./SUBMISSION.md) → [README.md](./README.md)

This checklist is **only** what you must do personally before upload. Do not skip items marked **required**.

---

## A. Pre-upload verification (15 min)

Run from repo root:

```bash
git rev-parse HEAD
just build
just test-py
just test-ts
cargo test -p anki gre_atlas
cargo test -p mobile_bridge
just eval-gre-atlas mobile/ios/GREAtlasCompanion/Resources/DemoBundle/collection.anki2
just eval-gre-atlas-ai
just bench-gre-atlas --synthetic-cards 10000
```

- [ ] All commands exit 0
- [ ] `docs/gre-atlas-submission/results/gre-atlas-eval.md` timestamp updated
- [ ] `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md` shows **Release verdict: PASS**

Optional full gate (requires `CONTRIBUTORS` entry for your git email):

```bash
just check
```

---

## B. Screenshots — **required** (see [SCREENSHOTS.md](./SCREENSHOTS.md))

Save PNGs to `docs/gre-atlas-submission/screenshots/`. Step-by-step: [screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md](./screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md).

| File                         | Status                        | Placeholder / script                                                                             |
| ---------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `01-gre-home.png`            | **YOU CAPTURE**               | [screenshots/pending/01-gre-home.md](./screenshots/pending/01-gre-home.md)                       |
| `02-abstention-progress.png` | **YOU CAPTURE**               | [screenshots/pending/02-abstention-progress.md](./screenshots/pending/02-abstention-progress.md) |
| `03-practice-question.png`   | **YOU CAPTURE**               | [screenshots/pending/03-practice-question.md](./screenshots/pending/03-practice-question.md)     |
| `04-study-plan.png`          | **YOU CAPTURE**               | [screenshots/pending/04-study-plan.md](./screenshots/pending/04-study-plan.md)                   |
| `05-readiness-scores.png`    | **YOU CAPTURE**               | [screenshots/pending/05-readiness-scores.md](./screenshots/pending/05-readiness-scores.md)       |
| `06-topic-detail.png`        | **YOU CAPTURE**               | [screenshots/pending/06-topic-detail.md](./screenshots/pending/06-topic-detail.md)               |
| `07-congrats-cta.png`        | **YOU CAPTURE**               | [screenshots/pending/07-congrats-cta.md](./screenshots/pending/07-congrats-cta.md)               |
| `08-eval-report.png`         | ✅ in repo                    | —                                                                                                |
| `09-unlocked-scores.png`     | **YOU CAPTURE** (recommended) | [screenshots/pending/09-unlocked-scores.md](./screenshots/pending/09-unlocked-scores.md)         |
| `10-benchmark-output.png`    | ✅ in repo                    | —                                                                                                |
| `11-gre-modal-dashboard.png` | optional                      | [screenshots/pending/11-gre-modal-dashboard.md](./screenshots/pending/11-gre-modal-dashboard.md) |

**Capture command:** `just run` → use dev profile → `Cmd+Shift+4`

After capture, delete matching files under `screenshots/pending/` or mark them done in this list.

---

## C. Screen recordings — **required** (see [RECORDINGS.md](./RECORDINGS.md))

Save `.mov` files to `docs/gre-atlas-submission/recordings/` (gitignored). Full scripts: [recordings/REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md).

| File                          | Status                                | Placeholder / script                                                                             |
| ----------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `01-gre-shell-tour.mov`       | **YOU RECORD**                        | [recordings/pending/01-gre-shell-tour.md](./recordings/pending/01-gre-shell-tour.md)             |
| `02-practice-isolation.mov`   | **YOU RECORD**                        | [recordings/pending/02-practice-isolation.md](./recordings/pending/02-practice-isolation.md)     |
| `03-study-plan-readiness.mov` | **YOU RECORD**                        | [recordings/pending/03-study-plan-readiness.md](./recordings/pending/03-study-plan-readiness.md) |
| `04-eval-artifacts.mov`       | **YOU RECORD**                        | [recordings/pending/04-eval-artifacts.md](./recordings/pending/04-eval-artifacts.md)             |
| `05-sync-friday-loop.mov`     | **YOU RECORD** (or terminal fallback) | [recordings/pending/05-sync-friday-loop.md](./recordings/pending/05-sync-friday-loop.md)         |
| `06-desktop-review.mov`       | **YOU RECORD**                        | [recordings/pending/06-desktop-review.md](./recordings/pending/06-desktop-review.md)             |
| `07-phone-review.mov`         | **YOU RECORD**                        | [recordings/pending/07-phone-review.md](./recordings/pending/07-phone-review.md)                 |
| `08-installer-smoke.mov`      | recommended                           | [recordings/pending/08-installer-smoke.md](./recordings/pending/08-installer-smoke.md)           |

**Sync fallback (no iOS hardware):**

```bash
cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect -- --nocapture
```

Record terminal output for `05-sync-friday-loop.mov`.

---

## D. Clean-machine desktop install — **required**

Follow [INSTALL.md](./INSTALL.md) on a Mac **without** this repo checked out.

1. Copy `out/installer/dist/anki-26.05-mac-apple.dmg` to test machine
2. Install → launch → GRE `/home` → one practice attempt → abstention on fresh profile
3. Record as `recordings/08-installer-smoke.mov` (or separate clean-install clip)

- [ ] Clean install succeeds
- [ ] GRE shell loads at `/home`
- [ ] Recording captured

**Rebuild installer if needed:**

```bash
tools/build-installer
ls -lh out/installer/dist/
```

---

## E. Physical iPhone / packaged mobile — **required if course expects device**

Follow [WEDNESDAY-PHONE-REVIEW.md](./WEDNESDAY-PHONE-REVIEW.md) and [mobile/ios/DEMO.md](../../mobile/ios/DEMO.md).

```bash
cd mobile/ios
PLATFORM_NAME=iphonesimulator ARCHS=arm64 ./scripts/build-mobile-bridge.sh
open GREAtlasCompanion.xcodeproj
# Xcode → select physical device → Product → Run (or Archive for .ipa)
```

- [ ] App runs on device or simulator (simulator build verified in [build.md](./build.md))
- [ ] `07-phone-review.mov` recorded
- [ ] Optional: export Archive / `.ipa` for Sunday packaged mobile proof

---

## F. Bundle for upload

Include this folder structure:

```
docs/gre-atlas-submission/
├── SUBMISSION.md
├── SUBMISSION-CHECKLIST.md   ← this file (check all boxes)
├── INSTALL.md
├── AI.md
├── ARCHITECTURE.md
├── RELEASE-CHECKLIST.md
├── SYNC-VERIFICATION.md
├── WEDNESDAY-PHONE-REVIEW.md
├── FRIDAY-DELIVERABLES.md
├── results/                  ← regenerated eval + benchmark
├── logs/                     ← wednesday-*.log
├── screenshots/              ← 01–07 PNGs + existing 08, 10
└── recordings/               ← .mov files (zip separately if large)
```

Also attach or note path to **`out/installer/dist/anki-26.05-mac-apple.dmg`** (not in git).

---

## G. Sign-off

| Item                                                   | Done |
| ------------------------------------------------------ | ---- |
| Screenshots `01`–`07` captured                         | ☐    |
| Recording `06-desktop-review.mov`                      | ☐    |
| Recording `07-phone-review.mov`                        | ☐    |
| Recording `05-sync-friday-loop.mov` (or test fallback) | ☐    |
| Clean-machine install verified                         | ☐    |
| Eval artifacts regenerated same day as submit          | ☐    |
| Commit hash in docs matches submitted tree             | ☐    |

**Highest priority if short on time:** capture `02-abstention-progress.png`, `05-readiness-scores.png`, `06-desktop-review.mov`, `07-phone-review.mov`, and complete clean-machine install recording.
