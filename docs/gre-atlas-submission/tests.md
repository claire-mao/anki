# GRE Atlas — Wednesday test evidence

**Audit date:** 2026-07-05\
**Commit:** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

Related: [BUILD.md](./BUILD.md), [FEATURE-INDEX.md](./FEATURE-INDEX.md) (test → source map), [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md).

---

## Summary

| Suite                                | Command                       | Result   | Passed                       | Failed | Ignored |
| ------------------------------------ | ----------------------------- | -------- | ---------------------------- | ------ | ------- |
| Rust (full `anki` crate)             | `cargo test -p anki`          | **PASS** | 709                          | 0      | 1       |
| Rust (`gre_atlas` modules)           | (subset of above)             | **PASS** | 178                          | 0      | 0       |
| Python (`just test-py`)              | `just test-py`                | **PASS** | 73 (qt) + 145 (pylib direct) | 0      | —       |
| TypeScript / Vitest (`just test-ts`) | `just test-ts` + `vitest run` | **PASS** | 111                          | 0      | —       |

Raw logs: [logs/](./logs/) (`wednesday-cargo-test.log`, `wednesday-pytest*.log`, `wednesday-vitest-direct.log`).

---

## Rust — `cargo test -p anki`

```
running 710 tests
test result: ok. 709 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 43.72s
```

**GRE Atlas module tests:** 178 tests matching `gre_atlas::` — all passed.

### Key GRE Atlas test names

| Area                  | Example tests                                                                                                                                                             |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Abstention (50 cards) | `gre_atlas::abstention::test::insufficient_studied_cards_abstains`, `…::memory_sufficient_when_all_requirements_met`, `…::readiness_lists_all_unmet_requirements`         |
| Calibration           | `gre_atlas::calibration::test::brier_score_perfect_when_predictions_match`, `…::insufficient_calibration_data_is_reported_honestly`                                       |
| Sync                  | `gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect`, `gre_atlas::storage::sync_bundle::test::bundle_round_trip_preserves_question_attribution` |
| AI eval               | `gre_atlas::ai_eval::test::ai_eval_report_is_deterministic_for_fixed_timestamp`                                                                                           |
| Questions             | `gre_atlas::questions::…` (generator, bank, eval pipeline)                                                                                                                |

Full output: [logs/wednesday-cargo-test.log](./logs/wednesday-cargo-test.log)

---

## Python — `just test-py`

`just test-py` runs `./ninja check:pytest` (stamp green on audit date).

Direct re-run for counts (same env as ninja):

```bash
PYTHONPATH=out/pylib:pylib ANKI_TEST_MODE=1 out/pyenv/bin/pytest -p no:cacheprovider pylib/tests -q
# 145 passed in 2.10s

PYTHONPATH=out/pylib:pylib:out/qt:out/qt/tools ANKI_TEST_MODE=1 out/pyenv/bin/pytest -p no:cacheprovider qt/tests -q
# 73 passed in 23.67s
```

### Key GRE Atlas Python tests (`pylib/tests/test_gre_atlas.py`)

| Test                                                   | Purpose                                    |
| ------------------------------------------------------ | ------------------------------------------ |
| `test_gre_atlas_record_attempt_does_not_modify_revlog` | Practice isolation — no FSRS/revlog writes |
| `test_readiness_abstains_without_minimum_evidence`     | Abstention on sparse profile               |
| `test_readiness_abstention_lists_missing_requirements` | Requirement checklist in API               |
| `test_gre_atlas_sync_pull_push_roundtrip`              | Sidecar sync round-trip                    |
| `test_get_study_plan_returns_ranked_recommendations`   | Study plan RPC                             |
| `test_get_dashboard_returns_full_state`                | Dashboard snapshot                         |

Logs: [logs/wednesday-pytest-direct.log](./logs/wednesday-pytest-direct.log), [logs/wednesday-pytest-qt.log](./logs/wednesday-pytest-qt.log)

---

## TypeScript — `just test-ts` / Vitest

`just test-ts` runs `./ninja check:vitest` (stamp green).

Direct Vitest run (`node_modules/.bin/vitest run` from `ts/`):

```
Test Files  21 passed (21)
     Tests  111 passed (111)
  Duration  1.99s
```

### GRE route tests (representative)

| File                                                  | Tests |
| ----------------------------------------------------- | ----- |
| `routes/(gre)/practice/practice-presentation.test.ts` | 14    |
| `routes/(gre)/daily-mission.test.ts`                  | 8     |
| `routes/(gre)/calibration-presentation.test.ts`       | 5     |
| `routes/(gre)/recommendation-presentation.test.ts`    | 5     |
| `routes/(gre)/summary-metrics.test.ts`                | 4     |
| `routes/(gre)/practice/practice-session.test.ts`      | 4     |

Log: [logs/wednesday-vitest-direct.log](./logs/wednesday-vitest-direct.log)

---

## Known historical note

[results/friday-verification-2026-07-05.log](./results/friday-verification-2026-07-05.log) recorded one transient failure in the full `gre_atlas::sync` filter run; isolated re-run of `friday_sync_loop_desktop_phone_offline_reconnect` passed. **Wednesday full `cargo test -p anki` run: 709/709 passed (1 ignored).**
