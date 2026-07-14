# Runtime architecture

lexFlex has one runtime path for translation, ingestion and document question answering. The CLI and TUI are adapters around the engine boundary.

```text
┌──────────────┐
│ CLI / TUI    │  input, commands, rendering
└──────┬───────┘
       │ EngineRequest / EngineResponse
┌──────▼───────┐
│ Conversation │  session lifecycle and orchestration
│ Engine       │
└──────┬───────┘
       ├── language detection and parser
       ├── translation through Interlingua
       ├── SourceProvider
       ├── SessionWorkspace
       └── QueryService
              │
              ▼
       document compilation
       graph construction
       entity resolution
       temporal/discourse analysis
       knowledge extraction
```

## Boundaries

### Frontends

`src/main.rs` implements the CLI. `src/chat/` implements the terminal client, command parsing, worker channel, transcript and rendering. Frontends construct requests and display responses; semantic decisions belong to the engine.

### Engine

`src/engine/` owns request dispatch, source resolution, session mutation, query execution, response metadata and trace persistence.

### Language and Interlingua

`src/engines/en/` and `src/engines/pl/` parse and generate language-specific forms. `src/core/interlingua/` is the language-neutral semantic contract used between language processing and runtime query planning.

### Document pipeline

`src/document/` contains compilation, graph, resolution, temporal/discourse and knowledge stages. `src/runtime/` assembles these stages into a `DocumentArtifactBundle` and validates cross-stage lineage.

### Query layer

`src/query/` validates `QueryInterlingua`, creates a query plan, executes typed constraints over knowledge claims and produces an evidence-bearing `DocumentAnswer`.

## Determinism and provenance

Semantic IDs and hashes are derived from input content, schema/algorithm data and parent artifacts. Source hashes and exact source spans are retained as evidence. Session snapshots and traces are persisted atomically and validated on load.

Runtime duration, terminal paths and other operational details are not part of semantic artifact hashes.

## Current scope

The supported runtime scope is Polish/English translation and offline-first document knowledge QA. Wikipedia is the first source adapter, not a special QA path. The engine is designed so additional source providers and language engines can be added without coupling them to the chat client.
