# GRE Atlas performance model

The performance model estimates accuracy on **GRE-style practice questions** stored in `greatlas.db`. It summarizes all recorded attempts into a 0–100 accuracy score with a Wilson confidence interval.

**Implementation:** `rslib/src/gre_atlas/readiness.rs` (`compute_performance_score`), with attempts aggregated in `rslib/src/gre_atlas/storage/`.

**Model version (eval reports):** `performance_v1`

---

## Inputs

| Input             | Source                                    | Role                                                            |
| ----------------- | ----------------------------------------- | --------------------------------------------------------------- |
| Practice attempts | `bl_performance_attempt` in `greatlas.db` | Each row: topic, correct/incorrect, timestamp, optional session |
| Correct count     | Sum of `correct = true`                   | Numerator for accuracy                                          |
| Attempt count     | Total rows                                | Denominator and abstention gate                                 |

Attempts are **independent of Anki revlog**: answering practice questions does not mutate card scheduling history. Topic labels on attempts are used for dashboard insights and eval stratification, but the live score uses **global** accuracy (correct / total).

---

## Outputs

When abstention requirements are met:

| Field                      | Meaning                                                           |
| -------------------------- | ----------------------------------------------------------------- |
| `value`                    | Accuracy × 100 (0–100)                                            |
| `value_low` / `value_high` | Wilson score 95% interval on accuracy (z = 1.96), scaled to 0–100 |
| `attempt_count`            | Total practice attempts logged                                    |
| `detail`                   | Human-readable `correct/total` summary                            |
| `abstention_requirements`  | Structured evidence checklist                                     |

When abstaining, `value` and interval fields are omitted; `detail` and `abstain_reason` describe the shortfall.

Downstream consumers (readiness, dashboard, study plan) treat performance as a single collection-level accuracy, not per-topic scores in the live path.

---

## Confidence

The reported interval is a **Wilson score interval** at 95% (same formula as `wilson_ci` in `readiness.rs`). It widens with small sample sizes and narrows as attempt count grows.

There is no separate “confidence level” string on `PerformanceScore`; readiness combines this interval with the memory interval when projecting readiness bounds.

---

## Abstention rule

Performance abstains unless:

| Requirement           | Threshold                             |
| --------------------- | ------------------------------------- |
| GRE practice attempts | ≥ 20 scored attempts in `greatlas.db` |

Below the minimum, `sufficient_data = false` and the model returns guidance to continue practicing rather than a numeric accuracy.

Readiness inherits this gate: it cannot project a score until performance (and memory) both unlock.

---

## Limitations

- **Practice-only:** Does not infer ability from flashcard reviews or external tests.
- **Global accuracy:** Live score ignores topic breakdown; weak areas are surfaced elsewhere (dashboard topic insights, study plan), not in the headline percentage.
- **Question bank scope:** Accuracy is over the embedded GRE question set the learner has attempted, not a full official GRE form.
- **No time decay:** All attempts weigh equally; recent mistakes are not weighted more heavily in the score itself.
- **Binary outcomes:** Partial credit and confidence ratings are stored but do not change the accuracy numerator.

---

## Evaluation methodology

Held-out evaluation uses a separate **topic-stratified prediction model** for calibration metrics (not the live global accuracy display). Details: [`scripts/eval/README.md`](../../scripts/eval/README.md).

| Aspect                    | Rule                                                                                                             |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| **Train split**           | `bl_performance_attempt.id % 5 != 0`                                                                             |
| **Test split**            | `bl_performance_attempt.id % 5 == 0`                                                                             |
| **Minimum test attempts** | 5                                                                                                                |
| **Train model**           | Per-topic empirical accuracy on training rows; global training accuracy for unseen topics                        |
| **Prediction rule**       | Predict **correct** when estimated P(correct) ≥ 0.5                                                              |
| **Test metrics**          | Held-out accuracy, Wilson 95% CI, precision, recall, F1, prediction accuracy, confusion matrix                   |
| **Leakage safety**        | Held-out membership fixed at insert time from auto-increment id; training rows never appear in test denominators |

The eval harness is read-only:

```bash
just eval-gre-atlas /path/to/collection.anki2
```

Production API timing:

```bash
just bench-gre-atlas --collection /path/to/collection.anki2
```

(`get_scores()` exercises readiness + performance together; practice-only latency is dominated by the shared signal load path.)
