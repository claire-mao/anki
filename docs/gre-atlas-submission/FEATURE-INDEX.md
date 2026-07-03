# GRE Atlas — feature index

Maps each major GRE Atlas capability to Rust source, tests, UI, and documentation.

## Scores & abstention

| Feature                                                 | Rust                                            | Tests                       | UI / RPC                                                  | Docs                                                                                                                                                     |
| ------------------------------------------------------- | ----------------------------------------------- | --------------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Memory / Performance / Readiness scores                 | `rslib/src/gre_atlas/scores.rs`, `readiness.rs` | `scores.rs`, `readiness.rs` | `GetScores`, `(gre)/readiness`, `(gre)/progress`          | [memory-model.md](../models/memory-model.md), [performance-model.md](../models/performance-model.md), [readiness-model.md](../models/readiness-model.md) |
| Abstention gates (200 cards, 50% coverage, 20 attempts) | `rslib/src/gre_atlas/abstention.rs`             | `abstention.rs`             | `(gre)/ui/GreAbstentionChecklist.svelte`                  | [ARCHITECTURE.md](./ARCHITECTURE.md#score-pipeline), model docs                                                                                          |
| Estimated GRE score                                     | `rslib/src/gre_atlas/estimated_gre.rs`          | `estimated_gre.rs`          | `GetScores`, `(gre)/summaries/EstimatedGreSummary.svelte` | [readiness-model.md](../models/readiness-model.md)                                                                                                       |

## Readiness & calibration

| Feature                                | Rust                                 | Tests            | UI / RPC                                     | Docs                                                                                     |
| -------------------------------------- | ------------------------------------ | ---------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Readiness composite (45/45/10)         | `rslib/src/gre_atlas/readiness.rs`   | `readiness.rs`   | `GetReadinessCalibration`, `(gre)/readiness` | [readiness-model.md](../models/readiness-model.md)                                       |
| Prediction logging & Brier calibration | `rslib/src/gre_atlas/calibration.rs` | `calibration.rs` | `(gre)/ui/GreCalibrationPanel.svelte`        | [EVALUATION.md](./EVALUATION.md), [scripts/eval/README.md](../../scripts/eval/README.md) |
| Confidence levels (high/medium/low)    | `readiness.rs::confidence_level`     | `readiness.rs`   | `(gre)/ui/GreConfidenceIndicator.svelte`     | [readiness-model.md](../models/readiness-model.md)                                       |

## Memory signal (TopicMastery)

| Feature                          | Rust                                                          | Tests                       | UI / RPC                                      | Docs                                                                                                                          |
| -------------------------------- | ------------------------------------------------------------- | --------------------------- | --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| FSRS retrievability by GRE tag   | `rslib/src/stats/mastery.rs`                                  | `mastery.rs`                | `StatsService.TopicMastery`, `(gre)/progress` | [memory-model.md](../models/memory-model.md), [gre-atlas-topic-mastery-rust-note.md](../gre-atlas-topic-mastery-rust-note.md) |
| Catalog coverage                 | `rslib/src/gre_atlas/domain/coverage.rs`, `domain/catalog.rs` | `coverage.rs`, `catalog.rs` | `(gre)/ui/GreCoverageSummary.svelte`          | [memory-model.md](../models/memory-model.md)                                                                                  |
| Coverage report (dashboard/eval) | `rslib/src/gre_atlas/coverage_report.rs`                      | `coverage_report.rs`        | `GetDashboard`, eval report                   | [EVALUATION.md](./EVALUATION.md)                                                                                              |

## Study plan & topic insights

| Feature                    | Rust                                            | Tests                                 | UI / RPC                                     | Docs                                                                      |
| -------------------------- | ----------------------------------------------- | ------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------- |
| Ranked recommendations     | `rslib/src/gre_atlas/study_plan.rs`             | `study_plan.rs`                       | `GetStudyPlan`, `(gre)/study-plan`           | [gre-atlas-product-architecture.md](../gre-atlas-product-architecture.md) |
| Daily focus topics (top 3) | `study_plan.rs` (`DAILY_FOCUS_TOPIC_COUNT = 3`) | `study_plan.rs`                       | `(gre)/DailyStudyPlan.svelte`                | [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)                                  |
| Topic priority factors     | `rslib/src/gre_atlas/topic_insights.rs`         | (via `study_plan.rs`, `dashboard.rs`) | `(gre)/ui/GreStudyRecommendationCard.svelte` | [ARCHITECTURE.md](./ARCHITECTURE.md)                                      |
| Topic drill-down           | `rslib/src/gre_atlas/topic_details.rs`          | `topic_details.rs`                    | `GetTopicDetails`, `(gre)/topics/[topicId]`  | [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md) D4                         |

## Dashboard & practice

| Feature                  | Rust                                               | Tests                                             | UI / RPC                                         | Docs                                                           |
| ------------------------ | -------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------ | -------------------------------------------------------------- |
| Dashboard aggregation    | `rslib/src/gre_atlas/dashboard.rs`                 | `dashboard.rs`                                    | `GetDashboard`, `(gre)/home`, `(gre)/dashboard`  | [ARCHITECTURE.md](./ARCHITECTURE.md)                           |
| GRE question bank        | `rslib/src/gre_atlas/questions/mod.rs`             | `questions/mod.rs`                                | `ListQuestions`, `GetQuestion`, `(gre)/practice` | [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)                       |
| Practice sessions        | `rslib/src/gre_atlas/service.rs`, `storage/mod.rs` | `storage/mod.rs`, `pylib/tests/test_gre_atlas.py` | `CreateSession`, `RecordAttempt`                 | [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md) A2–A4           |
| Service layer (all RPCs) | `rslib/src/gre_atlas/service.rs`                   | integration via module tests                      | `@generated/backend`                             | [proto/anki/brainlift.proto](../../proto/anki/brainlift.proto) |

## Evaluation modules

| Feature                    | Rust                          | Tests                                     | Output                                      | Docs                                                                            |
| -------------------------- | ----------------------------- | ----------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------- |
| Eval report orchestration  | `rslib/src/gre_atlas/eval.rs` | `eval.rs`                                 | `results/gre-atlas-eval.{json,md}`          | [EVALUATION.md](./EVALUATION.md)                                                |
| Readiness calibration eval | `calibration.rs`              | `calibration.rs`                          | eval report § calibration                   | [scripts/eval/README.md](../../scripts/eval/README.md)                          |
| FSRS memory eval           | `memory_eval.rs`              | `memory_eval.rs`                          | eval report § memory                        | [scripts/eval/README.md](../../scripts/eval/README.md)                          |
| Performance model eval     | `performance_eval.rs`         | `performance_eval.rs`                     | eval report + `results/performance-eval.md` | [EVALUATION.md](./EVALUATION.md)                                                |
| Topic-priority ablation    | `ablation_eval.rs`            | `ablation_eval.rs`                        | eval report § ablation                      | [scripts/eval/README.md](../../scripts/eval/README.md)                          |
| Eval CLI                   | —                             | `pylib/tests/test_gre_atlas.py`           | `scripts/eval/gre_atlas_eval.py`            | [EVALUATION.md](./EVALUATION.md)                                                |
| Benchmark harness          | —                             | `pylib/tests/test_gre_atlas_benchmark.py` | `scripts/eval/gre_atlas_benchmark.py`       | [EVALUATION.md](./EVALUATION.md) §2; outputs in `results/gre-atlas-benchmark.*` |

## AI question generation

| Feature                               | Rust                                                | Tests        | UI / RPC                            | Docs                                |
| ------------------------------------- | --------------------------------------------------- | ------------ | ----------------------------------- | ----------------------------------- |
| ETS source excerpts + attribution     | `questions/source.rs`, `storage/upgrade_3_to_4.sql` | `ai_gen.rs`  | `GenerateQuestion`                  | [AI.md](./AI.md)                    |
| Template generation + confidence gate | `questions/ai_gen.rs`                               | `ai_gen.rs`  | `GenerateQuestion`                  | [AI.md](./AI.md) § confidence       |
| Gold-set eval (50 questions)          | `ai_eval.rs`, `questions/gold_eval_set.json`        | `ai_eval.rs` | `GenerateBrainLiftAiEvalReport`     | [AI.md](./AI.md) § eval             |
| Keyword baseline comparison           | `questions/ai_gen.rs`                               | `ai_eval.rs` | —                                   | [AI.md](./AI.md)                    |
| AI eval CLI                           | —                                                   | —            | `scripts/eval/gre_atlas_ai_eval.py` | [EVALUATION.md](./EVALUATION.md) §3 |

## Sync, demo, mobile

| Feature                   | Rust                                 | Tests                         | UI / RPC                            | Docs                                                                                              |
| ------------------------- | ------------------------------------ | ----------------------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------- |
| greatlas.db sync          | `rslib/src/gre_atlas/sync.rs`        | `sync.rs`                     | `GetBrainLiftSyncStatus`, Pull/Push | [gre-atlas-mobile.md](../gre-atlas-mobile.md)                                                     |
| Demo collection bootstrap | `rslib/src/gre_atlas/demo.rs`        | `demo.rs`                     | `PrepareDemoCollection`             | [mobile/ios/DEMO.md](../../mobile/ios/DEMO.md)                                                    |
| Mobile parity             | `mobile/mobile_bridge/src/parity.rs` | `cargo test -p mobile_bridge` | iOS companion                       | [gre-atlas-mobile.md](../gre-atlas-mobile.md), [mobile/ios/README.md](../../mobile/ios/README.md) |

## Python & Qt bridge

| Feature                | Location                  | Tests                           |
| ---------------------- | ------------------------- | ------------------------------- |
| Python wrappers        | `pylib/anki/gre_atlas.py` | `pylib/tests/test_gre_atlas.py` |
| GRE main shell         | `qt/aqt/gre_dashboard.py` | manual / e2e                    |
| GRE modal dialog       | `qt/aqt/gre_atlas.py`     | manual / e2e                    |
| mediasrv RPC whitelist | `qt/aqt/mediasrv.py`      | —                               |

## Protobuf

| File                         | RPC count                        |
| ---------------------------- | -------------------------------- |
| `proto/anki/brainlift.proto` | 18 (`BrainLiftService`)          |
| `proto/anki/stats.proto`     | +1 (`StatsService.TopicMastery`) |

Full RPC table: [ARCHITECTURE.md](./ARCHITECTURE.md#rpc-surface).
