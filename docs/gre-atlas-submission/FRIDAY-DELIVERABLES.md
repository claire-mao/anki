# GRE Atlas — Friday deliverables

Master index for the Friday submission bundle. Every item links to **evidence on disk** or a **reproducible command**. Nothing here documents features that are not implemented.

Generated: **2026-07-05**

---

## Deliverable checklist

| # | Deliverable             | Document                                                                | Evidence                                                                                                                   | Status                                                                                                                                        |
| - | ----------------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 | **AI design note**      | [AI.md](./AI.md)                                                        | Source: `rslib/src/gre_atlas/questions/`, `ai_eval.rs`, `retrieval.rs`                                                     | ✅ Complete                                                                                                                                   |
| 2 | **Evaluation report**   | [EVALUATION.md](./EVALUATION.md)                                        | [results/gre-atlas-eval.md](./results/gre-atlas-eval.md), [performance-eval.md](./results/performance-eval.md)             | ✅ Regenerated 2026-07-05                                                                                                                     |
| 3 | **Baseline comparison** | [AI.md § Baseline comparison](./AI.md#baseline-comparison-eval-harness) | [results/gre-atlas-ai-eval.md](./results/gre-atlas-ai-eval.md)                                                             | ✅ Regenerated 2026-07-05                                                                                                                     |
| 4 | **Sync verification**   | [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md)                          | [results/friday-verification-2026-07-05.log](./results/friday-verification-2026-07-05.log)                                 | ✅ Automated tests pass                                                                                                                       |
| 5 | **Demo checklist**      | [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)                                | [SCREENSHOTS.md](./SCREENSHOTS.md), [RECORDINGS.md](./RECORDINGS.md), [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md) | ⚠️ UI screenshots 01–07 + recordings manual — see [screenshots/pending/](./screenshots/pending/), [recordings/pending/](./recordings/pending/) |

### Screenshot status

| File                                      | Status                                                                                     |
| ----------------------------------------- | ------------------------------------------------------------------------------------------ |
| `08-eval-report.png`                      | ✅ Generated from eval output                                                              |
| `10-benchmark-output.png`                 | ✅ Generated from benchmark output                                                         |
| `01-gre-home.png` … `07-congrats-cta.png` | ⚠️ Capture from running Anki (`just run`) — external browser cannot render mediasrv webview |

---

## Evaluation numbers (2026-07-05)

### Full eval (`just eval-gre-atlas` on demo collection)

Collection: `mobile/ios/GREAtlasCompanion/Resources/DemoBundle/collection.anki2`

| Section                 | Result                                                                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Readiness calibration   | Insufficient (0 predictions) — honest abstention                                                                                |
| FSRS memory calibration | Insufficient (FSRS disabled in demo collection)                                                                                 |
| Performance model       | Insufficient (0 held-out attempts; 4 train attempts)                                                                            |
| Abstention              | 100% — FSRS off, 0 studied cards, 4/50 practice attempts                                                                        |
| Topic-priority ablation | **GRE Atlas priority wins** learning gain (1.800 vs random 0.390 vs vanilla 1.650) and coverage (+15.4%) on collection scenario |

### AI eval (`just eval-gre-atlas-ai`)

| Metric                                | Value                                                |
| ------------------------------------- | ---------------------------------------------------- |
| Release verdict                       | **PASS**                                             |
| Held-out accuracy                     | 100% (50/50)                                         |
| Wrong-answer rate                     | 0%                                                   |
| AI pipeline accuracy vs best baseline | 62.0% vs BM25 60.0%                                  |
| Rejection pipeline                    | 1 hallucination, 1 duplicate, 1 unsupported rejected |

### Benchmark (`just bench-gre-atlas --synthetic-cards 10000`)

| API          |     p50 |     p95 |   Worst |
| ------------ | ------: | ------: | ------: |
| TopicMastery | 0.02 ms | 0.03 ms | 0.03 ms |
| Dashboard    | 0.08 ms | 0.08 ms | 0.09 ms |
| Readiness    | 0.04 ms | 0.04 ms | 0.05 ms |
| Study plan   | 2.03 ms | 2.06 ms | 2.06 ms |

Data source: `synthetic_reference` (10,000 GRE-tagged cards, FSRS enabled).

---

## Sync verification summary

Automated tests (see log):

```bash
cargo test -p anki gre_atlas::storage::sync_bundle   # 6 passed
cargo test -p anki gre_atlas::sync                   # 22 passed (incl. friday_sync_loop HTTP test)
cargo test -p anki gre_atlas                         # 178 passed
cargo check -p mobile_bridge                         # pass
```

Friday HTTP loop test: `gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect` — desktop ↔ phone practice sync, offline edits on both devices, reconnect merge.

Full scenario matrix: [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md).

---

## Proof artifacts

| Artifact              | Path                                         | How to regenerate                               |
| --------------------- | -------------------------------------------- | ----------------------------------------------- |
| Eval JSON             | `results/gre-atlas-eval.json`                | `just eval-gre-atlas /path/to/collection.anki2` |
| Eval Markdown         | `results/gre-atlas-eval.md`                  | same                                            |
| Performance deep-dive | `results/performance-eval.md`                | same                                            |
| AI eval JSON/MD       | `results/gre-atlas-ai-eval.{json,md}`        | `just eval-gre-atlas-ai`                        |
| Benchmark JSON/MD/CSV | `results/gre-atlas-benchmark.{json,md,csv}`  | `just bench-gre-atlas --synthetic-cards 10000`  |
| Verification log      | `results/friday-verification-2026-07-05.log` | Re-run commands in log header                   |
| UI screenshots        | `screenshots/01-*.png` … `08-*.png`          | [SCREENSHOTS.md](./SCREENSHOTS.md) — **manual** |
| Screen recordings     | `recordings/`                                | [RECORDINGS.md](./RECORDINGS.md) — **manual**   |

Note: `results/*` and `screenshots/*.png` are gitignored locally; regenerate before submission.

---

## Documentation ↔ implementation audit

| Claim                                                                  | Verified against                           |
| ---------------------------------------------------------------------- | ------------------------------------------ |
| 18 BrainLift RPCs + TopicMastery                                       | `proto/anki/brainlift.proto`, `service.rs` |
| Practice does not write revlog                                         | `pylib/tests/test_gre_atlas.py`            |
| Abstention gates (50 cards, 50% coverage, 50 attempts)                 | `abstention.rs`, model docs                |
| AI eval baselines (keyword, BM25, TF-IDF, AI retrieval)                | `retrieval.rs`, `ai_eval.rs`               |
| Eval gate thresholds (0.55 confidence, 0.15 grounding, 0.85 duplicate) | `eval_pipeline.rs`, `ai_gen.rs`            |
| AI-off mode (`GRE_ATLAS_AI_DISABLED`; scores independent of LLM)       | `llm.rs`, [AI.md § Offline-first](./AI.md) |
| Sync LWW on `mtime_secs`                                               | `sync_bundle.rs`, unit tests               |
| ExplainAnswer on web practice only                                     | `ts/routes/(gre)/practice/+page.svelte`    |

---

## Reviewer quick path

1. Read this page → [AI.md](./AI.md) → [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md)
2. Run `just eval-gre-atlas-ai` and `just bench-gre-atlas --synthetic-cards 10000`
3. Follow [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md) for live demo
4. Cross-check [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md)
