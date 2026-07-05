# GRE Atlas Friday artifact audit — 2026-07-05

Automated inventory of submission proof. Regenerate eval via commands in [FRIDAY-DELIVERABLES.md](./FRIDAY-DELIVERABLES.md).

## Inventory summary

### `results/`

| File                                 | Present | Last modified                                     |
| ------------------------------------ | ------- | ------------------------------------------------- |
| `gre-atlas-eval.{json,md}`           | ✅      | 2026-07-05                                        |
| `performance-eval.md`                | ✅      | 2026-07-05                                        |
| `gre-atlas-ai-eval.{json,md}`        | ✅      | 2026-07-05                                        |
| `gre-atlas-benchmark.{json,md,csv}`  | ✅      | 2026-07-05                                        |
| `friday-verification-2026-07-05.log` | ✅      | 2026-07-05 (sync test failed once; re-run passes) |
| `brainlift-*` (legacy aliases)       | ✅      | older copies alongside gre-atlas-*                |

### `screenshots/`

| Count   | Notes                                                                                                                            |
| ------- | -------------------------------------------------------------------------------------------------------------------------------- |
| 2 / 10  | `08-eval-report.png`, `10-benchmark-output.png` only                                                                             |
| Missing | `01`–`07`, `09`, `11` — see [screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md](./screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md) |

### `recordings/`

| Count         | Notes                                                                                                                          |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 0 video files | No `.mp4`/`.mov`/`.webm` anywhere in repo                                                                                      |
| Scripts       | [recordings/REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md), [RECORDINGS.md](./RECORDINGS.md) |

### Documentation checklist

| Doc                    | Present                          |
| ---------------------- | -------------------------------- |
| DEMO-CHECKLIST.md      | ✅                               |
| SYNC-VERIFICATION.md   | ✅                               |
| AI.md                  | ✅                               |
| EVALUATION.md          | ✅                               |
| RELEASE-CHECKLIST.md   | ✅                               |
| INSTALL.md             | ✅ (clean-machine install guide) |
| SCREENSHOTS.md         | ✅                               |
| RECORDINGS.md          | ✅                               |
| FRIDAY-DELIVERABLES.md | ✅                               |

### Installer

| Artifact                    | Present                                                                                    |
| --------------------------- | ------------------------------------------------------------------------------------------ |
| `docs/gre-atlas-release.md` | ✅                                                                                         |
| `qt/installer/` templates   | ✅                                                                                         |
| Built macOS `.dmg` (local)  | ✅ `out/installer/dist/anki-26.05-mac-apple.dmg` (214 MB, 2026-07-05 14:35, not committed) |
| Installer build log         | ✅ [logs/wednesday-installer-build.log](./logs/wednesday-installer-build.log)              |

### Sync verification (automated)

```bash
cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect  # pass (re-verified 2026-07-05)
```

## Requirement status

| #  | Requirement          | Status      | Primary artifact                                                                    |
| -- | -------------------- | ----------- | ----------------------------------------------------------------------------------- |
| 1  | AI evaluation        | **EXISTS**  | `results/gre-atlas-ai-eval.md`, `AI.md`, `EVALUATION.md` §3                         |
| 2  | Baseline comparison  | **EXISTS**  | `results/gre-atlas-ai-eval.md` § Benchmark comparison                               |
| 3  | Desktop review       | **PARTIAL** | `DEMO-CHECKLIST.md` §7; recording `06-desktop-review.mov` **missing**               |
| 4  | Phone review         | **PARTIAL** | `mobile/ios/DEMO.md`; recording `07-phone-review.mov` **missing**                   |
| 5  | Desktop–phone sync   | **PARTIAL** | `SYNC-VERIFICATION.md`, passing `friday_sync_loop` test; live recording **missing** |
| 6  | Offline sync         | **PARTIAL** | Same as #5 + `SYNC-VERIFICATION.md` §3; automated test covers merge                 |
| 7  | Three scores         | **PARTIAL** | Model docs + UI code; `05-readiness-scores.png` **missing**                         |
| 8  | Give-up rule         | **PARTIAL** | `abstention.rs`, eval § Abstention; `02-abstention-progress.png` **missing**        |
| 9  | Uncertainty (ranges) | **PARTIAL** | `docs/models/memory-model.md`; `09-unlocked-scores.png` **missing**                 |
| 10 | Evidence             | **PARTIAL** | `GreEvidenceCard` UI; screenshots `05`, `06` **missing**                            |
| 11 | Installer            | **EXISTS**  | `docs/gre-atlas-release.md`, `qt/installer/`, local `out/installer/dist/*.dmg`      |

## Human-only gaps

1. Capture screenshots `01`–`07` (+ recommended `09`) per [screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md](./screenshots/REQUIREMENT-SCREENSHOT-CHECKLIST.md) — placeholders in [screenshots/pending/](./screenshots/pending/).
2. Record `.mov` files per [recordings/REQUIREMENT-RECORDING-SCRIPTS.md](./recordings/REQUIREMENT-RECORDING-SCRIPTS.md) — placeholders in [recordings/pending/](./recordings/pending/).
3. Complete personal checklist: [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md).

```bash
cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_desktop_phone_offline_reconnect -- --nocapture 2>&1 | tee -a docs/gre-atlas-submission/results/friday-verification-2026-07-05.log
```
