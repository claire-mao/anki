# GRE Atlas — evaluation instructions

All evaluation paths are **read-only**: they do not call `maintain_readiness_calibration()`, record new predictions, or mutate scheduling state.

Full methodology: [../../scripts/eval/README.md](../../scripts/eval/README.md).

## Prerequisites

```bash
just build          # or ./ninja pylib
```

## 1. Full evaluation report (primary artifact)

Generates readiness calibration, FSRS memory calibration, performance model eval, abstention/coverage stats, prediction distribution, and topic-priority ablation.

```bash
just eval-gre-atlas /path/to/collection.anki2
```

Or directly:

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_eval.py \
  --collection /path/to/collection.anki2 \
  --output-dir docs/gre-atlas-submission/results
```

### Outputs

All artifacts are written to `docs/gre-atlas-submission/results/`:

| File                  | Contents                                                                              |
| --------------------- | ------------------------------------------------------------------------------------- |
| `gre-atlas-eval.json` | Machine-readable report (calibration, coverage, prediction distribution, ablation, …) |
| `gre-atlas-eval.md`   | Human-readable summary                                                                |
| `performance-eval.md` | Standalone performance-model eval (held-out split, confusion matrix, Wilson CI)       |

### Report sections

| Section                 | Held-out rule                                              | Minimum data                       |
| ----------------------- | ---------------------------------------------------------- | ---------------------------------- |
| Readiness calibration   | `bl_readiness_prediction.id % 5 == 0`                      | 5 resolved outcomes                |
| Calibration bins        | 10-point bins on projected score (held-out, resolved only) | Same as calibration                |
| Prediction distribution | All logged predictions in `greatlas.db`                    | n/a (counts even when sparse)      |
| FSRS memory calibration | `revlog.id % 5 == 0`                                       | 5 held-out reviews (FSRS required) |
| Performance model       | `bl_performance_attempt.id % 5 == 0`                       | 5 test attempts                    |
| Topic-priority ablation | Counterfactual simulation                                  | 3+ eligible recommendations        |
| Abstention & coverage   | Current collection snapshot                                | n/a                                |

### Interpreting key metrics

- **Brier score** (readiness, held-out): lower is better; ≤ 0.08 is well calibrated in production thresholds.
- **Calibration bins**: compare predicted mean vs outcome mean per 10-point score band; large gaps indicate miscalibration.
- **Prediction distribution**: score histogram and confidence-level counts for all logged predictions; useful for spotting sparse or skewed logging before trusting calibration.
- **Held-out split summary** (`held_out_split` in JSON): documents the `id % 5 == 0` rule, outcome resolution, and leakage safety.

### Sparse collections

- Readiness/memory/performance metrics show **insufficient data** honestly when thresholds are not met.
- Prediction distribution still reports counts (often zero rows).
- Ablation includes a **synthetic reference scenario** (labeled `synthetic_reference`) for reproducible comparison even when live data is sparse.

### Determinism

Given the same `collection.anki2` + `greatlas.db` snapshot, reruns produce identical JSON/Markdown. The report timestamp is derived from the latest prediction row (not wall clock). The harness never writes prediction rows during report generation.

## 2. Production API benchmark

Times live GRE Atlas entry points (not instrumented in Rust):

```bash
# Synthetic labeled large collection (default 10,000 cards if no args)
just bench-gre-atlas

# Custom size
just bench-gre-atlas --synthetic-cards 50000

# Live collection
just bench-gre-atlas --collection /path/to/collection.anki2 --iterations 30 --warmup 3
```

### Outputs

| File                                                         | Metrics                              |
| ------------------------------------------------------------ | ------------------------------------ |
| `docs/gre-atlas-submission/results/gre-atlas-benchmark.json` | p50, p95, worst, mean per API        |
| `docs/gre-atlas-submission/results/gre-atlas-benchmark.md`   | Summary tables                       |
| `docs/gre-atlas-submission/results/gre-atlas-benchmark.csv`  | Same timings in CSV for spreadsheets |

APIs timed (production entry points):

| Benchmark ID            | Production API                                      |
| ----------------------- | --------------------------------------------------- |
| `topic_mastery`         | `Collection.topic_mastery(search=deck:"GRE Atlas")` |
| `dashboard_generation`  | `gre_atlas.get_dashboard()`                         |
| `readiness_calculation` | `gre_atlas.get_scores()`                            |
| `study_plan_generation` | `gre_atlas.get_study_plan()`                        |

Warmup iterations are excluded from p50/p95/worst statistics. Benchmark timestamps in JSON reflect run time (timing varies by machine); use the same collection and iteration settings when comparing locally.

## 3. AI gold-set evaluation

Read-only comparison of **keyword, BM25, and TF-IDF retrieval baselines** vs **catalog-aware AI retrieval** and the **full AI generation pipeline** (template variants + eval gate) on a bundled 50-question gold set. Does not require a collection path or API key.

```bash
just eval-gre-atlas-ai
```

Or directly:

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_ai_eval.py \
  --output-dir docs/gre-atlas-submission/results
```

### Outputs

| File                                                       | Contents                                                                                    |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.json` | Baseline vs AI accuracy/F1, held-out release gate, rejection pipeline, attribution metadata |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md`   | Human-readable summary                                                                      |

Full methodology, source attribution, and confidence threshold: [AI.md](./AI.md).

## 4. Unit tests (regression)

```bash
just test-rust    # rslib/gre_atlas/*, stats/mastery, eval modules
just test-py      # pylib/tests/test_gre_atlas.py, test_gre_atlas_benchmark.py
```

Key Rust modules with eval tests:

- `rslib/src/gre_atlas/eval.rs` — full report, prediction distribution, collection determinism
- `rslib/src/gre_atlas/calibration.rs` — Brier, held-out filter, calibration curve
- `rslib/src/gre_atlas/readiness.rs` — abstention, composite weights, projection
- `rslib/src/stats/mastery.rs` — TopicMastery aggregation
- `rslib/src/gre_atlas/memory_eval.rs`
- `rslib/src/gre_atlas/performance_eval.rs`
- `rslib/src/gre_atlas/ablation_eval.rs`
- `rslib/src/gre_atlas/questions/ai_gen.rs` — generation, confidence gate, source attribution
- `rslib/src/gre_atlas/ai_eval.rs` — gold-set baseline vs generation

## 5. Model documentation cross-check

Thresholds in eval reports should match:

| Model                                            | Doc                                                              |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| Memory abstention (FSRS, 50 cards, 50% coverage) | [../models/memory-model.md](../models/memory-model.md)           |
| Performance abstention (50 attempts)             | [../models/performance-model.md](../models/performance-model.md) |
| Readiness composite (45/45/10)                   | [../models/readiness-model.md](../models/readiness-model.md)     |
| AI generation confidence (0.55)                  | [AI.md](./AI.md), `MIN_GENERATION_CONFIDENCE` in `ai_gen.rs`     |
| AI eval gate (grounding 0.15, duplicate 0.85)    | [AI.md](./AI.md), `eval_pipeline.rs`                             |
| AI release gate (≥95% held-out accuracy)         | [AI.md](./AI.md), `DEFAULT_MIN_ACCURACY` in `ai_eval.rs`         |

## 6. What eval does _not_ measure

- End-to-end GRE exam score prediction accuracy on real test takers
- UI latency inside Qt (use benchmark harness for API timing)
- Mobile sync conflict resolution under load (see `mobile/mobile_bridge` parity tests)

## Reproducibility checklist

- [ ] Same `collection.anki2` + `greatlas.db` snapshot → identical `gre-atlas-eval.json`
- [ ] Eval harness never writes to `greatlas.db` prediction tables during report generation
- [ ] Held-out membership fixed at row insert (`id % 5 == 0`) for predictions, attempts, and revlog
- [ ] Benchmark artifacts committed only when intentionally snapshotting a run (`results/` is gitignored by default)
