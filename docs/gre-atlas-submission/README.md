# GRE Atlas submission — package index

**Grader entry point.** One-page summary: [SUBMISSION.md](./SUBMISSION.md). **Your remaining tasks:** [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md).

**Commit (2026-07-05 verification):** `1323b37859cc9baaa5a8a1a850a20fe76d3c0e8f`

---

## Required core documents

| Document                                                 | Purpose                                                          |
| -------------------------------------------------------- | ---------------------------------------------------------------- |
| [SUBMISSION.md](./SUBMISSION.md)                         | One-page grader quick start                                      |
| [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md)     | **Your** pre-upload checklist (screenshots, recordings, install) |
| [PATH-AUDIT.md](./PATH-AUDIT.md)                         | Link + artifact path verification (0 broken links)               |
| [INSTALL.md](./INSTALL.md)                               | Desktop installer + clean-machine smoke test                     |
| [AI.md](./AI.md)                                         | AI design, eval gate, attribution, AI-off mode                   |
| [ARCHITECTURE.md](./ARCHITECTURE.md)                     | BrainLiftService diagram + RPC table                             |
| [RELEASE-CHECKLIST.md](./RELEASE-CHECKLIST.md)           | Full release / submission verification matrix                    |
| [SYNC-VERIFICATION.md](./SYNC-VERIFICATION.md)           | Sync scenarios + conflict documentation                          |
| [WEDNESDAY-PHONE-REVIEW.md](./WEDNESDAY-PHONE-REVIEW.md) | iOS simulator / phone recording steps                            |
| [FRIDAY-DELIVERABLES.md](./FRIDAY-DELIVERABLES.md)       | Friday bundle index + eval numbers                               |

---

## Evidence folders

| Folder                         | Contents                             | Regenerate                                                                         |
| ------------------------------ | ------------------------------------ | ---------------------------------------------------------------------------------- |
| [results/](./results/)         | Eval, benchmark, AI eval JSON/MD     | `just eval-gre-atlas`, `just eval-gre-atlas-ai`, `just bench-gre-atlas`            |
| [logs/](./logs/)               | Raw `wednesday-*.log` command output | Re-run commands in [build.md](./build.md), [tests.md](./tests.md)                  |
| [screenshots/](./screenshots/) | UI PNG captures                      | [SCREENSHOTS.md](./SCREENSHOTS.md), [screenshots/pending/](./screenshots/pending/) |
| [recordings/](./recordings/)   | Demo `.mov` files (gitignored)       | [RECORDINGS.md](./RECORDINGS.md), [recordings/pending/](./recordings/pending/)     |

---

## Wednesday automated proof

| Document                                                   | Purpose                                        |
| ---------------------------------------------------------- | ---------------------------------------------- |
| [build.md](./build.md)                                     | `just build`, iOS simulator, installer paths   |
| [tests.md](./tests.md)                                     | cargo / pytest / vitest results                |
| [artifacts.md](./artifacts.md)                             | DMG, `.app`, mobile bridge sizes               |
| [release.md](./release.md)                                 | Release checklist + install/recording overview |
| [wednesday-release-proof.md](./wednesday-release-proof.md) | Requirement matrix                             |
| [wednesday-test-output.txt](./wednesday-test-output.txt)   | Combined test command output                   |

---

## Evaluation & demo

| Document                                       | Purpose                      |
| ---------------------------------------------- | ---------------------------- |
| [EVALUATION.md](./EVALUATION.md)               | Eval + benchmark commands    |
| [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md)       | Live demo script             |
| [SCREENSHOTS.md](./SCREENSHOTS.md)             | Screenshot filenames         |
| [RECORDINGS.md](./RECORDINGS.md)               | Recording filenames          |
| [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md) | Requirement → evidence map   |
| [FEATURE-INDEX.md](./FEATURE-INDEX.md)         | Feature → source → test      |
| [PERFORMANCE-MODEL.md](./PERFORMANCE-MODEL.md) | Performance eval methodology |

---

## Quick start (reviewers)

```bash
just check          # optional; needs CONTRIBUTORS entry
just run            # GRE shell at /home
just eval-gre-atlas mobile/ios/GREAtlasCompanion/Resources/DemoBundle/collection.anki2
just eval-gre-atlas-ai
just bench-gre-atlas --synthetic-cards 10000
```

---

## External references (outside this folder)

| Topic              | Path                                                                                                           |
| ------------------ | -------------------------------------------------------------------------------------------------------------- |
| Upstream dev setup | [../development.md](../development.md)                                                                         |
| Release overview   | [../gre-atlas-release.md](../gre-atlas-release.md)                                                             |
| Model docs         | [../models/](../models/)                                                                                       |
| Eval harness       | [../../scripts/eval/README.md](../../scripts/eval/README.md)                                                   |
| iOS companion      | [../../mobile/ios/README.md](../../mobile/ios/README.md), [../../mobile/ios/DEMO.md](../../mobile/ios/DEMO.md) |
| Protobuf API       | [../../proto/anki/brainlift.proto](../../proto/anki/brainlift.proto)                                           |

---

## Screenshot & recording status (2026-07-05)

| Asset type       | Present                                                           | Missing                                                                    |
| ---------------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Screenshots      | `08-eval-report.png`, `10-benchmark-output.png`                   | `01`–`07`, `09`, `11` — see [screenshots/pending/](./screenshots/pending/) |
| Recordings       | _(none in repo)_                                                  | `01`–`08` `.mov` — see [recordings/pending/](./recordings/pending/)        |
| Installer binary | `out/installer/dist/anki-26.05-mac-apple.dmg` (local, gitignored) | Clean-machine recording                                                    |

**Do not fabricate** missing PNG/MOV files. Follow [SUBMISSION-CHECKLIST.md](./SUBMISSION-CHECKLIST.md).
