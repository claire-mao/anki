# GRE Atlas — grader one-page summary

**Start here:** [README.md](./README.md) (full table of contents)

## What was built

GRE Atlas is a GRE study layer on this Anki fork: practice MCQs, three abstaining scores (Memory / Performance / Readiness), study-plan recommendations, readiness calibration, and read-only evaluation. GRE practice writes to `greatlas.db` only — FSRS / revlog are unchanged.

## Quick start (5 commands)

```bash
just check                                              # format, build, test
just run                                                # GRE main shell at /home
just test-rust                                          # Rust unit tests (GRE Atlas modules)
just eval-gre-atlas /path/to/collection.anki2           # read-only eval report
just bench-gre-atlas --synthetic-cards 5000             # API latency benchmark
just eval-gre-atlas-ai                                  # AI gold-set eval (read-only)
```

## What to verify manually

1. **Shell:** `just run` → main window loads GRE at `/home` (header: Dashboard, Study, Practice, Progress, Settings).
2. **Abstention:** Fresh profile → Progress / Readiness show missing requirements (FSRS, 20 cards, 50% coverage, 50 practice attempts).
3. **Practice isolation:** Answer one MCQ → performance count rises; revlog unchanged (`pylib/tests/test_gre_atlas.py::test_gre_atlas_record_attempt_does_not_modify_revlog`).
4. **Study plan:** `/study-plan` shows ranked recommendations with factor explanations.
5. **Modal entry:** Congrats screen → Practice / Dashboard opens GRE modal at `/dashboard`.

Live demo script: [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md). Requirement mapping: [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md).

## Evaluation artifacts

| Output                | Path                                                                  | Contents                                       |
| --------------------- | --------------------------------------------------------------------- | ---------------------------------------------- |
| Full eval (JSON)      | `docs/gre-atlas-submission/results/gre-atlas-eval.json`               | All metrics, machine-readable                  |
| Full eval (Markdown)  | `docs/gre-atlas-submission/results/gre-atlas-eval.md`                 | Human-readable summary                         |
| Performance deep-dive | `docs/gre-atlas-submission/results/performance-eval.md`               | Held-out precision/recall/F1, confusion matrix |
| Benchmark             | `docs/gre-atlas-submission/results/gre-atlas-benchmark.{json,md,csv}` | p50 / p95 / worst-case API latency             |
| AI eval               | `docs/gre-atlas-submission/results/gre-atlas-ai-eval.{json,md}`       | Gold-set baseline vs template generation       |

`just eval-gre-atlas` writes all three eval paths in one run. See [EVALUATION.md](./EVALUATION.md). AI pipeline: [AI.md](./AI.md).

## Architecture at a glance

- **18 BrainLiftService RPCs** + `StatsService.TopicMastery` — [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Rust engine:** `rslib/src/gre_atlas/`
- **GRE UI:** `ts/routes/(gre)/`
- **Feature → source map:** [FEATURE-INDEX.md](./FEATURE-INDEX.md)

## Screenshots

Capture checklist and filenames: [SCREENSHOTS.md](./SCREENSHOTS.md). Place PNGs under [screenshots/](./screenshots/).

## Note on entry points

Collection open lands in the **GRE main shell** at `/home` (`greDashboard` state). The **GRE modal dialog** opens at `/dashboard` from congrats CTAs and `open_gre_atlas()`. The Qt menu bar has **GRE → Debug** only; there is no separate “Open GRE” menu item in this build.
