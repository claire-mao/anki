# GRE Atlas performance model — evaluation methodology

This document describes how the **performance model** is evaluated for the GRE Atlas submission. It is distinct from [`../models/performance-model.md`](../models/performance-model.md), which documents live scoring in the product UI.

---

## What the performance model measures

The performance model estimates accuracy on **GRE-style practice questions** stored in `greatlas.db` (`bl_performance_attempt`). Each attempt records a topic label and a binary outcome (correct / incorrect).

Live scoring (`compute_performance_score` in `rslib/src/gre_atlas/readiness.rs`) reports **global** accuracy over all attempts with a Wilson confidence interval and abstains until at least 50 attempts exist.

Held-out evaluation uses a **separate train-fit model** for calibration metrics. It does not change live scoring.

---

## Assumptions

| Assumption                                          | Rationale                                                                 |
| --------------------------------------------------- | ------------------------------------------------------------------------- |
| Practice attempts are independent observations      | Each row is one answered question; attempts do not mutate Anki revlog     |
| Topic labels are stable                             | Stratification uses the `topic` column on each attempt                    |
| Binary outcomes are sufficient                      | Partial credit and confidence ratings are stored but not used in accuracy |
| Auto-increment attempt ids are monotonic            | Held-out membership is `id % 5 == 0` at insert time                       |
| Train-fit topic rates generalize to held-out topics | Unseen topics fall back to global training accuracy                       |

---

## Train/test split

The split is **deterministic and fixed at insert time**:

| Split                     | Rule                                             |
| ------------------------- | ------------------------------------------------ |
| **Training**              | `bl_performance_attempt.id % 5 != 0`             |
| **Test (held-out)**       | `bl_performance_attempt.id % 5 == 0`             |
| **Minimum test attempts** | 5 (below this, metrics report insufficient data) |

Implementation: `rslib/src/gre_atlas/performance_eval.rs` (`is_held_out_attempt`).

Training rows fit a topic-stratified empirical accuracy model. Test rows are scored against that model only; training rows never appear in test metric denominators.

---

## Train-fit model (not live scoring)

On training attempts only:

1. Compute per-topic empirical accuracy (correct / total per topic).
2. For a held-out attempt, estimate P(correct) from its topic rate, or global training accuracy if the topic was unseen in training.
3. Predict **correct** when estimated P(correct) ≥ 0.5.

This is the same practice-question system used in production; evaluation reuses attempt rows read-only from `greatlas.db`.

---

## Reported metrics (held-out only)

| Metric                      | Definition                                                     |
| --------------------------- | -------------------------------------------------------------- |
| **Held-out accuracy**       | Observed correct rate on test attempts (%)                     |
| **Wilson 95% CI**           | Wilson score interval on test accuracy (z = 1.96)              |
| **Precision / recall / F1** | Binary prediction quality; positive class = correct answer     |
| **Prediction accuracy**     | Share of test attempts where binary prediction matches outcome |
| **Confusion matrix**        | TP / FP / TN / FN from binary predictions vs actual outcomes   |

All metrics are computed in `rslib/src/gre_atlas/performance_eval.rs` and exported via the eval harness.

---

## Why readiness is evaluated separately

Readiness is a **composite** of memory, performance, and coverage (`compute_readiness_score`). Its calibration eval uses `bl_readiness_prediction` rows with delayed outcome resolution (3 days or 3 post-prediction practice attempts).

Performance evaluation answers: _“Given past practice on similar topics, can we predict whether the learner will get the next held-out question right?”_

Readiness calibration answers: _“Did the projected readiness score match later observed readiness?”_

These are different targets, different tables, and different held-out splits on different row types. Combining them would conflate flashcard retention with practice-question accuracy.

---

## Why memory ≠ performance

| Dimension           | Memory model                                           | Performance model                    |
| ------------------- | ------------------------------------------------------ | ------------------------------------ |
| **Data source**     | Anki revlog + FSRS on GRE deck cards                   | `greatlas.db` practice attempts      |
| **Signal**          | Predicted vs observed card recall                      | Topic-stratified practice accuracy   |
| **Abstention gate** | FSRS enabled, ≥50 studied cards, ≥50% catalog coverage | ≥50 practice attempts                |
| **Eval split**      | `revlog.id % 5 == 0`                                   | `bl_performance_attempt.id % 5 == 0` |

A learner can have strong FSRS retrievability on vocabulary cards yet miss novel quant questions, or vice versa. Readiness combines both only when each component has sufficient evidence; eval reports them independently.

---

## Limitations

- **Practice bank scope:** Accuracy is over embedded GRE questions the learner has attempted, not a full official GRE form.
- **No temporal weighting:** All attempts weigh equally in both live scoring and eval.
- **Simple train model:** Topic-stratified empirical rates do not model difficulty, response time, or session effects.
- **Sparse topics:** Topics with few training attempts produce noisy rate estimates; global fallback may dominate.
- **Id-based split:** `id % 5` is convenient and leakage-safe but not a random sample of learners or items.

When data is insufficient, the report states this honestly rather than fabricating metrics.

---

## How to regenerate results

```bash
just eval-gre-atlas /path/to/collection.anki2
```

Outputs:

| File                                                    | Contents                               |
| ------------------------------------------------------- | -------------------------------------- |
| `docs/gre-atlas-submission/results/performance-eval.md` | Standalone performance held-out report |
| `docs/gre-atlas-submission/results/gre-atlas-eval.md`   | Full GRE Atlas eval (all models)       |
| `docs/gre-atlas-submission/results/gre-atlas-eval.json` | Machine-readable full report           |

The harness is read-only: it does not record attempts, resolve readiness outcomes, or mutate scheduling state.

Full eval methodology: [`../../scripts/eval/README.md`](../../scripts/eval/README.md).
