# BrainLift Product Architecture

**Status:** Design proposal (Phase 1 not yet implemented)\
**Exam:** GRE\
**Positioning:** BrainLift is an AI-powered GRE study product. Anki’s scheduler and FSRS are the **memory engine** underneath. BrainLift is not “Anki with extra stats.”

---

## 1. Architecture document

### 1.1 Product principles

| Principle                            | Meaning                                                                                                            |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| **Memory ≠ Performance ≠ Readiness** | Three scores, three data sources, three computation paths                                                          |
| **FSRS is sacred**                   | Card ratings, `revlog`, scheduler state, and `cards.data` are never written from GRE practice                      |
| **Anki review stays Anki**           | `qt/aqt/reviewer.py` and the v3 scheduler path are unchanged                                                       |
| **BrainLift is a parallel workflow** | New navigation, new pages, new storage, new RPCs                                                                   |
| **Read-only memory reads**           | BrainLift may _read_ FSRS retrievability (existing `TopicMastery` work) but never back-propagates practice results |
| **Sync-ready from day one**          | Performance data lives in BrainLift-owned tables with USN/mod metadata, not in ad-hoc JSON blobs                   |

### 1.2 Layered system

```mermaid
flowchart TB
    subgraph brainlift_ui [BrainLift UI — product layer]
        BLNav["SvelteKit /brainlift/* shell"]
        Practice["Practice"]
        Readiness["Readiness dashboard"]
        StudyPlan["Study Plan placeholder"]
        ReviewEntry["Review entry / handoff"]
    end

    subgraph anki_ui [Anki UI — unchanged review]
        Reviewer["Reviewer qt/aqt/reviewer.py"]
        Congrats["Congrats screen ts/routes/congrats"]
    end

    subgraph bridge [Existing Anki bridge]
        MediaSrv["qt/aqt/mediasrv.py"]
        PyCol["pylib/anki/collection.py"]
        RSBridge["pylib/rsbridge"]
    end

    subgraph brainlift_engine [BrainLift engine — new]
        BLService["BrainLiftService protobuf"]
        BLStorage["brainlift.db SQLite"]
        BLQuestions["Question bank static → AI later"]
        ReadinessModel["Readiness predictor placeholder"]
    end

    subgraph anki_engine [Anki engine — read-only for BrainLift]
        Scheduler["rslib/scheduler + FSRS"]
        Stats["rslib/stats TopicMastery"]
        ColDB[("collection.anki2")]
    end

    BLNav --> MediaSrv
    ReviewEntry --> Reviewer
    Reviewer --> Congrats
    Congrats -->|"CTA only"| Practice
    MediaSrv --> PyCol --> RSBridge
    RSBridge --> BLService
    RSBridge --> Stats
    RSBridge --> Scheduler
    BLService --> BLStorage
    BLService --> BLQuestions
    Readiness --> BLService
    Readiness --> Stats
    Stats --> ColDB
    Scheduler --> ColDB
    BLStorage -.->|"future sync"| ColDB
```

### 1.3 Score ownership

| Score           | Owner                            | Inputs                                          | Phase 1                                      |
| --------------- | -------------------------------- | ----------------------------------------------- | -------------------------------------------- |
| **Memory**      | FSRS (via read-only aggregation) | Card retrievability, topic tags (`gre::`), reps | Wire to existing `StatsService.TopicMastery` |
| **Performance** | BrainLift                        | GRE question attempts (separate table)          | Hard-coded questions + attempt logging       |
| **Readiness**   | BrainLift predictor              | Memory + Performance + topic coverage           | Placeholder composite; real model later      |

**Contamination guard:** `RecordPerformanceAttempt` writes only to `brainlift.db`. No code path from BrainLift practice into `answer_card`, `revlog`, or card ease/FSRS state.

### 1.4 Relationship to existing work

The repo already contains a **Topic Mastery Engine** prototype:

- `proto/anki/stats.proto` → `TopicMastery`
- `rslib/src/stats/mastery.rs`
- `ts/routes/readiness/` dev page

**Design decision:** Keep `TopicMastery` as the **memory signal** inside Anki’s stats layer. BrainLift product APIs move to a new `BrainLiftService` in `proto/anki/brainlift.proto`. The Readiness dashboard calls both:

- `BrainLiftService.GetScores` (performance + readiness + coverage)
- `StatsService.TopicMastery` (memory), or a thin wrapper in `BrainLiftService` that delegates

The old `/readiness` dev route becomes `/brainlift/readiness` inside the BrainLift shell.

### 1.5 Desktop integration model

BrainLift runs **inside the same Anki process** (forked repo) but presents as its own section:

1. **Qt:** New menu item **BrainLift → Open BrainLift** opens a dedicated dialog/webview loading `/brainlift`.
2. **Review handoff:** Only touch the **congrats** finished screen — add a primary button “Continue to BrainLift Practice” that navigates to `/brainlift/practice`. No reviewer template/CSS/shortcut changes.
3. **Review tab inside BrainLift:** “Review” nav item calls existing Anki flow (`deckBrowser` → study) via bridge command, same as today’s “Study” — BrainLift does not reimplement card review.

---

## 2. Folder structure

New code lives in clearly named BrainLift namespaces. Anki upstream files get minimal, documented touch points.

```
anki/
├── proto/anki/
│   └── brainlift.proto              # NEW: BrainLiftService RPCs
│
├── rslib/src/brainlift/             # NEW: BrainLift domain
│   ├── mod.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── schema.sql               # brainlift.db DDL
│   │   ├── attempts.rs              # CRUD performance attempts
│   │   └── questions.rs             # static question bank (Phase 1)
│   ├── service.rs                   # protobuf impl
│   ├── scores.rs                    # memory/performance/readiness aggregation
│   └── questions/
│       └── seed_gre.json            # hard-coded GRE items (Phase 1)
│
├── pylib/anki/
│   └── brainlift.py                 # NEW: Collection.brainlift_* wrappers
│
├── qt/aqt/
│   ├── brainlift.py                 # NEW: BrainLiftDialog, menu registration
│   └── mediasrv.py                  # extend: sveltekit pages + RPC whitelist
│
├── ts/
│   ├── routes/brainlift/            # NEW: product shell + pages
│   │   ├── +layout.svelte           # BrainLift nav + branding
│   │   ├── +layout.ts               # i18n bootstrap
│   │   ├── review/+page.svelte      # launches Anki study (bridge)
│   │   ├── practice/
│   │   │   ├── +page.svelte
│   │   │   └── +page.ts
│   │   ├── readiness/+page.svelte
│   │   └── study-plan/+page.svelte  # placeholder
│   ├── lib/brainlift/               # NEW: shared components
│   │   ├── nav/
│   │   ├── practice/
│   │   └── scores/
│   └── routes/congrats/
│       └── CongratsPage.svelte      # ONLY Anki touch: BrainLift CTA button
│
├── docs/
│   ├── brainlift-architecture.md    # existing codebase map (reference)
│   └── brainlift-product-architecture.md  # this document
│
└── ftl/core/
    └── brainlift.ftl                # NEW: product strings (not Anki stats strings)
```

**Files intentionally not modified in Phase 1:** `reviewer.py`, `rslib/scheduler/*`, `rslib/src/storage/schema11.sql`, revlog writers.

---

## 3. Data flow diagram

### 3.1 End-to-end vertical slice (Phase 1)

```mermaid
sequenceDiagram
    actor Student
    participant BL as BrainLift UI
    participant MS as mediasrv
    participant Py as pylib Collection
    participant RS as rslib
    participant BDB as brainlift.db
    participant CDB as collection.anki2

    Student->>BL: Open BrainLift
    BL->>MS: GET /brainlift
    MS-->>BL: SvelteKit shell

    Student->>BL: Review tab → Study
    BL->>MS: bridge studyDeck
    Note over BL,CDB: Standard Anki reviewer unchanged
    Student->>CDB: FSRS answerCard → revlog

    Student->>BL: Congrats → Continue Practice
    BL->>MS: GET /brainlift/practice
    BL->>MS: POST BrainLiftService.ListQuestions
    MS->>Py: brainlift.list_questions()
    Py->>RS: BrainLiftService RPC
    RS-->>BL: hard-coded GRE questions

    Student->>BL: Submit answer
    BL->>MS: POST BrainLiftService.RecordAttempt
    MS->>Py: brainlift.record_attempt()
    Py->>RS: BrainLiftService RPC
    RS->>BDB: INSERT bl_performance_attempt
    Note over RS,CDB: No write to revlog or cards

    Student->>BL: Readiness tab
    BL->>MS: POST BrainLiftService.GetScores
    RS->>BDB: aggregate performance
    RS->>CDB: read-only TopicMastery / FSRS
    RS-->>BL: memory, performance, readiness placeholders
```

### 3.2 Score computation flow (target state)

```mermaid
flowchart LR
    subgraph inputs [Inputs]
        FSRS["FSRS card state cards.data"]
        Tags["notes.tags gre::"]
        Attempts["bl_performance_attempts"]
        Outline["GRE topic outline coverage"]
    end

    subgraph memory [Memory score]
        TM["TopicMastery aggregation"]
    end

    subgraph performance [Performance score]
        PA["Per-topic accuracy + latency"]
    end

    subgraph readiness [Readiness score]
        RP["Predictor model placeholder → ML later"]
    end

    FSRS --> TM
    Tags --> TM
    Attempts --> PA
    TM --> RP
    PA --> RP
    Outline --> RP
    Tags --> RP
```

---

## 4. Storage design

### 4.1 Two-database model

| Database           | Path                         | Purpose                                                    |
| ------------------ | ---------------------------- | ---------------------------------------------------------- |
| `collection.anki2` | `{profile}/collection.anki2` | Anki cards, notes, revlog, FSRS — **BrainLift read-only**  |
| `brainlift.db`     | `{profile}/brainlift.db`     | BrainLift-owned performance, sessions, future AI artifacts |

**Why sidecar DB:** Keeps GRE assessment data out of Anki’s sync merge logic until BrainLift sync is designed. Avoids schema 19 fights with upstream. Clear audit boundary for “never contaminate FSRS.”

**Collection binding:** `bl_meta.collection_path` stores absolute path to the active `collection.anki2` so mobile/desktop can validate pairing.

### 4.2 Schema (`brainlift.db`)

```sql
-- meta / migration
CREATE TABLE bl_meta (
  key TEXT PRIMARY KEY,
  val TEXT NOT NULL
);
-- keys: schema_version, collection_path, collection_crt

CREATE TABLE bl_question (
  id TEXT PRIMARY KEY,           -- stable slug, e.g. "gre-quant-pct-001"
  topic TEXT NOT NULL,           -- e.g. "gre::quant::arithmetic::percent"
  section TEXT NOT NULL,         -- verbal | quant
  format TEXT NOT NULL,          -- mcq | text_completion | rc | quant | data_interp
  stem TEXT NOT NULL,
  choices_json TEXT,             -- JSON array for MCQ; null for numeric entry
  correct_answer TEXT NOT NULL,
  explanation TEXT NOT NULL,
  difficulty REAL,               -- 0..1 optional
  usn INTEGER NOT NULL DEFAULT 0,
  mtime_secs INTEGER NOT NULL
);

CREATE TABLE bl_performance_attempt (
  id INTEGER PRIMARY KEY,
  question_id TEXT NOT NULL REFERENCES bl_question(id),
  topic TEXT NOT NULL,
  answered_at_secs INTEGER NOT NULL,
  answer TEXT NOT NULL,
  correct INTEGER NOT NULL,      -- 0/1
  response_time_ms INTEGER NOT NULL,
  confidence INTEGER,            -- 1-5 optional, null allowed
  session_id TEXT,               -- groups post-review practice
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL
);

CREATE INDEX ix_bl_attempt_topic ON bl_performance_attempt(topic);
CREATE INDEX ix_bl_attempt_time ON bl_performance_attempt(answered_at_secs);
CREATE INDEX ix_bl_attempt_usn ON bl_performance_attempt(usn);
```

**Phase 1 seeding:** On first open, Rust migrates schema and inserts ~5–10 hard-coded rows from `seed_gre.json` into `bl_question` if table empty.

### 4.3 Performance attempt record (protobuf ↔ row)

| Field         | Storage                                     |
| ------------- | ------------------------------------------- |
| question id   | `question_id`                               |
| topic         | `topic` (denormalized for fast aggregation) |
| timestamp     | `answered_at_secs`                          |
| answer        | `answer`                                    |
| correct       | `correct`                                   |
| response time | `response_time_ms`                          |
| confidence    | `confidence` (optional)                     |

### 4.4 Future sync (not Phase 1)

Design hooks only:

- `usn` / `mtime_secs` on mutable BrainLift rows mirror Anki’s pattern
- Future `BrainLiftSyncService` exchanges `bl_*` increments separately from AnkiWeb sync
- Mobile companion reads same protobuf RPCs; local `brainlift.db` on device
- Conflict rule (draft): latest `answered_at_secs` wins per attempt id; attempts are append-mostly

### 4.5 What we explicitly do not do

- No rows in `revlog` for practice
- No `answer_card` calls from practice
- No custom fields in `cards.data` for GRE results
- No deck/config keys that FSRS reads

---

## 5. UI wireframe

### 5.1 BrainLift shell (all pages)

```
┌─────────────────────────────────────────────────────────────┐
│  ◆ BrainLift                              [profile] [sync]  │
├──────────┬──────────────────────────────────────────────────┤
│ Review   │                                                  │
│ Practice │   <main content>                                 │
│ Readiness│                                                  │
│ Study Plan│                                                 │
└──────────┴──────────────────────────────────────────────────┘
```

Branding: distinct header color/wordmark (“BrainLift”, not “Anki”). Reuse Anki design tokens (`base.scss`) for accessibility, but separate layout component.

### 5.2 Review (`/brainlift/review`)

```
┌────────────────────────────────────────┐
│  Memory review                         │
│  Finish your scheduled cards in Anki.  │
│                                        │
│  [ Start review ]  → existing Anki flow│
│                                        │
│  Due today: 42 reviews · 8 new         │
└────────────────────────────────────────┘
```

### 5.3 Practice (`/brainlift/practice`) — Phase 1

```
┌────────────────────────────────────────┐
│  Practice · Quantitative reasoning     │
│  Topic: gre::quant::arithmetic::percent│
├────────────────────────────────────────┤
│  A store marks down a $80 item by 25%. │
│  What is the sale price?               │
│                                        │
│  ○ $55   ○ $60   ● $65   ○ $70         │
│                                        │
│  Confidence: [1][2][3][4][5] optional  │
│                                        │
│  [ Submit ]                            │
└────────────────────────────────────────┘

        ─── after submit ───

┌────────────────────────────────────────┐
│  ✓ Correct · 12.4s                     │
│  Explanation: 25% off → $20 discount…  │
│  [ Next question ]  [ Back to Readiness] │
└────────────────────────────────────────┘
```

### 5.4 Readiness (`/brainlift/readiness`) — Phase 1 placeholders

```
┌────────────────────────────────────────┐
│  Your GRE readiness                    │
├──────────────┬──────────────┬───────────┤
│ Memory       │ Performance  │ Readiness │
│   72%        │   58%        │   —       │
│ FSRS topics  │ 3/5 today    │ Phase 2   │
├──────────────┴──────────────┴───────────┤
│  Recent practice                        │
│  • Percent discount — correct, 12s      │
│  • RC inference — incorrect, 45s      │
└────────────────────────────────────────┘
```

### 5.5 Study Plan — Phase 1 stub

```
┌────────────────────────────────────────┐
│  Study plan                            │
│  Coming soon: ranked topics based on    │
│  memory gaps and practice misses.       │
└────────────────────────────────────────┘
```

### 5.6 Anki congrats handoff (only Anki UI change)

```
┌────────────────────────────────────────┐
│  Congratulations! Finished for now.    │
│  …existing Anki messages…              │
│                                        │
│  [ Continue to BrainLift Practice ]    │  ← NEW primary CTA
│  [ Custom study ] …                    │
└────────────────────────────────────────┘
```

---

## 6. Protobuf API (BrainLiftService)

New file `proto/anki/brainlift.proto`:

```protobuf
service BrainLiftService {
  rpc ListQuestions(ListQuestionsRequest) returns (ListQuestionsResponse);
  rpc RecordAttempt(RecordAttemptRequest) returns (RecordAttemptResponse);
  rpc GetScores(GetScoresRequest) returns (GetScoresResponse);
  rpc GetRecentAttempts(GetRecentAttemptsRequest) returns (GetRecentAttemptsResponse);
}
```

Phase 1 behavior:

| RPC                 | Behavior                                                                                                                    |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `ListQuestions`     | Return seeded MCQ items; filter by `topic_prefix` / `limit`                                                                 |
| `RecordAttempt`     | Insert into `bl_performance_attempt`; return `{ correct, explanation }`                                                     |
| `GetScores`         | Memory from delegated TopicMastery; Performance from attempt aggregates; Readiness = placeholder (`sufficient_data: false`) |
| `GetRecentAttempts` | Last N attempts for dashboard list                                                                                          |

Register in backend codegen same as other services. Expose via `mediasrv` `exposed_backend_list` + localhost whitelist.

---

## 7. Step-by-step implementation plan

### Phase 0 — Design approval (now)

- [ ] Review this document
- [ ] Confirm sidecar `brainlift.db` vs shared-schema tradeoff
- [ ] Confirm congrats-only Anki UI touch is acceptable

### Phase 1 — Minimum vertical slice (implement after approval)

**Goal:** Student can open BrainLift → review in Anki → get one hard-coded GRE question after congrats → submit → see result on Readiness dashboard. Everything compiles.

| Step | Work                                                                                                                  | Est. |
| ---- | --------------------------------------------------------------------------------------------------------------------- | ---- |
| 1.1  | Add `proto/anki/brainlift.proto`; codegen; stub `BrainLiftService` in rslib                                           | S    |
| 1.2  | Create `rslib/src/brainlift/storage/` + `schema.sql`; open `brainlift.db` beside profile                              | M    |
| 1.3  | Seed ~5 hard-coded questions; implement `ListQuestions`, `RecordAttempt`                                              | M    |
| 1.4  | Implement `GetScores` (memory delegates to `compute_topic_mastery`; performance from attempts; readiness placeholder) | M    |
| 1.5  | `pylib/anki/brainlift.py` wrappers on `Collection`                                                                    | S    |
| 1.6  | mediasrv: register routes `brainlift`, `brainlift/practice`, etc.; expose RPCs                                        | S    |
| 1.7  | SvelteKit `ts/routes/brainlift/` shell + 4 nav pages                                                                  | M    |
| 1.8  | Practice page: fetch question, timer, submit, show explanation                                                        | M    |
| 1.9  | Readiness page: three score cards + recent attempts                                                                   | S    |
| 1.10 | Qt `BrainLiftDialog` + menu entry; `load_sveltekit_page("brainlift")`                                                 | S    |
| 1.11 | Congrats CTA → `/brainlift/practice` (single button + link)                                                           | S    |
| 1.12 | Rust unit tests: attempt insert, score aggregation; pylib smoke test                                                  | S    |
| 1.13 | `just check` green                                                                                                    | S    |

**Phase 1 exit criteria**

- [ ] `./run` → BrainLift menu opens product shell
- [ ] Review tab starts normal Anki review; no reviewer diffs
- [ ] After finishing reviews, congrats shows BrainLift Practice CTA
- [ ] Practice shows ≥1 seeded GRE MCQ; submit stores attempt in `brainlift.db` only
- [ ] Readiness shows Memory (from FSRS/topics), Performance (from attempts), Readiness placeholder
- [ ] `grep revlog` / scheduler: zero writes from BrainLift practice path

### Phase 2 — Readiness model + Study Plan (later)

- GRE topic outline JSON; coverage denominator
- Readiness predictor v1 (rules-based)
- Study Plan ranking RPC
- Replace placeholder Readiness score

### Phase 3 — AI generation (later)

- `GenerateQuestion` RPC behind provider interface
- Prompt templates per format (MCQ, TC, RC, quant)
- Human review queue / caching in `bl_question`

### Phase 4 — Mobile + sync (later)

- iOS shell calling same protobuf RPCs
- `BrainLiftSyncService` for `brainlift.db`
- Offline attempt queue on mobile

---

## 8. Risk register

| Risk                          | Mitigation                                                                   |
| ----------------------------- | ---------------------------------------------------------------------------- |
| BrainLift feels like Anki     | Dedicated `/brainlift` shell, branding, product copy in `brainlift.ftl`      |
| Accidental FSRS contamination | Separate DB; code review checklist; tests assert no `revlog` writes          |
| API 403 in external browser   | Whitelist BrainLift RPCs on localhost (same as stats dev pages)              |
| Upstream Anki merge pain      | BrainLift code isolated under `rslib/src/brainlift/`, `ts/routes/brainlift/` |
| TopicMastery in stats.proto   | Acceptable as memory read; BrainLift-facing API still `BrainLiftService`     |

---

## 9. Open questions for approval

1. **Sidecar DB:** Confirm `{profile}/brainlift.db` vs new tables in `collection.anki2` (schema 19).
2. **Congrats CTA:** Always show, or only when GRE-tagged deck / BrainLift enabled setting?
3. **Review inside BrainLift:** Launch current deck study, or force a specific “GRE deck”?
4. **Existing `/readiness` dev page:** Remove or redirect to `/brainlift/readiness`?

---

_Next step: approve this design, then implement Phase 1 only._
