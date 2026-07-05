# GRE Atlas — architecture

## System diagram

```mermaid
flowchart TB
    subgraph clients [Clients]
        GREShell["GRE main shell<br/>qt/aqt/gre_dashboard.py"]
        GREDialog["GRE modal dialog<br/>qt/aqt/gre_atlas.py"]
        Mobile["iOS companion<br/>mobile/mobile_bridge"]
        EvalCLI["Eval CLI<br/>scripts/eval/"]
    end

    subgraph gre_ui [GRE UI — SvelteKit]
        Pages["ts/routes/(gre)/<br/>home · dashboard · practice<br/>study-plan · readiness · progress<br/>topics/[topicId] · settings"]
        Congrats["ts/routes/congrats<br/>Practice / Dashboard CTAs"]
    end

    subgraph qt_bridge [Qt bridge]
        MediaSrv["qt/aqt/mediasrv.py<br/>POST /_anki/* protobuf RPCs"]
        PyCol["pylib/anki/collection.py<br/>anki.gre_atlas wrappers"]
        RSBridge["pylib/rsbridge → Rust Backend"]
    end

    subgraph gre_atlas_engine [GRE Atlas engine — rslib/src/gre_atlas/]
        Service["BrainLiftService<br/>18 RPCs"]
        Storage["greatlas.db<br/>practice · calibration · sync"]
        Models["Scores · readiness · study plan<br/>calibration · eval · ablation"]
        Questions["Static GRE question bank"]
    end

    subgraph anki_engine [Anki engine — read-only for GRE Atlas memory]
        Stats["StatsService.TopicMastery<br/>rslib/src/stats/mastery.rs"]
        Scheduler["FSRS scheduler<br/>rslib/scheduler/"]
    end

    subgraph data [SQLite]
        ColDB[("collection.anki2<br/>cards · revlog · decks")]
        BLDB[("greatlas.db<br/>beside profile")]
    end

    GREShell --> Pages
    GREDialog --> Pages
    Congrats --> GREDialog
    Pages --> MediaSrv
    EvalCLI --> PyCol
    Mobile --> RSBridge
    MediaSrv --> PyCol --> RSBridge
    RSBridge --> Service
    RSBridge --> Stats
    Service --> Storage
    Service --> Models
    Service --> Questions
    Models --> Stats
    Stats --> ColDB
    Scheduler --> ColDB
    Storage --> BLDB
    Service -.->|"never writes revlog"| ColDB
```

## Layer responsibilities

| Layer                         | Responsibility                                             | GRE Atlas rule                                                 |
| ----------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------- |
| **GRE UI**                    | Product pages, navigation, abstention UX                   | Calls `@generated/backend` RPCs only                           |
| **mediasrv**                  | Protobuf over HTTP to Rust                                 | Whitelists GRE RPCs for embedded webview                       |
| **BrainLiftService**          | Practice, scores, dashboard, study plan, calibration, eval | Writes **only** `greatlas.db` (+ deck/card ops for flashcards) |
| **StatsService.TopicMastery** | FSRS retrievability by `gre::` tags                        | Read-only on `collection.anki2`                                |
| **Reviewer**                  | Standard Anki review                                       | Unchanged; GRE deck returns to GRE shell after review          |

## Score pipeline

```mermaid
flowchart LR
    subgraph inputs [Inputs]
        Revlog["FSRS revlog + gre:: tags"]
        Practice["bl_performance_attempt"]
        Catalog["GRE topic catalog"]
    end

    subgraph scores [Scores]
        Memory["MemoryScore<br/>retrievability × 100"]
        Perf["PerformanceScore<br/>accuracy × 100"]
        Ready["ReadinessScore<br/>0.45·M + 0.45·P + 0.10·C"]
    end

    Revlog --> Memory
    Practice --> Perf
    Catalog --> Memory
    Catalog --> Ready
    Memory --> Ready
    Perf --> Ready
```

**Abstention:** Memory requires FSRS + ≥50 studied GRE cards + ≥50% weighted catalog coverage. Performance requires ≥50 practice attempts. Readiness requires both.

## RPC surface

### BrainLiftService (`proto/anki/brainlift.proto`)

| RPC                                    | Purpose                                          |
| -------------------------------------- | ------------------------------------------------ |
| `ListQuestions` / `GetQuestion`        | GRE practice bank                                |
| `CreateSession` / `RecordAttempt`      | Practice sessions (→ `greatlas.db` only)         |
| `GetScores`                            | Memory + performance + readiness + estimated GRE |
| `GetDashboard`                         | Full dashboard state + recent activity           |
| `GetRecentAttempts`                    | Attempt history filter                           |
| `GetGreStudyStatus`                    | Deck existence, due counts                       |
| `GetStudyPlan`                         | Ranked recommendations + daily plan              |
| `GetReadinessCalibration`              | Live readiness + calibration stats               |
| `GetTopicDetails`                      | Per-topic drill-down                             |
| `GenerateBrainLiftEvalReport`          | Read-only eval JSON + Markdown                   |
| `GenerateBrainLiftAiEvalReport`        | Read-only AI gold-set eval                       |
| `GenerateQuestion`                     | Template MCQ from ETS source (optional persist)  |
| `GetBrainLiftSyncStatus` / Pull / Push | Mobile `greatlas.db` sync                        |
| `PrepareDemoCollection`                | Idempotent demo seed (mobile / tests)            |

### StatsService

| RPC            | Purpose                                      |
| -------------- | -------------------------------------------- |
| `TopicMastery` | FSRS retrievability aggregation for GRE tags |

## Navigation model (desktop)

| Surface                 | Default route                                  | Notes                                                                                                        |
| ----------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Main GRE shell          | `/home`                                        | Opened on collection load (`greDashboard` state); header nav: Dashboard, Study, Practice, Progress, Settings |
| Modal GRE dialog        | `/dashboard`                                   | Congrats CTAs, `open_gre_atlas()`                                                                            |
| Toolbar (outside shell) | `/home`, `/practice`, `/progress`, `/settings` | Quick links back into shell                                                                                  |

Both hosts load the same SvelteKit route tree under `ts/routes/(gre)/`.

The Qt menu bar has **GRE → Debug** only (Deck Browser, Browse, Add, Stats, Sync). Collection open is the primary GRE entry — there is no separate “Open GRE” menu item.

## Evaluation architecture

```mermaid
flowchart LR
    EvalPy["gre_atlas_eval.py"] --> RPC["GenerateBrainLiftEvalReport"]
    RPC --> EvalRs["rslib/gre_atlas/eval.rs"]
    EvalRs --> Cal["calibration.rs"]
    EvalRs --> MemE["memory_eval.rs"]
    EvalRs --> PerfE["performance_eval.rs"]
    EvalRs --> Abl["ablation_eval.rs"]
    EvalRs --> Out["JSON + Markdown"]
```

Held-out rule everywhere: **`id % 5 == 0`** (predictions, attempts, revlog).

## Source index

```
proto/anki/brainlift.proto
rslib/src/gre_atlas/          # engine
rslib/src/stats/mastery.rs    # memory signal
pylib/anki/gre_atlas.py       # Python API
qt/aqt/gre_dashboard.py       # main shell
qt/aqt/gre_atlas.py           # modal + bridge commands
qt/aqt/mediasrv.py            # HTTP RPC whitelist
ts/routes/(gre)/              # GRE UI
scripts/eval/                 # eval + benchmark CLI
docs/models/                  # model specifications
```
