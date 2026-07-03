# GRE Atlas — grading checklist

Maps **project requirements** to **verification steps** and **evidence locations**. Check each box during review.

Legend: **Req ID** groups related deliverables from the product architecture, Speedrun constraints, and evaluation extensions.

---

## A. Product architecture (Phase 1 core)

| ✓ | Req ID | Requirement                                           | How to verify                                                      | Evidence                                                                              |
| - | ------ | ----------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| ☐ | A1     | GRE product shell distinct from Anki reviewer         | `just run` → GRE shell at `/home`; separate Svelte layout          | `ts/routes/(gre)/`, `qt/aqt/gre_dashboard.py`                                         |
| ☐ | A2     | Practice workflow with GRE questions                  | Practice page loads MCQs; submit records attempt                   | `ts/routes/(gre)/practice/`, `RecordAttempt` RPC                                      |
| ☐ | A3     | Practice does **not** write revlog / FSRS state       | Code review + test: revlog count unchanged after attempt           | `pylib/tests/test_gre_atlas.py::test_gre_atlas_record_attempt_does_not_modify_revlog` |
| ☐ | A4     | `greatlas.db` sidecar for performance data            | Attempts in `{profile}/greatlas.db`, not collection schema         | `rslib/src/gre_atlas/storage/`                                                        |
| ☐ | A5     | Three separate scores: Memory, Performance, Readiness | Readiness page shows three cards; different inputs                 | `GetScores`, `docs/models/*.md`                                                       |
| ☐ | A6     | BrainLiftService protobuf API                         | 18 RPCs defined and implemented                                    | `proto/anki/brainlift.proto`, `rslib/src/gre_atlas/service.rs`                        |
| ☐ | A7     | Dashboard with GRE state                              | `/home` and `/dashboard` load dashboard RPC data                   | `GetDashboard`, `(gre)/home`, `(gre)/dashboard`                                       |
| ☐ | A8     | Qt integration without reviewer diffs                 | GRE review uses standard reviewer; no scheduler edits for practice | `qt/aqt/reviewer.py`, `rslib/scheduler/` unchanged by practice                        |
| ☐ | A9     | Congrats → GRE CTAs                                   | Finish non-GRE review → congrats Practice/Dashboard buttons        | `ts/routes/congrats/CongratsPage.svelte`, `qt/aqt/overview.py`                        |
| ☐ | A10    | `just check` green                                    | Run `just check` on submission branch                              | CI / local log                                                                        |

---

## B. Memory engine (TopicMastery / FSRS)

| ✓ | Req ID | Requirement                                | How to verify                                              | Evidence                                                                           |
| - | ------ | ------------------------------------------ | ---------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| ☐ | B1     | TopicMastery in Rust (not Python hot path) | `cargo test topic_mastery` / `just test-rust`              | `rslib/src/stats/mastery.rs`                                                       |
| ☐ | B2     | FSRS retrievability per GRE topic tag      | Progress page + `topic_mastery()` response                 | `StatsService.TopicMastery`, `(gre)/progress`                                      |
| ☐ | B3     | Exam-weighted catalog coverage             | Coverage % matches GRE catalog weights                     | `rslib/src/gre_atlas/domain/coverage.rs`                                           |
| ☐ | B4     | Memory abstention gates                    | Sparse profile → no memory score; requirements listed      | 200 cards, 50% coverage, FSRS — `rslib/src/gre_atlas/abstention.rs`                |
| ☐ | B5     | Performance at scale (Speedrun)            | `just bench-gre-atlas` on large collection; p95 documented | `scripts/eval/gre_atlas_benchmark.py`, `docs/gre-atlas-topic-mastery-rust-note.md` |

---

## C. Performance & readiness models

| ✓ | Req ID | Requirement                                              | How to verify                                           | Evidence                                                        |
| - | ------ | -------------------------------------------------------- | ------------------------------------------------------- | --------------------------------------------------------------- |
| ☐ | C1     | Performance from practice attempts only                  | Global accuracy after attempts; Wilson CI when unlocked | `compute_performance_score`, `docs/models/performance-model.md` |
| ☐ | C2     | Performance abstention (<20 attempts)                    | Fresh profile → performance abstains                    | `MIN_PERFORMANCE_ATTEMPTS = 20`                                 |
| ☐ | C3     | Readiness composite formula                              | 45% memory + 45% performance + 10% coverage             | `readiness.rs`, `docs/models/readiness-model.md`                |
| ☐ | C4     | Readiness abstention inherits memory + performance gates | Unmet requirements listed on readiness page             | `GetReadinessCalibration`, `(gre)/readiness`                    |
| ☐ | C5     | Confidence levels (high/medium/low)                      | Sufficient profile shows confidence on readiness        | `readiness.rs::confidence_level`                                |

---

## D. Study plan & recommendations

| ✓ | Req ID | Requirement                  | How to verify                                        | Evidence                                            |
| - | ------ | ---------------------------- | ---------------------------------------------------- | --------------------------------------------------- |
| ☐ | D1     | Ranked study recommendations | Study plan page sorted by priority                   | `GetStudyPlan`, `rslib/src/gre_atlas/study_plan.rs` |
| ☐ | D2     | Daily focus topics (top 3)   | Daily plan includes focus topic tasks                | `DAILY_FOCUS_TOPIC_COUNT = 3`                       |
| ☐ | D3     | Factor-based explanations    | Each recommendation has factors + explanation string | `(gre)/study-plan`, proto `StudyPlanRecommendation` |
| ☐ | D4     | Topic detail drill-down      | `/topics/[topicId]` loads                            | `GetTopicDetails`, `(gre)/topics/`                  |

---

## E. Calibration & evaluation (read-only)

| ✓ | Req ID | Requirement                          | How to verify                                            | Evidence                                                                                      |
| - | ------ | ------------------------------------ | -------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| ☐ | E1     | Readiness prediction logging         | `greatlas.db` prediction rows after sufficient readiness | `rslib/src/gre_atlas/calibration.rs`                                                          |
| ☐ | E2     | Held-out eval split (`id % 5 == 0`)  | Eval report documents rule; deterministic rerun          | `scripts/eval/README.md`, `eval.rs`                                                           |
| ☐ | E3     | Brier + calibration bins (readiness) | `just eval-gre-atlas` → calibration section              | `calibration.rs`, eval report                                                                 |
| ☐ | E4     | FSRS memory calibration eval         | Eval report → FSRS memory section (requires FSRS)        | `memory_eval.rs`                                                                              |
| ☐ | E5     | Performance model held-out eval      | Eval report → precision/recall/F1 on test attempts       | `performance_eval.rs`                                                                         |
| ☐ | E6     | Topic-priority ablation              | Eval report → GRE Atlas vs random vs vanilla             | `ablation_eval.rs`                                                                            |
| ☐ | E7     | No fabricated eval statistics        | Sparse data → `insufficient` / `n/a`; synthetic labeled  | ablation synthetic_reference, eval assessments                                                |
| ☐ | E8     | One-command eval                     | `just eval-gre-atlas /path/to/collection.anki2`          | `justfile`, `scripts/eval/gre_atlas_eval.py`; outputs in `docs/gre-atlas-submission/results/` |

---

## F. Benchmark harness

| ✓ | Req ID | Requirement                                              | How to verify                                              | Evidence                              |
| - | ------ | -------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------- |
| ☐ | F1     | Benchmark TopicMastery, dashboard, readiness, study plan | `just bench-gre-atlas`                                     | `scripts/eval/gre_atlas_benchmark.py` |
| ☐ | F2     | Report p50, p95, worst case                              | `gre-atlas-benchmark.md` tables                            | eval output                           |
| ☐ | F3     | Large collection support                                 | `--collection` or `--synthetic-cards`                      | benchmark script                      |
| ☐ | F4     | Markdown export                                          | `docs/gre-atlas-submission/results/gre-atlas-benchmark.md` | output dir                            |

---

## G. Tests & documentation

| ✓ | Req ID | Requirement                                             | How to verify                                            | Evidence                                                       |
| - | ------ | ------------------------------------------------------- | -------------------------------------------------------- | -------------------------------------------------------------- |
| ☐ | G1     | Rust unit tests for GRE Atlas                           | `just test-rust`                                         | `rslib/src/gre_atlas/**` test modules                          |
| ☐ | G2     | Python smoke tests                                      | `just test-py`                                           | `pylib/tests/test_gre_atlas.py`                                |
| ☐ | G3     | Model documentation (inputs, outputs, abstention, eval) | Read model docs (~1 page each)                           | `docs/models/`                                                 |
| ☐ | G4     | Build & eval instructions                               | Follow submission BUILD + EVALUATION docs                | `docs/gre-atlas-submission/`                                   |
| ☐ | G5     | Architecture diagram                                    | Mermaid in ARCHITECTURE.md renders                       | `docs/gre-atlas-submission/ARCHITECTURE.md`                    |
| ☐ | G6     | Feature index & screenshots                             | FEATURE-INDEX.md maps features; SCREENSHOTS.md checklist | `docs/gre-atlas-submission/FEATURE-INDEX.md`, `SCREENSHOTS.md` |

---

## I. AI question generation (Speedrun extension)

| ✓ | Req ID | Requirement                   | How to verify                                                             | Evidence                                            |
| - | ------ | ----------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------- |
| ☐ | I1     | Named source with attribution | Generated rows store `source_name`, `source_section`, `generated_at_secs` | `questions/source.rs`, schema v4, [AI.md](./AI.md)  |
| ☐ | I2     | Question generation RPC       | `GenerateQuestion(topic_id, persist)` returns MCQ + confidence            | `proto/anki/brainlift.proto`, `questions/ai_gen.rs` |
| ☐ | I3     | Low-confidence rejection      | Confidence &lt; 0.55 → rejected (no persist)                              | `MIN_GENERATION_CONFIDENCE` in `ai_gen.rs`          |
| ☐ | I4     | 50-question gold eval set     | `"verified": true` entries in bundled JSON                                | `questions/gold_eval_set.json`                      |
| ☐ | I5     | Baseline vs generation eval   | `just eval-gre-atlas-ai` → keyword baseline + template metrics            | `ai_eval.rs`, `results/gre-atlas-ai-eval.{json,md}` |
| ☐ | I6     | No chat / RAG / vector DB     | Docs state template-based offline pipeline                                | [AI.md](./AI.md)                                    |

---

## H. Mobile companion (bonus / Phase 4)

| ✓ | Req ID | Requirement               | How to verify                    | Evidence                                            |
| - | ------ | ------------------------- | -------------------------------- | --------------------------------------------------- |
| ☐ | H1     | iOS companion scaffold    | Build per `mobile/ios/README.md` | `mobile/ios/`                                       |
| ☐ | H2     | GRE Atlas sync RPCs       | Pull/push/status implemented     | proto sync RPCs, `rslib/src/gre_atlas/sync.rs`      |
| ☐ | H3     | Mobile parity tests       | `cargo test -p mobile_bridge`    | `mobile/mobile_bridge/src/parity.rs`                |
| ☐ | H4     | Demo collection bootstrap | `PrepareDemoCollection` RPC      | `rslib/src/gre_atlas/demo.rs`, `mobile/ios/DEMO.md` |

---

## Scoring guide (suggested)

| Grade band                  | Expectation                                                                                            |
| --------------------------- | ------------------------------------------------------------------------------------------------------ |
| **Full credit (A)**         | All A–I items pass; eval + ablation + AI eval reproducible; abstention honest; FSRS isolation verified |
| **Strong (B+)**             | A–D pass; E eval runs with partial data; tests green; minor doc gaps only                              |
| **Partial (B)**             | Core shell + practice + scores work; abstention present; eval incomplete                               |
| **Incomplete (C or below)** | Practice writes revlog; readiness shows numbers without gates; build fails; no tests                   |

---

## Reviewer quick commands

```bash
just check
just run
just test-rust
just test-py
just eval-gre-atlas /path/to/collection.anki2
just bench-gre-atlas --synthetic-cards 5000
just eval-gre-atlas-ai
```

Submission index: [README.md](./README.md)
