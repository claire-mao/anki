# GRE Atlas — screenshot checklist

Place captured PNGs in [screenshots/](./screenshots/). Reference them from [README.md](./README.md) and [GRADING-CHECKLIST.md](./GRADING-CHECKLIST.md) once added.

## Required captures

| # | Filename                     | What to show                                                | How to capture                                                                         |
| - | ---------------------------- | ----------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| 1 | `01-gre-home.png`            | GRE main shell at `/home` with header nav and daily mission | `just run` → default landing page                                                      |
| 2 | `02-abstention-progress.png` | Progress page with abstention checklist (sparse profile)    | Fresh dev profile, open Progress                                                       |
| 3 | `03-practice-question.png`   | Practice MCQ with answer choices                            | `/practice`, one question loaded                                                       |
| 4 | `04-study-plan.png`          | Ranked recommendations with factor explanations             | `/study-plan`                                                                          |
| 5 | `05-readiness-scores.png`    | Readiness page: Memory / Performance / Readiness cards      | `/readiness` (seeded profile preferred)                                                |
| 6 | `06-topic-detail.png`        | Topic drill-down page                                       | Click topic from Progress or study plan                                                |
| 7 | `07-congrats-cta.png`        | Congrats screen with Practice / Dashboard buttons           | Finish non-GRE deck review                                                             |
| 8 | `08-eval-report.png`         | Terminal or editor showing `gre-atlas-eval.md` sections     | `just eval-gre-atlas /path/to/collection.anki2` → `docs/gre-atlas-submission/results/` |

## Optional captures

| Filename                     | What to show                                         |
| ---------------------------- | ---------------------------------------------------- |
| `09-unlocked-scores.png`     | All three scores unlocked with confidence band       |
| `10-benchmark-output.png`    | `gre-atlas-benchmark.md` p50/p95 table               |
| `11-gre-modal-dashboard.png` | GRE modal dialog at `/dashboard` (from congrats CTA) |

## Tips

- Use a **dev profile** (File → Switch Profile) so personal data is not exposed.
- macOS: `Cmd+Shift+4` for region capture; name files exactly as above.
- For abstention demo, use an empty profile; for unlocked scores, seed GRE deck + 20+ practice attempts or run `PrepareDemoCollection` on mobile.

See [DEMO-CHECKLIST.md](./DEMO-CHECKLIST.md) for the live demo flow these screenshots support.
