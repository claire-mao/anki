# GRE Atlas model input sync audit

Audit date: 2026-07-03. Scope: readiness-related model inputs across desktop (Qt/web) and iOS companion after GRE Atlas sidecar sync (`greatlas.db` bundle via `perform_gre_atlas_sync` / `gre_atlas/sync.rs`).

## Executive summary

| Area | Classification | Notes |
|------|----------------|-------|
| Performance | **PASS** | `bl_session` + `bl_performance_attempt` sync via bundle; cross-device id collision fixed |
| Calibration | **PASS** | `bl_readiness_prediction` syncs; outcome resolution now bumps USN |
| Confidence | **PASS** (sidecar path) | Derived from readiness + calibration rows; matches after sidecar sync when collections match |
| Readiness | **PARTIAL** | Performance/calibration inputs sync; memory/coverage legs require collection |
| Memory | **PARTIAL** | FSRS retrievability from `collection.anki2`; no collection sync on iOS |
| Coverage | **PARTIAL** | `gre::` tag coverage from collection; no collection sync on iOS |
| Dashboard | **PARTIAL** | Sidecar slices match; weak/recommended topics and coverage diverge without collection |
| Progress | **PARTIAL** | Uses same RPCs as dashboard/scores/calibration; same split as dashboard |
| Study plan | **PARTIAL** | Practice targets sync; review due counts and memory-based ranking need collection |

Automated parity tests live in `rslib/src/gre_atlas/sync_parity.rs`. All **155** `gre_atlas` Rust tests pass (`cargo test -p anki gre_atlas`).

---

## Dual storage model

| Store | Path | Contents | Desktop sync | iOS sync |
|-------|------|----------|--------------|----------|
| Collection | `collection.anki2` | Cards, revlog, FSRS, `gre::` tags, deck due counts | AnkiWeb collection sync | **None** |
| Sidecar | `greatlas.db` beside profile | Practice, sessions, calibration predictions, question bank | GRE Atlas bundle sync (`/gre/sync/`) | GRE Atlas bundle sync |

Signal pipeline entry point: `load_gre_atlas_signals()` in `rslib/src/gre_atlas/signals.rs`.

---

## Per-metric audit

### Memory (45% of readiness)

| Field | Source | Sync channel | Parity requirement |
|-------|--------|--------------|-------------------|
| `overall_avg_retrievability` | `StatsService.TopicMastery` on GRE deck + `gre::` tags | Collection only | Identical collection state |
| `studied_cards`, topic retrievability | FSRS revlog in `collection.anki2` | Collection only | Identical collection state |
| `coverage_ratio` (memory leg) | Leaf topics with studied cards / catalog | Derived from collection | Identical collection state |

**Syncs today:** derived-only on each device; **not** in GRE bundle.

**Classification:** **PARTIAL** — sidecar sync cannot replicate memory without collection sync on iOS.

---

### Performance (45% of readiness)

| Field | Source | Sync channel | Parity requirement |
|-------|--------|--------------|-------------------|
| `correct`, `total`, `attempt_count` | `bl_performance_attempt` aggregates | Bundle (`attempts`) | Full bundle merge |
| `practice_by_topic` | Same table, grouped by topic | Bundle | Full bundle merge |
| Session FK | `bl_session` | Bundle (`sessions`) | Sessions before attempts (FK-safe order) |

**Syncs today:** `pull_sync_bundle` / `apply_sync_bundle` in `storage/sync_bundle.rs`; HTTP via `sync_transport.rs` and `sync/http_server/gre_atlas_sync.rs`.

**Gap fixed:** Per-device auto-increment attempt ids collided on merge (id `1` on both devices treated as one row). `push_changes` now detects identity mismatch (`question_id`, `answered_at_secs`, `session_id`) and inserts a new row instead of last-write-wins on the colliding id.

**Classification:** **PASS**

---

### Coverage (10% of readiness)

| Field | Source | Sync channel | Parity requirement |
|-------|--------|--------------|-------------------|
| `weighted_ratio`, `covered_leaf_count` | `compute_coverage()` over mastery topics with `studied_cards > 0` | Collection only | Identical collection state |
| Section breakdown | GRE catalog + observed tags | Derived | Identical collection state |

**Syncs today:** derived-only.

**Classification:** **PARTIAL**

---

### Calibration

| Field | Source | Sync channel | Parity requirement |
|-------|--------|--------------|-------------------|
| Prediction rows | `bl_readiness_prediction` | Bundle (`predictions`) | Merge by id + `mtime_secs` LWW |
| `brier_score`, calibration curve | `compute_calibration_stats()` over synced rows | Derived after merge | Same prediction rows |
| Outcome resolution | `resolve_pending_outcomes()` updates rows | Must bump `usn` | Upload resolved outcomes |

**Syncs today:** bundle includes predictions since schema v3+.

**Gap fixed:** `resolve_pending_outcomes()` updated `mtime_secs` but not `usn`, so resolved outcomes never uploaded. Update now assigns `next_usn()` on outcome write.

**Classification:** **PASS**

---

### Confidence

| Field | Source | Sync channel |
|-------|--------|--------------|
| `confidence_level`, intervals | `compute_readiness_score()` + `apply_calibration_honesty()` | Derived |
| Calibration downgrade | `bl_readiness_prediction` held-out Brier | Sidecar + derived |

**Classification:** **PASS** when sidecar + identical collection; **PARTIAL** on iOS without collection (memory abstention affects readiness confidence inputs).

---

### Readiness

| Input | Weight | Sync |
|-------|--------|------|
| Memory score | 45% | Collection |
| Performance score | 45% | Sidecar |
| Coverage ratio | 10% | Collection |
| Calibration honesty | overlay | Sidecar |

**Classification:** **PARTIAL** — performance and calibration align after bundle sync; projected score still differs if memory/coverage differ across devices.

---

### Dashboard

| Slice | Primary inputs | Sync |
|-------|----------------|------|
| Memory / performance / readiness scores | Signals cache | Mixed |
| Coverage panel | Collection mastery | Collection |
| Weak / recommended topics | Mastery + practice_by_topic | Mixed |
| Recent activity | `bl_performance_attempt` | Sidecar |
| Estimated GRE | All of the above | Mixed |

RPC: `gre_atlas_get_dashboard`.

**Classification:** **PARTIAL**

---

### Progress

Progress page (`ts/routes/(gre)/progress/+page.ts`) loads: `getScores`, `getDashboard`, `getRecentAttempts`, `topicMastery`, `getReadinessCalibration`.

Same split as dashboard + scores; no separate backend store.

**Classification:** **PARTIAL**

---

### Study plan

| Slice | Primary inputs | Sync |
|-------|----------------|------|
| Practice daily targets | Performance + readiness signals | Sidecar |
| Review due targets | GRE deck due counts from collection scheduler | Collection |
| Topic recommendations | Mastery, coverage, `practice_by_topic` | Mixed |

RPC: `gre_atlas_get_study_plan`.

**Classification:** **PARTIAL**

---

## Sync bundle contents (sidecar)

Exported incrementally by `usn > after_usn` in `storage/sync_bundle.rs`:

| Table | Included | Merge strategy |
|-------|----------|----------------|
| `bl_session` | Yes | LWW on `mtime_secs` |
| `bl_question` | Yes | LWW on `mtime_secs` |
| `bl_performance_attempt` | Yes | LWW on `mtime_secs`; id collision → new insert |
| `bl_readiness_prediction` | Yes | LWW on `mtime_secs` |

Merge order: sessions → questions → attempts → predictions (FK safe). Stale sessions ride along with new attempts when session `usn ≤ last_pushed`.

Questions: seeded identically on each device from foundation bank; bundle sync needed for AI-generated questions created on one device only.

---

## Code changes (this audit)

1. **`storage/mod.rs` — `resolve_pending_outcomes`:** bump `usn` when writing outcome fields so resolved calibration rows upload.
2. **`storage/mod.rs` — `push_changes`:** detect cross-device attempt id collision via `attempt_identity_differs()` and insert as new row.
3. **`sync_parity.rs`:** automated before/after parity tests (new file).
4. **`mod.rs`:** register test module.

No readiness formula weights changed. Foreign keys remain enabled.

---

## Automated before/after comparisons

Tests simulate isolated desktop and mobile profiles (`CollectionBuilder` temp dirs), apply `pull_sync_bundle(0)` → `apply_sync_bundle`, then compare RPC snapshots.

### Test: `bundle_sync_sidecar_metrics_match_after_desktop_to_mobile`

Desktop seeds 50 practice attempts (`MIN_PERFORMANCE_ATTEMPTS`); mobile starts empty; identical empty collections.

| Metric | Mobile before sync | Mobile after sync | Desktop after sync | Match? |
|--------|-------------------|-------------------|--------------------|--------|
| `performance_attempt_count` | 0 | 50 | 50 | Yes |
| `performance_sufficient` | false | true | true | Yes |
| `performance_value` | None | Some | Some | Yes |
| `recent_activity_count` | 0 | 10 | 10 | Yes |
| `daily_practice_target` | (bootstrap) | (computed) | (computed) | Yes |
| Full `SidecarMetricsSnapshot` | ≠ desktop | = desktop | = desktop | **PASS** |
| `memory_studied_cards` | 0 | 0 | 0 | Yes (both empty) |
| `coverage_observed_leaves` | 0 | 0 | 0 | Yes |

### Test: `bundle_sync_predictions_match_calibration_stats`

| Metric | Mobile before | Mobile after | Match? |
|--------|---------------|--------------|--------|
| `calibration_total_predictions` | 0 | 1 | Yes |
| `calibration_resolved` | 0 | 1 | Yes |
| Full sidecar snapshot vs desktop | ≠ | = | **PASS** |

### Test: `outcome_resolution_marks_prediction_for_sync`

Pending prediction → 3 practice attempts → `resolve_pending_outcomes` → `pull_sync_bundle(last_pushed)` includes prediction with `outcome_score` set. **PASS**

### Test: `divergent_collections_sidecar_matches_collection_differs`

Same sidecar sync with FSRS enabled on desktop only; sidecar metrics still match; collection metrics match because both collections remain empty (documents that real-world divergence comes from collection activity, not sidecar). **PARTIAL** scenario documented.

---

## What cannot sync without collection sync on iOS

- FSRS retrievability and memory score
- Catalog coverage from studied `gre::` flashcards
- Topic mastery weak/recommended ranking (memory leg)
- Study plan review-card due counts from GRE deck scheduler
- Full readiness projected score (requires memory + coverage legs)
- Progress mastery charts driven by `topicMastery` RPC

**What does sync and aligns today:**

- All practice attempts and sessions
- Readiness calibration prediction history and resolved outcomes
- Performance score and practice-derived study-plan targets
- Dashboard recent activity list
- AI-generated questions (when created on one device)

---

## Recommendations

1. **Ship sidecar fixes** (USN on outcome resolution, attempt id collision) — done in this audit.
2. **Treat iOS readiness as PARTIAL** in product copy until collection sync exists or a dedicated flashcard export/import path is added.
3. **Optional future work:** export minimal collection mastery snapshot in GRE bundle (high scope; not implemented here).
4. **Verify in QA:** run `perform_gre_atlas_sync` on desktop after practice, then sync iOS companion; confirm performance and calibration RPCs match; expect memory/coverage to differ if iOS deck has different review history.

---

## Test command

```bash
cargo test -p anki gre_atlas
```

Expected: **155 passed** (includes 4 parity tests in `sync_parity.rs`).
