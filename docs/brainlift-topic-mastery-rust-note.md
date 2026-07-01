# BrainLift Topic Mastery Engine — Rust Change Note

## Why this belongs in Rust

The Topic Mastery Engine aggregates FSRS retrievability across up to 50,000 cards. Doing this in Python would require either tens of thousands of RPC round-trips or ad-hoc SQL via dbproxy. Rust provides:

- One bulk SQL join (`cards` + `notes.tags`) over the search table
- Direct reuse of FSRS retrievability math from existing stats code
- The same protobuf RPC on desktop, TypeScript web views, and iOS (no Python on mobile)
- Read-only access with no undo/sync side effects

## What it does

`StatsService.TopicMastery` rolls card-level FSRS state into per-topic metrics for notes tagged with a prefix (default `gre::`). It returns:

- Per topic: total/studied/mastered cards, average retrievability with a 95% range
- Summary: coverage ratio, overall memory estimate, and an honest abstain flag when data is insufficient

FSRS remains the memory model; this layer only **reads** scheduling state.

## Files touched (upstream merge difficulty)

| File                                                    | Change                    | Merge risk       |
| ------------------------------------------------------- | ------------------------- | ---------------- |
| `proto/anki/stats.proto`                                | New RPC + messages        | Low — additive   |
| `rslib/src/stats/mastery.rs`                            | New module                | None — new file  |
| `rslib/src/stats/mod.rs`                                | `mod mastery`             | Low              |
| `rslib/src/stats/service.rs`                            | One RPC handler           | Low              |
| `rslib/src/storage/card/cards_with_tags_for_search.sql` | New query                 | None — new file  |
| `rslib/src/storage/card/mod.rs`                         | `CardWithTags` accessor   | Low              |
| `pylib/anki/collection.py`                              | `topic_mastery()` wrapper | Low              |
| `pylib/tests/test_stats.py`                             | One test                  | Low              |
| `qt/aqt/mediasrv.py`                                    | Expose RPC + Svelte page  | Low              |
| `ts/routes/(gre)/dashboard/*`                           | GRE dashboard page        | None — new files |

**Overall merge difficulty: low–medium.** All changes are additive. The only shared hot path is `stats.proto` service ordering (append-only RPC).

## Give-up rule (embedded in response)

`sufficient_data = true` only when:

1. FSRS is enabled
2. At least 200 studied cards in search scope
3. Topic coverage ratio ≥ 50% (topics with studied cards / topics with any cards)

Otherwise `abstain_reason` explains why readiness scores must not be shown.

## Benchmark

Run the included manual benchmark (1000 cards):

```bash
cd rslib && cargo test topic_mastery_benchmark -- --ignored --nocapture
```

For a full 50k-card deck, import a large collection and run the same RPC via Python:

```python
import time
from anki.collection import Collection
col = Collection("/path/to/collection.anki2")
start = time.perf_counter()
col.topic_mastery(search="")
print(f"elapsed: {time.perf_counter() - start:.3f}s")
```

**Target:** p95 < 500ms on 50,000 cards (Speedrun dashboard refresh target). The 1000-card dev benchmark completed in **~2.6ms** (debug build, M-series Mac).

Record your numbers in the project README when testing on the reference deck.

## GRE deck convention

Tag notes with hierarchical topics:

```
gre::quant::algebra
gre::verbal::text_completion
```

## Tests

- **Rust:** 5 unit tests in `rslib/src/stats/mastery.rs` (empty collection, single topic, multi-tag, mastery threshold, storage join)
- **Python:** `test_topic_mastery` in `pylib/tests/test_stats.py`

Run:

```bash
cd rslib && cargo test stats::mastery::test
./ninja pylib && out/pyenv/bin/pytest pylib/tests/test_stats.py::test_topic_mastery
```
