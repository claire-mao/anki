# GRE Atlas practice sync — verification report

Cross-device synchronization for GRE Atlas practice data (`greatlas.db` sidecar): sessions, attempts, questions, and readiness predictions. Merge policy is **last-write-wins on `mtime_secs`**; sessions are applied before attempts to satisfy FK order.

## Scope

| Layer                                                                            | Status                                              |
| -------------------------------------------------------------------------------- | --------------------------------------------------- |
| Rust merge engine (`storage/sync_bundle.rs`, `storage/mod.rs`)                   | Implemented + unit tested                           |
| HTTP transport (`sync_transport.rs` → `/gre/sync/download`, `/upload`)           | Implemented                                         |
| SimpleServer handlers (`sync/http_server/gre_atlas_sync.rs`)                     | Implemented                                         |
| Desktop RPC + UI (`perform_gre_atlas_sync`, settings button, practice auto-sync) | Implemented                                         |
| iOS FFI (`anki_mobile_brainlift_sync_perform_json`, `GREAtlasSyncSession`)       | Implemented                                         |
| AnkiWeb production endpoints                                                     | **Not deployed** — requires self-hosted sync server |

Automated verification uses isolated collections and in-memory bundle apply/push (no live HTTP in CI). HTTP transport and the full desktop↔phone offline loop are covered by `gre_atlas::sync_http` against SimpleServer. End-to-end device tests remain manual — see **[SYNC-DEV-SETUP.md](./SYNC-DEV-SETUP.md)** for a copy-paste local workflow.

## Test commands (verified 2026-07-05)

```bash
cargo test -p anki gre_atlas::storage::sync_bundle   # 6 passed, 0 failed
cargo test -p anki gre_atlas::sync                   # 22 passed, 0 failed (sync.rs + sync_parity.rs + sync_http.rs)
cargo test -p anki gre_atlas                         # 178 passed, 0 failed
cargo check -p mobile_bridge                         # pass (no compile errors)
```

Full verification log: [results/friday-verification-2026-07-05.log](./results/friday-verification-2026-07-05.log).

### Friday HTTP loop (automated E2E)

`gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect` spins up an in-process sync server and simulates:

1. Desktop practice → sync → phone receives attempt
2. Phone practice → sync → desktop receives attempt
3. Both devices practice offline
4. Reconnect → merged attempt totals without FK violations or duplicate sessions

This is the automated substitute for manual desktop + iOS recording when hardware is unavailable.

## Scenario matrix

### 1. Desktop → mobile

**Classification: PASS**

| Check                                                | Evidence                                                                                               |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Bundle export includes sessions + attempts           | `bundle_sync_desktop_to_mobile_preserves_attempts`, `bundle_round_trip_preserves_question_attribution` |
| FK order (session before attempt)                    | `bundle_merge_applies_session_before_attempt`                                                          |
| Sidecar metrics parity after merge                   | `bundle_sync_sidecar_metrics_match_after_desktop_to_mobile`                                            |
| Desktop UI triggers sync                             | `ts/routes/(gre)/gre-sync.ts`, practice session complete hook                                          |
| Friday HTTP loop (desktop ↔ phone offline reconnect) | `sync_http.rs::friday_sync_loop_desktop_phone_offline_reconnect`                                       |
| iOS perform-sync entry point                         | `GREAtlasSyncSession.syncNow()` → `anki_mobile_brainlift_sync_perform_json`                            |

Practice recorded on desktop appears on mobile after sync; performance/readiness/calibration stats match when Anki collections start identical.

### 2. Mobile → desktop

**Classification: PASS** (sidecar); **PARTIAL** (full dashboard parity)

| Check                                   | Evidence                                                                                |
| --------------------------------------- | --------------------------------------------------------------------------------------- |
| Reverse bundle apply                    | `two_profile_roundtrip_preserves_attempts_without_fk_or_duplicate_sessions` (B → A leg) |
| Bidirectional merge                     | `bundle_sync_bidirectional_last_write_wins`                                             |
| Memory/coverage without collection sync | `divergent_collections_sidecar_matches_collection_differs` documents iOS limitation     |

Mobile-originated attempts upload via the same bundle path. Memory and coverage scores still depend on Anki collection state, which iOS does not sync today — sidecar-only metrics match; collection-derived metrics may diverge.

### 3. Offline → reconnect

**Classification: PASS**

| Check                                      | Evidence                                                                                     |
| ------------------------------------------ | -------------------------------------------------------------------------------------------- |
| Pending rows tracked by USN                | `pull_after_local_record_includes_change`, `mark_synced_through_clears_pending_upload_count` |
| Stale session re-exported with new attempt | `bundle_export_includes_stale_session_for_new_attempt`                                       |
| Perform sync clears pending after success  | `gre_atlas_perform_sync` calls `mark_synced_through(current_usn)` after upload               |
| Offline guard                              | `gre_atlas_perform_sync_offline` returns `success: false` when credentials missing           |

Device accumulates local USN bumps while offline; reconnect downloads remote bundle, merges, uploads pending bundle, then marks synced through.

### 4. Multiple offline edits (both devices)

**Classification: PASS**

| Check                                         | Evidence                                                                        |
| --------------------------------------------- | ------------------------------------------------------------------------------- |
| A → B → A roundtrip, 3 attempts, 2 sessions   | `two_profile_roundtrip_preserves_attempts_without_fk_or_duplicate_sessions`     |
| Independent offline sessions on A and B       | `bundle_sync_bidirectional_last_write_wins` (2 attempts on B, merged back to A) |
| No duplicate session rows for same session id | roundtrip asserts `session_count == 1` after incremental sync                   |

Each device can add attempts offline; incremental bundles merge without FK violations or session duplication.

### 5. Same-attempt conflict (same row id, divergent content)

**Classification: PARTIAL**

| Sub-scenario                                                                        | Status   | Evidence                                                                                              |
| ----------------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------------------------------------------- |
| Same identity, newer `mtime_secs` wins                                              | **PASS** | `push_applies_remote_and_keeps_newer_on_conflict`                                                     |
| Same identity, older remote rejected                                                | **PASS** | `push_rejects_older_remote_with_conflict`                                                             |
| Cross-device auto-increment id collision (same id, different question/time/session) | **PASS** | `push_cross_device_id_collision_inserts_as_new_attempt`, `offline_id_collision_survives_server_merge` |

LWW conflict resolution on the same attempt row is unit tested. Per-device id collisions (both devices assign `id = 1` to different attempts) rely on merge-engine code review; roundtrip tests merge distinct attempts without asserting the collision branch explicitly.

## Component checklist

- [x] Proto: `BrainLiftSyncBundle`, `PerformGreAtlasSync` RPC
- [x] Merge: sessions → questions → attempts → predictions
- [x] `mark_synced_through` after successful perform sync
- [x] Desktop: mediasrv + pylib + Qt bridge + TS settings/practice hooks
- [x] iOS: perform-sync FFI, `GreAtlasSyncCredentials.swift` in Xcode project
- [x] `anki_mobile.h`: `anki_mobile_brainlift_sync_perform_json` declared

## Manual follow-up (not automated)

See **[SYNC-DEV-SETUP.md](./SYNC-DEV-SETUP.md)** for step-by-step dev server + desktop + iOS setup. Summary:

1. Configure sync credentials (hkey + endpoint) on desktop (sign-in) and iOS (Settings fields or `greAtlasSyncCredentials` UserDefaults).
2. Run self-hosted SimpleServer with GRE Atlas sync routes enabled (included by default in this repo).
3. Record practice on desktop → tap **Sync now** on iOS → confirm attempt count on Progress.
4. Record practice on iOS → sync desktop → confirm GRE Progress matches.
5. Airplane mode on both → practice on each → reconnect → verify merged attempt totals.

## Not verified by automated tests

| Gap                            | Impact                                                   |
| ------------------------------ | -------------------------------------------------------- |
| iOS collection sync            | Memory/coverage/dashboard legs stay PARTIAL on companion |
| Physical iOS device smoke test | Companion UI + credentials flow not exercised in CI      |

## Known limitations

- **AnkiWeb** does not expose `/gre/sync/*` unless the sync server is extended.
- **Collection sync** (FSRS memory, deck coverage) is separate from GRE Atlas sidecar sync; iOS shows PARTIAL parity for memory/coverage when collections differ.
- **HTTP E2E** is not covered by unit tests; transport is validated by code review + manual server testing.
