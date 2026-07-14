# Conversation engine

`ConversationEngine` is the runtime boundary used by both the CLI and the TUI. Clients submit typed `EngineRequest` values and render typed `EngineResponse` values. They do not access parser, graph or knowledge internals directly.

## Request contract

```rust
pub enum EngineRequest {
    UserTurn { text: String, language: LanguageMode },
    Translate { text: String, from: LanguageId, to: LanguageId },
    IngestSource { source: SourceRequest },
    Query { query: QueryInterlingua },
    Inspect { target: InspectTarget },
    ClearSession,
}
```

The main response variants are:

- `Conversation` for natural-language turns;
- `Translation` for translation requests;
- `Ingest` for source ingestion;
- `Answer` for structured queries;
- `Inspection` for session inspection;
- `Error` for typed engine, source, pipeline, query or persistence failures.

Every response carries status, request ID, session snapshot ID, diagnostics, trace reference and artifact hashes. Fact answers also carry typed rows and evidence.

## Runtime flow

```text
UserTurn
  → language detection or explicit language
  → parser
  → Interlingua question semantics
  → QueryInterlingua
  → logical/physical query plan
  → knowledge execution
  → status, answer and evidence
```

Source ingestion uses one document pipeline:

```text
SourceSnapshot
  → DocumentCompilation
  → DocumentGraph
  → DocumentEntityResolution
  → DocumentTemporalDiscourse
  → DocumentKnowledgeExtraction
  → DocumentArtifactBundle
  → SessionWorkspace
```

Each published bundle validates its lineage and stage checksums. Incomplete bundles are rejected.

## Source policy

The engine uses a `SourceProvider` abstraction. The built-in provider supports Wikipedia snapshots, local files and live Wikipedia resolution through an explicit fetch policy.

- `SnapshotOnly` reads local data and never uses the network.
- `CacheFirst` uses a local snapshot and may fetch when live access is enabled.
- `Live` requires network access and updates the local cache.

Source text is hashed before processing. All downstream artifacts retain the source hash.

## Session model

`SessionWorkspace` is an immutable-snapshot model. Ingestion or clearing creates a new snapshot; previous snapshots remain readable. Session persistence validates source artifacts, bundles, hashes and path safety before loading.

The default storage layout is:

```text
data/sessions/<session-id>/
  snapshots/
  sources/
  bundles/
  traces/
```

## Answer semantics

The engine is open-world:

- evidence-backed matches produce an answer;
- absent evidence produces `Unknown`;
- conflicting claims remain visible as conflicts;
- no closed-world `No` is inferred from absence;
- the renderer cannot create values that are not present in result rows or evidence.

The runtime path is deterministic and does not require an LLM.
