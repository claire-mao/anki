# GRE Atlas — build instructions

Follow upstream Anki prerequisites first: [../development.md](../development.md#building-from-source).

## Clean checkout (recommended for grading)

```bash
git clone <repository-url> anki
cd anki

# Full format, build, and test
just check
```

`just check` runs formatting, builds pylib + Qt + TypeScript, and executes the main test suite. This is the authoritative green-build gate.

## Run the desktop app

```bash
just run
```

- Builds pylib and Qt if needed, then launches Anki with debugging enabled.
- On collection open, the app enters the **GRE main-window shell** (`greDashboard` state) at **`/home`**.
- GRE pages are served at `http://127.0.0.1:40000/_anki/pages/` during development.

Optimized dev build:

```bash
just run-optimized
```

## Targeted builds

| Command          | When to use                                      |
| ---------------- | ------------------------------------------------ |
| `just build`     | pylib + Qt only (`./ninja pylib qt`)             |
| `cargo check`    | Rust compile check (from repo root or `rslib/`)  |
| `./ninja pylib`  | Python/Rust bridge after Rust changes            |
| `just test-rust` | GRE Atlas + all Rust unit tests                  |
| `just test-py`   | Includes `pylib/tests/test_gre_atlas.py`         |
| `just test-ts`   | Svelte/TypeScript checks                         |
| `just web-watch` | Live reload for `ts/` while `just run` is active |

## After protobuf changes

```bash
touch proto/anki/brainlift.proto   # or stats.proto
./ninja rslib:proto ts:generated:proto pylib:anki:proto
just build
```

Generated code lives under `out/` — do not edit by hand.

## Evaluation & benchmark (no UI)

Requires pylib built:

```bash
just eval-gre-atlas /path/to/collection.anki2
just bench-gre-atlas --synthetic-cards 10000
```

See [EVALUATION.md](./EVALUATION.md).

## Installer (optional)

Same as upstream Anki:

```bash
tools/build-installer
# artifacts: out/installer/dist/
```

Smoke test after install: launch app, confirm GRE shell loads, answer one practice question, confirm dashboard updates.

## Common failures

| Symptom                     | Fix                                                                                                                  |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `ModuleNotFoundError: anki` | Run `just build` or `./ninja pylib` first; use `just run` / `just eval-gre-atlas` (they set `PYTHONPATH=out/pylib`). |
| GRE pages 403 in browser    | Use embedded Qt webview via `just run`, not an external browser tab without API access.                              |
| Stale generated bindings    | Re-run proto ninja targets after `.proto` edits.                                                                     |
| `just check` timeout        | Ensure network for first-time dependency fetch; see [../development.md](../development.md).                          |

## Platform notes

- **macOS / Linux:** Standard `just` recipes.
- **Windows:** Use `just` recipes from repo root; paths are handled in the justfile.
