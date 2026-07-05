# GRE Atlas — screen recording checklist

Required proof for the Friday/Sunday demo bundle. Recordings are **not** committed to git (see `recordings/.gitignore`). Place finished files in **`docs/gre-atlas-submission/recordings/`** before submission upload.

**Your checklist:** [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md) § C\
**Per-requirement scripts:** [recordings/REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md)\
**Pending placeholders:** [recordings/pending/](./recordings/pending/)

---

## Recording inventory

| # | Filename                      | Duration | Requirement                         | Status      | Script                                                                                |
| - | ----------------------------- | -------- | ----------------------------------- | ----------- | ------------------------------------------------------------------------------------- |
| 1 | `01-gre-shell-tour.mov`       | 2–3 min  | Give-up rule, GRE shell             | **MISSING** | [pending/01-gre-shell-tour.md](./recordings/pending/01-gre-shell-tour.md)             |
| 2 | `02-practice-isolation.mov`   | 2–3 min  | Practice / performance evidence     | **MISSING** | [pending/02-practice-isolation.md](./recordings/pending/02-practice-isolation.md)     |
| 3 | `03-study-plan-readiness.mov` | 2–3 min  | Three scores, ranges, evidence      | **MISSING** | [pending/03-study-plan-readiness.md](./recordings/pending/03-study-plan-readiness.md) |
| 4 | `04-eval-artifacts.mov`       | 1–2 min  | AI eval + baseline table            | **MISSING** | [pending/04-eval-artifacts.md](./recordings/pending/04-eval-artifacts.md)             |
| 5 | `05-sync-friday-loop.mov`     | 3–5 min  | Desktop↔phone sync, offline merge   | **MISSING** | [pending/05-sync-friday-loop.md](./recordings/pending/05-sync-friday-loop.md)         |
| 6 | `06-desktop-review.mov`       | 1–2 min  | Desktop GRE deck review             | **MISSING** | [pending/06-desktop-review.md](./recordings/pending/06-desktop-review.md)             |
| 7 | `07-phone-review.mov`         | 2–3 min  | iOS companion review                | **MISSING** | [pending/07-phone-review.md](./recordings/pending/07-phone-review.md)                 |
| 8 | `08-installer-smoke.mov`      | 1–2 min  | Clean-machine install (recommended) | **MISSING** | [pending/08-installer-smoke.md](./recordings/pending/08-installer-smoke.md)           |

Optional: `02-abstention-demo.mov` — see [REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md) § optional deep dive.

---

## Setup

- Use a **dev profile** (File → Switch Profile).
- macOS: QuickTime → New Screen Recording, or `Cmd+Shift+5` → Record Selected Portion.
- Hide personal Anki profiles and unrelated windows before recording.

---

## Acceptance criteria

- [ ] Filenames match the table above exactly (case-sensitive).
- [ ] No fabricated UI states — abstention on sparse profile is valid.
- [ ] Eval recording shows real output from `just eval-gre-atlas-ai` in this repo.
- [ ] Sync recording shows live hardware **or** terminal fallback test (see below).

---

## Automated sync alternative (recording #5)

If iOS hardware is unavailable, record this passing in terminal:

```bash
cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect -- --nocapture
```

This exercises desktop ↔ phone offline reconnect via in-process HTTP (same logic as [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md)).

---

## Demo script cross-reference

| Recording                     | Demo checklist section                                                         |
| ----------------------------- | ------------------------------------------------------------------------------ |
| `01-gre-shell-tour.mov`       | [DEMO-CHECKLIST §1–2](./DEMO-CHECKLIST.md#1-launch--gre-shell-2-min)           |
| `02-practice-isolation.mov`   | [DEMO-CHECKLIST §3](./DEMO-CHECKLIST.md#3-practice-3-min)                      |
| `03-study-plan-readiness.mov` | [DEMO-CHECKLIST §4–5](./DEMO-CHECKLIST.md#4-study-plan-2-min)                  |
| `04-eval-artifacts.mov`       | [DEMO-CHECKLIST §9](./DEMO-CHECKLIST.md#9-evaluation-artifacts-optional-1-min) |
| `05-sync-friday-loop.mov`     | [SYNC-DEV-SETUP.md](./SYNC-DEV-SETUP.md)                                       |
| `06-desktop-review.mov`       | [DEMO-CHECKLIST §7](./DEMO-CHECKLIST.md#7-anki-review-handoff-2-min)           |
| `07-phone-review.mov`         | [WEDNESDAY-PHONE-REVIEW.md](./WEDNESDAY-PHONE-REVIEW.md)                       |
| `08-installer-smoke.mov`      | [INSTALL.md](./INSTALL.md)                                                     |
