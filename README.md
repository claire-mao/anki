# Anki

[![Build Status](https://github.com/ankitects/anki/actions/workflows/ci.yml/badge.svg)](https://github.com/ankitects/anki/actions/workflows/ci.yml)
[![Documentation](https://img.shields.io/badge/docs-dev--docs.ankiweb.net-blue)](https://dev-docs.ankiweb.net)

This repo contains the source code for the computer version of
[Anki](https://apps.ankiweb.net).

## About

Anki is a spaced repetition program. Please see the [website](https://apps.ankiweb.net) to learn more.

This fork additionally ships **GRE Atlas** — a graduate-exam study layer built on Anki's spaced-repetition core.

**What it does:** GRE Atlas turns a tagged GRE deck into a predictive study dashboard. It reports three _separately modelled_ signals — **Memory** (FSRS retrievability), **Performance** (practice-question accuracy), and **Readiness** (a calibrated composite) — each with confidence intervals and an **abstention rule** that withholds a score until there is enough evidence. Predictions are validated with **held-out, reproducible evaluation**.

**How it's built:** a single Rust engine (`rslib/src/gre_atlas/`) powers both the **desktop** app (Svelte UI embedded in Qt) and the **iOS companion** (`GREAtlasCompanion`, which shares that engine over a C FFI). Practice attempts live in a sidecar `greatlas.db` that syncs independently of AnkiWeb collection sync. Question generation is **offline and deterministic** (template-based; no external LLM or API keys).

| Audience                        | Start here                                                                                                                                                                             |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Graders / submission review** | [GRE Atlas submission package](./docs/gre-atlas-submission/README.md) — build, demo, eval, grading checklist                                                                           |
| **Developers**                  | [GRE Atlas release guide](./docs/gre-atlas-release.md) · [architecture](./docs/gre-atlas-architecture.md) · [development](./docs/development.md#gre-atlas-gre)                         |
| **Models & metrics**            | [Memory](./docs/models/memory-model.md) · [Performance](./docs/models/performance-model.md) · [Readiness](./docs/models/readiness-model.md) · [Eval harness](./scripts/eval/README.md) |

Quick start after clone:

```bash
just check      # format, build, test
just run        # launch Anki → GRE shell at /home
just eval-gre-atlas /path/to/collection.anki2   # read-only eval report
```

## Getting Started

### Contributing

Want to contribute to Anki? Check out the [Contribution Guidelines](./docs/contributing.md).

For more information on building and developing, please see [Development](./docs/development.md).

#### Contributors

The following people have contributed to Anki: [CONTRIBUTORS](./CONTRIBUTORS)

### Anki Betas

If you'd like to try development builds of Anki but don't feel comfortable
building the code, please see [Anki betas](https://betas.ankiweb.net/).

## License

Anki's license: [LICENSE](./LICENSE)
