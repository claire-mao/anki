# GRE Atlas — Wednesday Release Proof

Generated: **2026-07-05**

This document records evidence that every **Wednesday requirement** passes on the current working tree.

## Commit

| Field      | Value                                                 |
| ---------- | ----------------------------------------------------- |
| **HEAD**   | `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`            |
| **Branch** | (working tree; uncommitted GRE Atlas changes present) |
| **Verify** | `git rev-parse HEAD`                                  |

Full command output: [wednesday-test-output.txt](./wednesday-test-output.txt)

---

## Requirement matrix

| Requirement                                    | Status | Evidence                                                                                                                                | Files modified                      |
| ---------------------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| **Desktop: Fork builds from source**           | PASS   | `just build` → Build succeeded in 25.86s                                                                                                | —                                   |
| **Desktop: Rust Topic Mastery end-to-end**     | PASS   | `rslib/src/stats/mastery.rs` uses `gre_atlas::abstention`; 11 Rust tests + 3 Python `test_topic_mastery_*` pass                         | `rslib/src/stats/mastery.rs`        |
| **Desktop: ≥3 Rust mastery tests**             | PASS   | 11 passed (`cargo test -p anki stats::mastery`)                                                                                         | —                                   |
| **Desktop: ≥1 Python integration test**        | PASS   | 22 passed: `test_topic_mastery_*` + `test_gre_atlas.py`                                                                                 | —                                   |
| **Desktop: Review loop on GRE deck**           | PASS   | `test_dashboard_memory_reflects_gre_deck_reviews`; `gre_atlas::demo::test::prepare_demo_seeds_deck_cards_and_practice`                  | —                                   |
| **Desktop: Memory model runs**                 | PASS   | `just eval-gre-atlas` → `results/gre-atlas-eval.md` § FSRS memory calibration; 5 `memory_eval` Rust tests pass                          | —                                   |
| **Desktop: Honest score (range + give-up)**    | PASS   | `MIN_STUDIED_CARDS=50`, `MIN_PERFORMANCE_ATTEMPTS=50`; 9 `abstention` tests + `test_readiness_abstention_lists_missing_requirements`    | `rslib/src/gre_atlas/abstention.rs` |
| **Desktop: Installer builds**                  | PASS   | `tools/build-installer` → `out/installer/dist/anki-26.05-mac-apple.dmg` (214 MB)                                                        | —                                   |
| **Mobile: Builds in Xcode**                    | PASS   | `xcodebuild … -destination 'platform=iOS Simulator,name=iPhone 17' build` → **BUILD SUCCEEDED**                                         | —                                   |
| **Mobile: Runs in simulator**                  | PASS   | `xcrun simctl launch booted org.ankitects.greatlas.companion` → pid 13122; see [WEDNESDAY-PHONE-REVIEW.md](./WEDNESDAY-PHONE-REVIEW.md) | `WEDNESDAY-PHONE-REVIEW.md`         |
| **Mobile: Loads GRE deck (DemoBundle)**        | PASS   | `cargo test -p mobile_bridge ios_demo_bundle::generate_ios_demo_bundle_writes_required_files`; `PrepareDemoCollection` idempotent       | —                                   |
| **Mobile: Review session works**               | PASS   | `gre_study_review_matches_between_mobile_ffi_and_direct_backend`; `grade_buttons_follow_scheduler_labels`                               | —                                   |
| **Mobile: Shared Rust engine (mobile_bridge)** | PASS   | `cargo test -p mobile_bridge` → 24 passed; `cargo check -p mobile_bridge`                                                               | —                                   |
| **Proof: Commit hash documented**              | PASS   | This file + `wednesday-test-output.txt` header                                                                                          | `wednesday-release-proof.md`        |
| **Proof: Clean build verified**                | PASS   | `just build`, `just test-ts`, `cargo test -p anki gre_atlas` (178 passed)                                                               | —                                   |
| **Proof: Test output captured**                | PASS   | [wednesday-test-output.txt](./wednesday-test-output.txt)                                                                                | `wednesday-test-output.txt`         |
| **Proof: Installer artifact exists**           | PASS   | `out/installer/dist/anki-26.05-mac-apple.dmg`                                                                                           | —                                   |
| **Proof: Phone review recording instructions** | PASS   | [WEDNESDAY-PHONE-REVIEW.md](./WEDNESDAY-PHONE-REVIEW.md)                                                                                | `WEDNESDAY-PHONE-REVIEW.md`         |

---

## Key commands (reproduce)

```bash
cd /Users/clairemao/anki
git rev-parse HEAD

just build
just test-py
just test-ts
cargo test -p anki stats::mastery
cargo test -p anki gre_atlas
cargo test -p mobile_bridge

just eval-gre-atlas mobile/ios/GREAtlasCompanion/Resources/DemoBundle/collection.anki2
tools/build-installer

cd mobile/ios
xcodebuild -project GREAtlasCompanion.xcodeproj -scheme GREAtlasCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 17' build
```

## Known issues resolved (prior audit)

| Issue                        | Resolution                                                                                   |
| ---------------------------- | -------------------------------------------------------------------------------------------- |
| MIN_STUDIED_CARDS still 20   | Already **50** in `abstention.rs`, `empty-states.ts`, `prediction-readiness-presentation.ts` |
| 6 gre_atlas AI test failures | **178/178 pass** (`cargo test -p anki gre_atlas`)                                            |
| sync_http.rs compile errors  | `friday_sync_loop_desktop_phone_offline_reconnect` passes                                    |
| Missing proof deliverables   | This file + `wednesday-test-output.txt` + `WEDNESDAY-PHONE-REVIEW.md`                        |
| Empty `out/installer/dist/`  | **anki-26.05-mac-apple.dmg** built 2026-07-05                                                |
| Review loop / phone review   | Automated tests + simulator recording script                                                 |
