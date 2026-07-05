# GRE Atlas evaluation

Reproducible, read-only evaluation of GRE Atlas readiness calibration and related metrics.

## What this measures

| Metric                               | Source                                                                           |
| ------------------------------------ | -------------------------------------------------------------------------------- |
| Brier score                          | Held-out readiness predictions vs later observed outcomes                        |
| Calibration bins                     | 10-point bins on projected score (held-out, resolved rows only)                  |
| Prediction distribution              | Score histogram and confidence-level counts for all logged predictions           |
| Confidence intervals                 | 95% normal-approximation CIs (z = 1.96) on Brier, MAE, and bin outcome means     |
| Abstention rate                      | Share of memory / performance / readiness score components currently abstaining  |
| Coverage statistics                  | GRE catalog leaf coverage (unweighted and exam-weighted), by section             |
| **Performance accuracy**             | Held-out practice attempt correct rate (never computed on training rows)         |
| **Performance Wilson CI**            | 95% Wilson score interval on held-out accuracy                                   |
| **Precision / recall / F1**          | Binary prediction quality on held-out attempts (positive class = correct answer) |
| **Memory Brier score**               | Held-out FSRS predicted recall vs observed recall (Again vs pass)                |
| **Memory reliability bins**          | 10-bin calibration on predicted recall probability (held-out reviews only)       |
| **Memory calibration curve**         | Mean predicted vs mean observed recall per bin                                   |
| **Ablation: expected learning gain** | Sum of production `priority_score` for simulated daily focus topics              |
| **Ablation: topic coverage gain**    | Delta in GRE weighted catalog coverage after simulated focus session             |
| **Ablation: readiness improvement**  | Delta in projected readiness after simulated focus session                       |

Readiness calibration math reuses `rslib/src/gre_atlas/calibration.rs`. FSRS memory evaluation reuses `historical_memory_states()` / `current_retrievability()` in `rslib/src/gre_atlas/memory_eval.rs`. Performance evaluation uses `rslib/src/gre_atlas/performance_eval.rs`. All run read-only via `rslib/src/gre_atlas/eval.rs`.

## Held-out split

- **Rule:** `bl_readiness_prediction.id % 5 == 0`
- **Minimum held-out rows:** 5 resolved outcomes (same as live calibration)
- **Training rows (live maintenance only):** `id % 5 != 0`

Outcome resolution matches production: an outcome is recorded when the prediction is at least **3 days** old **or** the learner has at least **3** practice attempts after the prediction.

## Why the split is leakage-safe

1. Held-out membership is fixed at **insert time** from the auto-increment row id, before any outcome exists.
2. Outcomes are observed later and never influence which rows are held out.
3. Calibration metrics are computed **only** on held-out rows with resolved outcomes.
4. This eval harness **does not** call `maintain_readiness_calibration()` — it reads `greatlas.db` and recomputes metrics without recording predictions or resolving outcomes.

Given the same `greatlas.db` snapshot, reruns are deterministic.

## Performance held-out split

- **Train rule:** `bl_performance_attempt.id % 5 != 0`
- **Test rule:** `bl_performance_attempt.id % 5 == 0`
- **Minimum test attempts:** 5

### Model (train only)

Topic-stratified empirical accuracy from training attempts, with global training accuracy as fallback for unseen topics. Each held-out attempt gets a binary prediction: predict **correct** when estimated P(correct) ≥ 0.5.

### Test metrics (held-out only)

- **Accuracy** — observed correct rate on test attempts (%)
- **Wilson 95% CI** — on test accuracy (z = 1.96, same formula as live `PerformanceScore`)
- **Precision / recall / F1** — positive class is a correct answer; predictions come from the train-fit model only
- **Prediction accuracy** — share of test attempts where the binary prediction matches the outcome

Training attempts are never included in test denominators. Held-out membership is fixed at insert time from the auto-increment attempt id.

## FSRS memory held-out split

- **Test rule:** `revlog.id % 5 == 0`
- **Minimum held-out reviews:** 5
- **Prediction:** FSRS `historical_memory_states()` + `current_retrievability()` at the review interval, using deck FSRS parameters
- **Outcome:** 1 = Hard/Good/Easy, 0 = Again
- **Metrics:** Brier score, 10 reliability bins, calibration curve (predicted vs observed recall per bin)

Predictions use only review history strictly before the held-out review on the same card. Requires FSRS enabled on the collection.

## Topic-priority ablation

Compares three daily focus-topic ordering policies on the **same eligible recommendation pool** (production `priority_score > 0` topics):

| Policy                     | Ordering                                                                                |
| -------------------------- | --------------------------------------------------------------------------------------- |
| **GRE Atlas priority (A)** | Production ranking: `priority_score` desc, then `exam_weight` desc, then `topic_id` asc |
| **Random topic order (B)** | Deterministic shuffle (`StdRng` seed 42), then take top 3 focus topics                  |
| **Vanilla Anki order (C)** | Lexicographic `topic_id` among eligible topics (no GRE Atlas topic-priority layer)      |

Each policy selects **3** focus topics (same as `DAILY_FOCUS_TOPIC_COUNT`). A documented one-session simulation estimates:

- **Expected learning gain** — sum of `priority_score` for selected topics
- **Topic coverage gain** — delta in GRE weighted catalog coverage after simulating coverage-gap closes
- **Readiness improvement** — delta in projected readiness (`n/a` when readiness abstains)

The report includes:

1. **Collection** results when the GRE deck has at least three eligible recommendations (live data).
2. **Synthetic reference scenario** — hand-authored mastery/practice inputs, clearly labeled `synthetic_reference`, for a reproducible baseline comparison.

Implementation: `rslib/src/gre_atlas/ablation_eval.rs`.

## GRE Atlas benchmark harness

Times production GRE Atlas APIs via pylib without modifying Rust code paths:

| Benchmark             | Production API                                      |
| --------------------- | --------------------------------------------------- |
| TopicMastery          | `Collection.topic_mastery(search=deck:"GRE Atlas")` |
| Dashboard generation  | `gre_atlas.get_dashboard()`                         |
| Readiness calculation | `gre_atlas.get_scores()`                            |
| Study plan generation | `gre_atlas.get_study_plan()`                        |

Each benchmark reports **p50**, **p95**, and **worst case** latency (milliseconds) after configurable warmup iterations.

### How to run

From the repo root, with pylib built:

```bash
# Large live collection
just bench-gre-atlas --collection /path/to/collection.anki2 --iterations 30 --warmup 3

# Labeled synthetic large collection (default 10,000 cards when no args)
just bench-gre-atlas --synthetic-cards 50000

# Default synthetic run (10,000 cards)
just bench-gre-atlas
```

Or directly:

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_benchmark.py \
  --collection /path/to/collection.anki2 \
  --iterations 30 \
  --warmup 3 \
  --output-dir docs/gre-atlas-submission/results
```

### Outputs

Default output directory (`just bench-gre-atlas` and `--output-dir docs/gre-atlas-submission/results`):

- `docs/gre-atlas-submission/results/gre-atlas-benchmark.json` — machine-readable timings
- `docs/gre-atlas-submission/results/gre-atlas-benchmark.md` — human-readable summary
- `docs/gre-atlas-submission/results/gre-atlas-benchmark.csv` — same timings in CSV (collection, benchmark id/label, p50/p95/worst/mean/min)

Synthetic collections are generated in a temporary directory and labeled `synthetic_reference` in the report.

## How to rerun

From the repo root, with pylib built (`just build` or `just check`):

```bash
just eval-gre-atlas /path/to/collection.anki2
```

Or directly (after `just build` or `./ninja pylib`):

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_eval.py \
  --collection /path/to/collection.anki2 \
  --output-dir docs/gre-atlas-submission/results
```

### Outputs

Default output directory (`just eval-gre-atlas` and `--output-dir docs/gre-atlas-submission/results`):

- `docs/gre-atlas-submission/results/gre-atlas-eval.json` — machine-readable report
- `docs/gre-atlas-submission/results/gre-atlas-eval.md` — human-readable summary
- `docs/gre-atlas-submission/results/performance-eval.md` — standalone performance held-out eval

Generated files are local artifacts; commit them only if you intentionally want to snapshot a benchmark run.

## AI question-generation release gate

Read-only evaluation on the bundled **held-out gold set** (`rslib/src/gre_atlas/questions/gold_eval_set.json`). Gold labels are never fed into generation — only each question's topic id is used to pick a candidate.

```bash
just eval-gre-atlas-ai
```

Or directly:

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_ai_eval.py \
  --output-dir docs/gre-atlas-submission/results
```

### Metrics (held-out only)

| Metric                | Definition                                                                                           |
| --------------------- | ---------------------------------------------------------------------------------------------------- |
| **Accuracy**          | Share of gold topics producing a candidate that passes the confidence cutoff and four-rule eval gate |
| **Wrong-answer rate** | Share rejected for hallucination (marked answer not among choices / not derivable)                   |
| **Acceptance cutoff** | Minimum generation confidence before the eval gate runs (default `0.55`)                             |

### Configurable thresholds

Override via CLI flags or environment variables (CLI wins when both are set):

| Variable / flag                                                       | Default | Purpose                       |
| --------------------------------------------------------------------- | ------- | ----------------------------- |
| `GRE_ATLAS_AI_EVAL_MIN_ACCURACY` / `--min-accuracy`                   | `0.95`  | Minimum held-out accuracy     |
| `GRE_ATLAS_AI_EVAL_MAX_WRONG_ANSWER_RATE` / `--max-wrong-answer-rate` | `0.0`   | Maximum wrong-answer rate     |
| `GRE_ATLAS_AI_EVAL_ACCEPTANCE_CUTOFF` / `--acceptance-cutoff`         | `0.55`  | Minimum generation confidence |

### Baseline comparison (stem-only retrieval)

The report also compares **keyword**, **BM25**, and **TF-IDF vector** retrieval baselines against catalog-aware **AI retrieval** and the full **AI generation pipeline** on the same 50 gold questions (query = stem only; keywords withheld).

| Metric                      | Definition                                                              |
| --------------------------- | ----------------------------------------------------------------------- |
| **Accuracy**                | Share of questions where the predicted GRE topic matches the gold label |
| **Precision / recall / F1** | Macro-averaged one-vs-rest topic classification                         |
| **Failure rate**            | Share of queries where retrieval returned no confident match            |
| **Keyword recall**          | Mean overlap between gold keywords and retrieved section metadata       |

Implementation: `rslib/src/gre_atlas/questions/retrieval.rs`.

### Pass / fail

The harness writes `gre-atlas-ai-eval.{json,md}` and exits **non-zero** when the release gate fails (failed models are rejected). CI runs this step after `just build`. Use `--allow-fail` to write the report without suppressing the non-zero exit code.

### Outputs

- `docs/gre-atlas-submission/results/gre-atlas-ai-eval.json` — machine-readable report (includes `held_out_quality`, `verdict`)
- `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md` — human-readable summary with release verdict

Implementation: `rslib/src/gre_atlas/ai_eval.rs`, `scripts/eval/gre_atlas_ai_eval.py`.

## Tests

Rust unit tests in `rslib/src/gre_atlas/eval.rs`, `rslib/src/gre_atlas/ablation_eval.rs`, `rslib/src/gre_atlas/calibration.rs`, `rslib/src/gre_atlas/memory_eval.rs`, and `rslib/src/gre_atlas/performance_eval.rs` cover deterministic recomputation and report formatting. Run:

```bash
just test-rust
```
