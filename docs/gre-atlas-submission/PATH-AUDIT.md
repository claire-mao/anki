# Path audit — submission package

**Audit date:** 2026-07-05\
**Auditor:** submission manager (automated link check + artifact inventory)

Regenerate this audit after adding screenshots/recordings:

```bash
cd docs/gre-atlas-submission
python3 - <<'PY'
import re
from pathlib import Path
root = Path(".")
broken = []
for md in root.rglob("*.md"):
    for m in re.finditer(r'\]\((\./[^)#]+|\.\./[^)#]+)\)', md.read_text()):
        rel = m.group(1).split('#')[0]
        if not rel: continue
        if not (md.parent / rel).resolve().exists():
            broken.append((str(md), rel))
print("Broken links:", len(broken))
for b in broken: print(" ", b)
PY
```

---

## Core documents (required)

| File                        | Exists |
| --------------------------- | ------ |
| `INSTALL.md`                | ✅     |
| `AI.md`                     | ✅     |
| `ARCHITECTURE.md`           | ✅     |
| `SUBMISSION.md`             | ✅     |
| `SUBMISSION-CHECKLIST.md`   | ✅     |
| `RELEASE-CHECKLIST.md`      | ✅     |
| `SYNC-VERIFICATION.md`      | ✅     |
| `WEDNESDAY-PHONE-REVIEW.md` | ✅     |
| `FRIDAY-DELIVERABLES.md`    | ✅     |

---

## Folders

| Folder         | Exists | Notes                                             |
| -------------- | ------ | ------------------------------------------------- |
| `results/`     | ✅     | Eval + benchmark artifacts                        |
| `logs/`        | ✅     | `wednesday-*.log`                                 |
| `screenshots/` | ✅     | 2 PNGs + `pending/` placeholders                  |
| `recordings/`  | ✅     | Scripts + `pending/` placeholders (no `.mov` yet) |

---

## Results inventory

| File                                         | Exists |
| -------------------------------------------- | ------ |
| `results/gre-atlas-eval.{json,md}`           | ✅     |
| `results/performance-eval.md`                | ✅     |
| `results/gre-atlas-ai-eval.{json,md}`        | ✅     |
| `results/gre-atlas-benchmark.{json,md,csv}`  | ✅     |
| `results/friday-verification-2026-07-05.log` | ✅     |

---

## Screenshots

| File                                                  | Exists                      |
| ----------------------------------------------------- | --------------------------- |
| `screenshots/08-eval-report.png`                      | ✅                          |
| `screenshots/10-benchmark-output.png`                 | ✅                          |
| `screenshots/01-gre-home.png` … `07-congrats-cta.png` | ❌ → `screenshots/pending/` |
| `screenshots/09-unlocked-scores.png`                  | ❌ → `screenshots/pending/` |

---

## Recordings

| File               | Exists                                             |
| ------------------ | -------------------------------------------------- |
| `recordings/*.mov` | ❌ (expected — gitignored) → `recordings/pending/` |

---

## Commands verified against `justfile`

| Document command                                                    | Valid                                              |
| ------------------------------------------------------------------- | -------------------------------------------------- |
| `just build`                                                        | ✅                                                 |
| `just check`                                                        | ✅                                                 |
| `just run`                                                          | ✅                                                 |
| `just test-py` / `just test-ts`                                     | ✅                                                 |
| `just eval-gre-atlas <collection>`                                  | ✅                                                 |
| `just eval-gre-atlas-ai`                                            | ✅                                                 |
| `just bench-gre-atlas --synthetic-cards N`                          | ✅                                                 |
| `tools/build-installer`                                             | ✅ (not a `just` recipe; documented in INSTALL.md) |
| `cargo test -p anki gre_atlas::sync_http::test::friday_sync_loop_*` | ✅                                                 |

---

## External paths referenced from this package

| Path                                          | Exists                 |
| --------------------------------------------- | ---------------------- |
| `docs/development.md`                         | ✅                     |
| `docs/gre-atlas-release.md`                   | ✅                     |
| `docs/models/*.md`                            | ✅                     |
| `mobile/ios/DEMO.md`                          | ✅                     |
| `mobile/ios/README.md`                        | ✅                     |
| `scripts/eval/README.md`                      | ✅                     |
| `proto/anki/brainlift.proto`                  | ✅                     |
| `out/installer/dist/anki-26.05-mac-apple.dmg` | ✅ (local, gitignored) |

---

## Last link check

**0 broken relative links** in `docs/gre-atlas-submission/` after 2026-07-05 path fixes.
