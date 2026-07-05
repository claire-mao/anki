# GRE Atlas — demo checklist

Use this script for a **10–15 minute live demo** or screen recording. Assumes a fresh dev build (`just run`).

## Before you start

- [ ] `just check` passes on the submission branch
- [ ] Use a **dev profile** (File → Switch Profile) so demo data does not mix with personal collections
- [ ] FSRS enabled (Settings → Preferences → Scheduling → FSRS)
- [ ] Optional: empty profile to show abstention, or profile with GRE deck + reviews for unlocked scores

## 1. Launch & GRE shell (2 min)

- [ ] Run `just run`
- [ ] App opens into **GRE shell** (main window) at **`/home`**
- [ ] Header shows: Dashboard, Study, Practice, Progress, Settings
- [ ] Daily mission / study plan teaser visible on home (if recommendations exist)

**Say:** GRE Atlas is a parallel GRE product layer; Anki FSRS remains the memory engine.

## 2. Memory & abstention (2 min)

- [ ] Open **Progress** (`/progress`)
- [ ] If sparse data: Memory / Readiness show **abstention** with explicit missing requirements (FSRS, studied cards, coverage, practice attempts)
- [ ] If seeded: topic mastery chart and coverage summary appear

**Optional seed path:**

- Create deck **GRE Atlas**, add cards tagged `gre::quant::algebra::linear`
- Review a few cards → return to Progress → studied count increases

**Say:** Three scores abstain honestly rather than showing fabricated numbers.

## 3. Practice (3 min)

- [ ] Open **Practice** (`/practice`)
- [ ] At least one GRE MCQ loads from the embedded question bank
- [ ] Submit an answer → feedback shown
- [ ] Open **Progress** or **Readiness** → performance attempt count increased

**Verify isolation (verbal):**

- Practice attempts live in `greatlas.db`; they do not alter card scheduling or revlog.

## 4. Study plan (2 min)

- [ ] From home, follow link to **Study plan** (`/study-plan`) or use in-page navigation
- [ ] Ranked topic recommendations with explanations and factors (coverage gap, low mastery, etc.)
- [ ] Daily plan tasks: review target, practice target, focus topics

**Say:** Recommendations use exam-weighted priority, not vanilla deck order.

## 5. Readiness & calibration (2 min)

- [ ] Open **Readiness** (`/readiness`)
- [ ] Three evidence cards: Memory, Performance, Readiness
- [ ] If sufficient data: projected score + confidence band
- [ ] Calibration section shows Brier / assessment (or insufficient-history message)

## 6. Topic drill-down (1 min)

- [ ] From Progress chart or study plan, open a **topic detail** page (`/topics/...`)
- [ ] Topic metadata, readiness contribution, practice questions listed

## 7. Anki review handoff (2 min)

- [ ] Click **Study** in header → Anki reviewer opens (GRE deck if configured)
- [ ] Complete or exit review → returns to GRE shell (not broken state)
- [ ] Memory evidence on dashboard/progress reflects new reviews

## 8. Congrats funnel (optional, 1 min)

- [ ] Study a **non-GRE** deck until congrats screen
- [ ] **Practice** / **Dashboard** buttons on congrats → GRE modal dialog opens

**Note:** Congrats opens the modal dialog (`/dashboard`), not the main shell (`/home`). Both are valid GRE entry points.

## 9. Evaluation artifacts (optional, 1 min)

```bash
just eval-gre-atlas /path/to/profile/collection.anki2
just eval-gre-atlas-ai
just bench-gre-atlas --synthetic-cards 10000
```

- [ ] Show `docs/gre-atlas-submission/results/gre-atlas-eval.md` sections: calibration, memory, performance, ablation
- [ ] Show `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md` baseline comparison table
- [ ] Show `docs/gre-atlas-submission/results/gre-atlas-benchmark.md` p50/p95 table
- [ ] Optional: show `docs/gre-atlas-submission/results/performance-eval.md` (precision/recall/F1 deep-dive)
- [ ] Point out held-out rule `id % 5 == 0` and synthetic reference labeling

## 10. Screen recordings (Friday proof)

Follow [RECORDINGS.md](./RECORDINGS.md). Minimum: GRE shell tour, practice isolation, eval artifacts terminal scroll.

Place finished `.mov` files in `recordings/` (gitignored).

## Demo data shortcuts

| Goal                 | Action                                                      |
| -------------------- | ----------------------------------------------------------- |
| Fast abstention demo | Fresh profile, no GRE deck                                  |
| Fast practice demo   | Open Practice immediately (seeded question bank)            |
| Unlock performance   | 50+ practice attempts (or use demo seed on mobile)          |
| Unlock memory        | FSRS + 20+ reviewed GRE-tagged cards + 50% catalog coverage |

## Troubleshooting during demo

| Issue              | Workaround                                                        |
| ------------------ | ----------------------------------------------------------------- |
| GRE page blank     | Confirm `just run` (not external browser without API)             |
| 403 on RPC         | Restart Anki; check mediasrv running on port 40000                |
| Readiness abstains | Expected on sparse data — show requirement checklist              |
| No GRE deck        | Create **GRE Atlas** deck or use study status message on Settings |
