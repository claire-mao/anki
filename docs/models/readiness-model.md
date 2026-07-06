# GRE Atlas readiness model

The readiness model combines memory, practice performance, and catalog coverage into a single **projected readiness index** (0–100). It answers: “Given current flashcard retention, practice accuracy, and topic coverage, how prepared is this learner for the GRE?”

**Implementation:** `rslib/src/gre_atlas/readiness.rs` (`compute_readiness_score`), with live calibration in `rslib/src/gre_atlas/calibration.rs`.

**Model version (eval reports):** `readiness_v1`

---

## Inputs

Readiness is computed from the **memory** and **performance** scores plus shared coverage:

| Input                     | Source                                       | Weight in composite |
| ------------------------- | -------------------------------------------- | ------------------- |
| Memory score (0–100)      | Memory model (`MemoryScore.value`)           | 45%                 |
| Performance score (0–100) | Performance model (`PerformanceScore.value`) | 45%                 |
| Coverage ratio (0–1)      | Memory model (`MemoryScore.coverage_ratio`)  | 10%                 |

Formula (when not abstaining):

```
projected = (0.45 × memory/100 + 0.45 × performance/100 + 0.10 × coverage) × 100
```

Coverage enters both as an abstention gate on memory and as an explicit 10% term in the composite, so breadth of catalog evidence affects readiness directly.

Live scoring also records **readiness predictions** to `greatlas.db` for later calibration (`maintain_readiness_calibration`), and attaches **calibration honesty** fields from held-out prediction history.

---

## Outputs

When all abstention requirements are met:

| Field                                                                          | Meaning                                                   |
| ------------------------------------------------------------------------------ | --------------------------------------------------------- |
| `projected_score`                                                              | Composite readiness index (0–100)                         |
| `projected_score_low` / `projected_score_high`                                 | Interval from combined memory and performance uncertainty |
| `confidence_level`                                                             | `high`, `medium`, or `low` (see below)                    |
| `coverage_ratio`                                                               | Same section-weighted catalog coverage as memory             |
| `evidence_summary`                                                             | Short text tying memory and performance evidence together |
| `last_updated_millis`                                                          | Computation timestamp                                     |
| `calibration_note` / `calibration_brier_score` / `calibration_well_calibrated` | Honest summary from held-out prediction calibration       |

When abstaining, `projected_score` and interval fields are omitted. `evidence_summary` still describes partial evidence; `abstention_requirements` lists unmet memory and performance gates.

If calibration history shows poor calibration (Brier above threshold on held-out predictions), live readiness may downgrade `confidence_level` to `low` even when the composite score is shown (`apply_calibration_honesty`).

---

## Confidence

**Interval:** `projected_score_low` / `projected_score_high` combine memory and performance interval widths in quadrature (half the root-sum-square of the two component widths), then clamp to 0–100.

**Level** (`confidence_level`):

| Level      | Conditions (all must hold for **high**)                                                          |
| ---------- | ------------------------------------------------------------------------------------------------ |
| **high**   | ≥ 400 studied cards, ≥ 50 practice attempts, ≥ 70% coverage, projected interval width ≤ 8 points |
| **medium** | Interval width ≤ 15 points (if not high)                                                         |
| **low**    | Wider interval, and/or forced low when calibration is insufficient or not well calibrated        |

Confidence describes **evidence quantity and interval tightness**, not a calibrated probability of passing the GRE.

---

## Abstention rule

Readiness abstains unless **every** memory **and** performance requirement is satisfied. It inherits the union of:

**Memory gates**

- FSRS enabled
- ≥ 50 studied GRE cards
- ≥ 50% section-weighted catalog coverage

**Performance gates**

- ≥ 20 practice attempts

Any unmet requirement appears in `abstention_requirements` with `met = false` and a `next_step` hint. `abstain_reason` concatenates human-readable labels for all unmet gates.

---

## Limitations

- **Composite, not causal:** Weights (45/45/10) are fixed; they do not adapt to individual learners.
- **Not an official GRE score:** Readiness is an internal index; `EstimatedGreScore` is a separate mapping with its own abstention rules.
- **Coverage double role:** The same coverage ratio gates memory and contributes to the composite; sparse catalogs cap readiness even with strong accuracy on covered topics.
- **Calibration lag:** Prediction/outcome pairs need time (≥ 3 days) or additional practice (≥ 3 attempts after prediction) before outcomes resolve; early users lack calibration feedback.
- **No section adaptivity:** One index for the whole GRE; section balance is indirect via topic coverage and tags.

---

## Evaluation methodology

Readiness calibration is evaluated read-only via `just eval-gre-atlas`. See [`scripts/eval/README.md`](../../scripts/eval/README.md).

| Aspect                           | Rule                                                                                                            |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Held-out split**               | `bl_readiness_prediction.id % 5 == 0`                                                                           |
| **Minimum held-out predictions** | 5 resolved outcomes                                                                                             |
| **Outcome resolution**           | Outcome recorded when prediction is ≥ 3 days old **or** learner has ≥ 3 practice attempts after prediction time |
| **Outcome score**                | Recomputed composite from observed memory, performance, and coverage at resolution time                         |
| **Metrics**                      | Brier score, mean absolute error, 10-point calibration bins on projected vs outcome (held-out only), 95% CIs    |
| **Well calibrated**              | Brier ≤ 0.08 on held-out pairs (moderate threshold 0.15)                                                        |
| **Leakage safety**               | Held-out id fixed at insert; eval harness does not call `maintain_readiness_calibration()`                      |

Related offline evaluations (same report, separate models):

- **Memory calibration** — FSRS recall vs revlog outcomes (`revlog.id % 5 == 0`)
- **Performance calibration** — topic-stratified attempt prediction (`bl_performance_attempt.id % 5 == 0`)
- **Topic-priority ablation** — counterfactual study-plan ordering (synthetic reference + collection scenarios)

```bash
just eval-gre-atlas /path/to/collection.anki2
just bench-gre-atlas --collection /path/to/collection.anki2
```
