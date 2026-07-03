# GRE Atlas — submission package

This folder is the **grader entry point** for the GRE Atlas layer built on Anki. It summarizes what was delivered, how to build and run it, how to reproduce evaluation artifacts, and how to score the project against requirements.

**One-page summary:** [SUBMISSION.md](./SUBMISSION.md)

## Table of contents

| Document                                       | Purpose                                                               |
| ---------------------------------------------- | --------------------------------------------------------------------- |
| [SUBMISSION.md](./SUBMISSION.md)               | One-page grader quick start — commands, artifacts, manual checks      |
| [BUILD.md](./BUILD.md)                         | Clean checkout → `just check` → `just run`                            |
| [EVALUATION.md](./EVALUATION.md)               | Reproducible eval + benchmark commands and outputs                    |
| [ARCHITECTURE.md](./ARCHITECTURE.md)           | System diagram (Mermaid) and layer responsibilities                   |
| [FEATURE-INDEX.md](./FEATURE-INDEX.md)         | Feature → Rust source → tests → doc section                           |
| [PERFORMANCE-MODEL.md](./PERFORMANCE-MODEL.md) | Performance model eval methodology (held-out split, metrics)          |
| [AI.md](./AI.md)                               | Template-based question generation, gold-set eval, source attribution |
| [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)       | Live demo script for reviewers                                        |
| [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md) | Requirement → evidence mapping                                        |
| [SCREENSHOTS.md](./SCREENSHOTS.md)             | Screenshot capture checklist and filenames                            |
| [screenshots/](./screenshots/)                 | PNG captures for UI evidence (gitignored)                             |
| [results/](./results/)                         | Submission eval outputs (`performance-eval.md`)                       |

## What GRE Atlas is

GRE Atlas is a GRE study product embedded in this Anki fork. It adds:

- A **GRE web shell** (SvelteKit pages under `ts/routes/(gre)/`)
- A **parallel data path** (`greatlas.db` for practice + calibration; `collection.anki2` for FSRS)
- Three scores — **Memory**, **Performance**, **Readiness** — with honest **abstention** when evidence is insufficient
- **Study plan** topic-priority recommendations
- **Read-only evaluation** (calibration, FSRS memory eval, performance eval, ablation, benchmarks)
- **AI question generation** (template pipeline from named ETS source, gold-set eval, confidence rejection)

Anki’s reviewer and FSRS scheduler are **not modified** by GRE practice.

## Deep reference (existing docs)

| Topic                                   | Location                                                                           |
| --------------------------------------- | ---------------------------------------------------------------------------------- |
| Codebase map                            | [../gre-atlas-architecture.md](../gre-atlas-architecture.md)                       |
| Product design & phases                 | [../gre-atlas-product-architecture.md](../gre-atlas-product-architecture.md)       |
| Release & installer                     | [../gre-atlas-release.md](../gre-atlas-release.md)                                 |
| Memory / performance / readiness models | [../models/](../models/)                                                           |
| Eval methodology (full)                 | [../../scripts/eval/README.md](../../scripts/eval/README.md)                       |
| TopicMastery Rust note                  | [../gre-atlas-topic-mastery-rust-note.md](../gre-atlas-topic-mastery-rust-note.md) |
| iOS companion                           | [../gre-atlas-mobile.md](../gre-atlas-mobile.md)                                   |

## Quick start (reviewers)

```bash
# Prerequisites: see docs/development.md
just check          # format, build, test
just run            # opens Anki → GRE shell at /home

# Optional: regenerate evaluation report (read-only)
just eval-gre-atlas /path/to/collection.anki2

# Optional: benchmark production APIs
just bench-gre-atlas --synthetic-cards 10000

# Optional: AI gold-set eval (read-only, no collection required)
just eval-gre-atlas-ai
```

## Key deliverables map

| Deliverable                   | Primary location                                                          |
| ----------------------------- | ------------------------------------------------------------------------- |
| Protobuf API                  | `proto/anki/brainlift.proto` (+ `StatsService.TopicMastery`)              |
| Rust engine                   | `rslib/src/gre_atlas/`                                                    |
| Topic mastery (memory signal) | `rslib/src/stats/mastery.rs`                                              |
| Python wrappers               | `pylib/anki/gre_atlas.py`                                                 |
| GRE UI                        | `ts/routes/(gre)/`                                                        |
| Qt shell                      | `qt/aqt/gre_dashboard.py`, `qt/aqt/gre_atlas.py`                          |
| Eval harness                  | `scripts/eval/`, `rslib/src/gre_atlas/eval.rs`                            |
| AI generation + eval          | `rslib/src/gre_atlas/questions/ai_gen.rs`, `ai_eval.rs`, [AI.md](./AI.md) |
| Tests                         | `rslib/src/gre_atlas/**` test modules, `pylib/tests/test_gre_atlas*.py`   |

## RPC inventory (18 + TopicMastery)

`BrainLiftService`: practice, dashboard, scores, study plan, readiness calibration, topic details, eval reports (full + AI), question generation, sync (3), demo bootstrap.

`StatsService.TopicMastery`: FSRS retrievability aggregation by GRE topic tags.

Full list and layer wiring: [ARCHITECTURE.md](./ARCHITECTURE.md).

## Screenshots

Capture checklist: [SCREENSHOTS.md](./SCREENSHOTS.md). Example filenames: `screenshots/01-gre-home.png`, `screenshots/05-readiness-scores.png`.

## Entry points (desktop)

| Surface                 | Default route                                  | How to reach                           |
| ----------------------- | ---------------------------------------------- | -------------------------------------- |
| Main GRE shell          | `/home`                                        | Collection open (`greDashboard` state) |
| GRE modal dialog        | `/dashboard`                                   | Congrats CTAs, `open_gre_atlas()`      |
| Toolbar (outside shell) | `/home`, `/practice`, `/progress`, `/settings` | Quick links when not in GRE shell      |

The Qt menu bar exposes **GRE → Debug** (Deck Browser, Browse, etc.). There is no separate “Open GRE” menu item — the product opens into the GRE shell automatically.
