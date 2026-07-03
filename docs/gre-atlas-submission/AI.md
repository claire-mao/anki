# GRE Atlas AI question generation

Minimal AI-assisted GRE practice question generation for the GRE Atlas Speedrun rubric. This is **not** a chat assistant, RAG pipeline, or vector database integration.

## What is implemented

| Component                                   | Location                                            |
| ------------------------------------------- | --------------------------------------------------- |
| Named source excerpts                       | `rslib/src/gre_atlas/questions/source.rs`           |
| Template generation + confidence filter     | `rslib/src/gre_atlas/questions/ai_gen.rs`           |
| Gold evaluation set (50 verified questions) | `rslib/src/gre_atlas/questions/gold_eval_set.json`  |
| Baseline keyword retrieval + eval report    | `rslib/src/gre_atlas/ai_eval.rs`                    |
| Persistence (attribution columns)           | `greatlas.db` schema v4, `bl_question`              |
| RPC                                         | `GenerateQuestion`, `GenerateBrainLiftAiEvalReport` |

## Named source

All generated questions attribute to a **single named source**:

**`ETS Official GRE Prep Material`**

Each generation records:

- `source_name` — always the named source above
- `source_section` — section heading within that source (e.g. `Quantitative Reasoning — Linear equations`)
- `generated_at_secs` — Unix timestamp when generation ran

Bundled excerpts live in-repo (`source.rs`); nothing is fetched from the network at generation time.

## Generation pipeline

1. Validate `topic_id` against the GRE catalog (`GreCatalog`).
2. Map topic → source section (`source_section_for_topic`).
3. Build a deterministic template question from the section excerpt.
4. Score confidence:

   `confidence = 0.3 × topic_match + 0.4 × keyword_coverage + 0.3 × template_validity`

5. **Reject** if `confidence < 0.55` (`MIN_GENERATION_CONFIDENCE`).
6. Optionally persist to `greatlas.db` when `persist=true`.

### Confidence threshold

| Constant                    | Value    | Rationale                                                                                                                                    |
| --------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `MIN_GENERATION_CONFIDENCE` | **0.55** | Requires topic-section alignment plus reasonable keyword coverage; rejects empty or mismatched drafts rather than storing low-quality items. |

Rejected generations return `accepted=false` with `rejection_reason` and attribution metadata (including sub-threshold confidence).

## Optional LLM path (not used in eval)

An external LLM API is **not required**. The default path is template generation from bundled ETS excerpts.

To experiment with an LLM provider later, set:

```bash
export GRE_ATLAS_OPENAI_API_KEY=sk-...
```

No LLM call is made unless that variable is set **and** a provider hook is wired in application code. The gold-set eval intentionally uses the deterministic template path only so results are reproducible.

## Baseline comparison

The eval harness compares two approaches on the static gold set:

### 1. Keyword retrieval baseline

For each gold question, score every bundled source section by keyword overlap with the gold question's keyword list. Pick the best section.

Metrics:

- **topic_match_rate** — retrieved section's catalog topic equals gold topic
- **mean_keyword_recall** — matched keywords / gold keywords

### 2. Template generation

For each gold question's topic, run `generate_question_for_topic` with a fixed timestamp (`1700000000`).

Metrics:

- **acceptance_rate / rejection_rate** — share passing the 0.55 confidence gate
- **topic_match_rate** — accepted draft topic equals gold topic
- **mean_keyword_overlap** — gold keywords found in generated stem

## Gold set

File: `rslib/src/gre_atlas/questions/gold_eval_set.json`

- Label: `gre_atlas_gold_eval_v1`
- `"verified": true` — manually authored/ reviewed for eval (not live learner data)
- 50 questions across quant, verbal, and AWA catalog leaves

## Assumptions

- Template generation is sufficient for a **minimal honest** AI rubric demo; it is not GPT-class open-ended generation.
- Keyword baseline is bag-of-words overlap, not semantic search.
- Gold-set topic match for generation is high when every gold topic has a template (by design); keyword overlap is the more informative generation metric.
- Seeded static questions (`seed_gre.json`) have no attribution columns (NULL); only AI-generated rows populate them.

## Limitations

- No chat UI, no RAG, no vector DB.
- No automated factual verification against ETS PDFs — templates are hand-authored from public overview material.
- Eval does not measure learner outcomes or post-generation item response theory.
- Optional LLM integration is documented but not enabled in the default eval path.

## How to run

Build pylib first:

```bash
just build
# or: just check
```

### Generate a question (RPC / pylib)

```python
from anki.collection import Collection
from anki.gre_atlas import generate_question

col = Collection("/path/to/collection.anki2")
resp = generate_question(col, topic_id="gre::quant::algebra::linear", persist=True)
print(resp.accepted, resp.confidence, resp.attribution)
col.close()
```

### Run AI eval

```bash
just eval-gre-atlas-ai
```

Or directly:

```bash
PYTHONPATH=out/pylib out/pyenv/bin/python scripts/eval/gre_atlas_ai_eval.py \
  --output-dir docs/gre-atlas-submission/results
```

The eval does not read collection-specific data; `--collection` defaults to `:memory:`.

### Outputs

| File                                                       | Contents                 |
| ---------------------------------------------------------- | ------------------------ |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.json` | Machine-readable metrics |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md`   | Human-readable summary   |

### Unit tests

```bash
just test-rust
```

Relevant modules:

- `rslib/src/gre_atlas/questions/ai_gen.rs`
- `rslib/src/gre_atlas/questions/mod.rs`
- `rslib/src/gre_atlas/ai_eval.rs`
- `rslib/src/gre_atlas/storage/mod.rs` (schema v4 migration)

## Environment variables

| Variable                   | Required | Purpose                                                          |
| -------------------------- | -------- | ---------------------------------------------------------------- |
| _(none)_                   | —        | Default template generation works offline                        |
| `GRE_ATLAS_OPENAI_API_KEY` | Optional | Reserved for future optional LLM provider hook; not used by eval |

## Schema

`greatlas.db` schema v4 adds to `bl_question`:

```sql
source_name TEXT
source_section TEXT
generated_at_secs INTEGER
generation_confidence REAL
```

Existing profiles migrate automatically on next open via `upgrade_3_to_4.sql`.
