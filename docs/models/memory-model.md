# GRE Atlas memory model

The memory model estimates how well GRE flashcard knowledge is retained in long-term memory. It aggregates FSRS retrievability across the **GRE Atlas** deck and expresses the result as a 0–100 score with an uncertainty interval.

**Implementation:** `rslib/src/gre_atlas/readiness.rs` (`compute_memory_score`), fed by `rslib/src/stats/mastery.rs` (`compute_topic_mastery`).

**Model version (eval reports):** `fsrs`

---

## Inputs

| Input                  | Source                                 | Role                                                             |
| ---------------------- | -------------------------------------- | ---------------------------------------------------------------- |
| FSRS enabled           | Collection / deck config               | Required for retrievability; disabled FSRS blocks scoring        |
| Studied cards          | GRE deck search (`deck:"GRE Atlas"`)   | Cards with at least one review (`min_reviews` default 1)         |
| Overall retrievability | Per-card FSRS `current_retrievability` | Unweighted mean across studied cards (0–1)                       |
| Topic mastery entries  | GRE topic tags (`gre::…`)              | Per-topic studied counts and retrievability CIs                  |
| Coverage ratio         | GRE catalog (`compute_coverage`)       | Section-weighted share of catalog leaf topics with studied evidence |

Topic tags are mapped to the GRE catalog via `GreCatalog::nearest_topic_for_tag`. Retrievability uses the same FSRS parameters as live scheduling.

---

## Outputs

When abstention requirements are met, the model returns:

| Field                      | Meaning                                                                                        |
| -------------------------- | ---------------------------------------------------------------------------------------------- |
| `value`                    | Overall retrievability × 100 (0–100)                                                           |
| `value_low` / `value_high` | 95%-style interval from topic-level retrievability bounds, weighted by studied cards per topic |
| `coverage_ratio`           | Section-weighted catalog coverage (0–1), also used downstream by readiness                        |
| `studied_cards`            | Count of reviewed GRE cards in scope                                                           |
| `detail`                   | Human-readable summary (studied cards, coverage %, leaf topics with data)                      |
| `abstention_requirements`  | Structured checklist of evidence gates                                                         |

When abstaining, `value`, `value_low`, and `value_high` are omitted; `detail` and `abstain_reason` explain what is missing.

`TopicMasteryResponse` (the upstream aggregation) additionally exposes per-topic entries and a summary with `overall_avg_retrievability`, `mastered_cards`, and the same coverage and abstention fields.

---

## Confidence

Uncertainty on the point estimate comes from **topic-stratified retrievability intervals**: each topic’s `avg_retrievability_low` / `avg_retrievability_high` (from Welford online stats in topic mastery) are combined into a studied-card-weighted envelope for the overall score.

This reflects spread across topics, not a formal posterior on a single population parameter. Readiness reuses this interval width when building its projected-score band.

---

## Abstention rule

Memory abstains unless **all** requirements are met (`rslib/src/gre_atlas/abstention.rs`):

| Requirement       | Threshold                                |
| ----------------- | ---------------------------------------- |
| FSRS scheduling   | FSRS enabled on the collection           |
| Studied GRE cards | ≥ 50 cards with at least one review      |
| Topic coverage    | ≥ 50% section-weighted GRE catalog coverage |

If any gate fails, `sufficient_data = false` and the UI should show requirements and `next_step` guidance rather than a numeric score.

---

## Limitations

- **Deck-scoped:** Only cards matching the GRE deck search contribute; untagged or mis-tagged cards are invisible to the model.
- **FSRS-dependent:** Without FSRS, retrievability is undefined and the model abstains.
- **Coverage ≠ mastery:** High coverage means evidence exists across the catalog, not that every topic is well learned.
- **Tag granularity:** Tags roll up to catalog topics; coarse tags dilute per-topic signal.
- **No practice signal:** Memory reflects flashcard reviews only; GRE practice questions are handled by the performance model.

---

## Evaluation methodology

Offline calibration is read-only and documented in [`scripts/eval/README.md`](../../scripts/eval/README.md).

| Aspect                       | Rule                                                                                                                                                                            |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Held-out split**           | `revlog.id % 5 == 0`                                                                                                                                                            |
| **Minimum held-out reviews** | 5                                                                                                                                                                               |
| **Prediction**               | FSRS `historical_memory_states()` + `current_retrievability()` at the review interval, using deck FSRS parameters; history strictly before the held-out review on the same card |
| **Outcome**                  | 1 = Hard/Good/Easy (rating > 1), 0 = Again                                                                                                                                      |
| **Metrics**                  | Brier score, 10 reliability bins, calibration curve (predicted vs observed recall per bin), 95% normal-approximation CI on Brier when enough data                               |
| **Leakage safety**           | Held-out membership fixed at insert time; predictions never use the held-out review itself                                                                                      |

Regenerate the report:

```bash
just eval-gre-atlas /path/to/collection.anki2
```

Latency of production `topic_mastery` can be measured separately:

```bash
just bench-gre-atlas --collection /path/to/collection.anki2
```
