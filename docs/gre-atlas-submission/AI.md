# GRE Atlas AI question generation

Optional AI-assisted GRE practice question generation and post-answer explanations for the GRE Atlas Speedrun rubric. AI **enhances** the experience but is **never required** — the app runs fully offline with deterministic templates when no API key is configured.

## What is implemented

| Component | Location |
| --- | --- |
| Named source excerpts | `rslib/src/gre_atlas/questions/source.rs` |
| Deterministic template generation (fallback) | `rslib/src/gre_atlas/questions/ai_gen.rs` |
| Optional LLM client (env-gated) | `rslib/src/gre_atlas/questions/llm.rs` |
| Generator orchestration + silent fallback | `rslib/src/gre_atlas/questions/generator.rs` |
| Pre-exposure eval gate | `rslib/src/gre_atlas/questions/eval_pipeline.rs` |
| Post-answer explanations | `rslib/src/gre_atlas/questions/explanation.rs` |
| Provenance / evaluation status types | `rslib/src/gre_atlas/questions/metadata.rs` |
| Gold evaluation set (50 verified questions) | `rslib/src/gre_atlas/questions/gold_eval_set.json` |
| Eval report (keyword baseline + rejection pipeline) | `rslib/src/gre_atlas/ai_eval.rs` |
| Persistence (attribution + eval log) | `greatlas.db` schema v5, `bl_question`, `bl_generation_eval` |
| RPC | `GenerateQuestion`, `ExplainAnswer`, `GenerateBrainLiftAiEvalReport` |
| Web practice UI | `ts/routes/(gre)/practice/+page.svelte` |

## Offline-first design

1. **Default path (no env vars):** deterministic template generation from bundled ETS excerpts. No network I/O.
2. **Optional LLM path:** enabled only when `GRE_ATLAS_OPENAI_API_KEY` is set. Tries LLM generation first, runs the eval gate, and on *any* failure or rejection falls back to templates **without surfacing an error**.
3. **Post-answer explanations:** `ExplainAnswer` RPC tries the LLM when enabled; otherwise builds a structured template explanation. Transport failures are swallowed in the UI.

When the fallback is used, the response includes the exact note:

**`Generated using offline templates.`**

(Wording is shared between Rust `OFFLINE_TEMPLATE_NOTE` and TypeScript `OFFLINE_TEMPLATE_NOTE`.)

## Named source

All generated questions attribute to a **single named source**:

**`ETS Official GRE Prep Material`**

Each generation records:

- `source_name` — always the named source above
- `source_section` — section heading within that source (e.g. `Quantitative Reasoning — Linear equations`)
- `source_document` — specific excerpt/section id used for grounding
- `generated_at_secs` — Unix timestamp when generation ran
- `model_version` — real model id (e.g. `gpt-4o-mini`) or `template_v1` for offline templates
- `provenance` — `ai_generated` or `offline_template`
- `evaluation_status` — e.g. `approved`, `rejected_hallucination`, `rejected_duplicate`, `rejected_unsupported`
- `generation_confidence` — grounding/confidence score stored on the question row

Bundled excerpts live in-repo (`source.rs`); the LLM path uses them as the only grounding material.

## Generation pipeline

### Orchestrator (`generate_with_fallback`)

1. Validate `topic_id` against the GRE catalog (`GreCatalog`).
2. Map topic → source section (`source_section_for_topic`).
3. **If LLM enabled:** build a grounded prompt from the excerpt + exemplars, call the provider, parse JSON into a draft.
4. **Eval gate** (see below) — reject hallucinated, duplicate, or unsupported candidates.
5. **On approval:** persist/serve the AI question with `provenance = ai_generated`.
6. **On any failure/rejection or when AI disabled:** run the existing deterministic template path with `provenance = offline_template` and attach `Generated using offline templates.`

Template confidence scoring (unchanged):

`confidence = 0.3 × topic_match + 0.4 × keyword_coverage + 0.3 × template_validity`

**Reject** template-only drafts if `confidence < 0.55` (`MIN_GENERATION_CONFIDENCE`).

### Pre-exposure evaluation gate

Before a generated candidate reaches the practice bank:

| Gate | Rejects when |
| --- | --- |
| Hallucination | Answer not among choices, malformed item, or structurally invalid |
| Unsupported | Grounding score (keyword overlap with source/gold) below `0.15` |
| Duplicate | Jaccard similarity to an existing bank stem ≥ `0.85` |
| Approved | Passes all three |

Rejected AI candidates are **not** persisted to `bl_question`. Outcomes are logged to `bl_generation_eval` with status, reason, model version, and confidence.

## Post-answer explanation (`ExplainAnswer`)

After the learner submits an answer, the practice page calls `explainAnswer` (best-effort, non-blocking). The response includes:

- Summary of why the correct answer is correct
- Per-choice reasoning (correct + each distractor)
- Source citation (name, section, excerpt)
- Provenance note when templates were used

Failures never block the result panel; the plain text from `recordAttempt` remains visible.

## Baseline comparison (eval harness)

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

### 3. Rejection pipeline (deterministic negatives)

Crafted negative drafts exercise each rejection rule (hallucination, duplicate, unsupported) plus grounded approvals. Reported in `rejection_pipeline` section of the eval JSON.

## Gold set

File: `rslib/src/gre_atlas/questions/gold_eval_set.json`

- Label: `gre_atlas_gold_eval_v1`
- `"verified": true` — manually authored/reviewed for eval (not live learner data)
- 50 questions across quant, verbal, and AWA catalog leaves

## Assumptions

- Deterministic templates are always available and sufficient for a fully offline demo.
- The LLM path is OpenAI-compatible (`/v1/chat/completions`); other providers work via `GRE_ATLAS_OPENAI_BASE_URL`.
- Eval uses the deterministic template path for reproducibility; the rejection pipeline validates gate logic independently of live LLM calls.
- Seeded static questions (`seed_gre*.json`) may have NULL attribution columns; generated rows populate them.

## Limitations

- No chat UI, no RAG, no vector DB.
- No automated factual verification against ETS PDFs — templates and prompts are hand-authored from public overview material.
- Eval does not measure learner outcomes or post-generation item response theory.
- iOS companion uses plain `recordAttempt` explanations today; structured `ExplainAnswer` is wired on the web practice page.

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
print(resp.accepted, resp.confidence, resp.provenance, resp.provenance_note)
col.close()
```

### Explain an answer (RPC / pylib)

```python
from anki.gre_atlas import explain_answer

resp = explain_answer(col, question_id="...", selected_answer="42")
print(resp.explanation.summary, resp.explanation.provenance_note)
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

| File | Contents |
| --- | --- |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.json` | Machine-readable metrics |
| `docs/gre-atlas-submission/results/gre-atlas-ai-eval.md` | Human-readable summary |

### Unit tests

```bash
just test-rust    # cargo test -p anki gre_atlas
just test-ts      # vitest, includes practice-presentation tests
```

Relevant modules:

- `rslib/src/gre_atlas/questions/generator.rs`
- `rslib/src/gre_atlas/questions/eval_pipeline.rs`
- `rslib/src/gre_atlas/questions/explanation.rs`
- `rslib/src/gre_atlas/questions/llm.rs`
- `rslib/src/gre_atlas/ai_eval.rs`
- `rslib/src/gre_atlas/storage/mod.rs` (schema v5 migration)

## Environment variables

| Variable | Required | Purpose |
| --- | --- | --- |
| _(none)_ | — | Default offline template generation and template explanations |
| `GRE_ATLAS_OPENAI_API_KEY` | Optional | Enables real LLM generation and explanations |
| `GRE_ATLAS_OPENAI_BASE_URL` | Optional | Override API base (default `https://api.openai.com/v1`) |
| `GRE_ATLAS_OPENAI_MODEL` | Optional | Override model id (default `gpt-4o-mini`) |
| `GRE_ATLAS_OPENAI_TIMEOUT_SECS` | Optional | Request timeout in seconds (default `20`) |

## Schema

`greatlas.db` schema v5 adds to `bl_question`:

```sql
source_document TEXT
model_version TEXT
provenance TEXT
evaluation_status TEXT
```

And a new audit table:

```sql
CREATE TABLE bl_generation_eval (
  id INTEGER PRIMARY KEY,
  candidate_id TEXT NOT NULL,
  topic TEXT NOT NULL,
  model_version TEXT NOT NULL,
  provenance TEXT NOT NULL,
  status TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  confidence REAL,
  evaluated_at_secs INTEGER NOT NULL
);
```

Schema v4 columns (`source_name`, `source_section`, `generated_at_secs`, `generation_confidence`) remain unchanged. Existing profiles migrate automatically on next open via `upgrade_4_to_5.sql`.
