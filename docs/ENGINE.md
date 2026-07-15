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

Every request trace also records the resolved runtime data root and offline/source policy, so runtime configuration is visible alongside semantic stages.

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

The engine uses a `SourceProvider` abstraction. The built-in provider supports Wikipedia snapshots, local files and live Wikipedia resolution. A natural-language `UserTurn` can discover a Wikipedia source automatically from its question entities.

- `SnapshotOnly` reads local data and never uses the network.
- `CacheFirst` uses a local snapshot and fetches only when it is missing.
- `Live` tries the network first and falls back to the local cache.

Source text is hashed before processing. All downstream artifacts retain the source hash.

## Runtime asset contract

The engine requires a complete runtime `data/` root. A valid root contains:

- `concepts/concepts.ron`
- `ontology/ontology.ron`
- `descriptors/en.ron` and `descriptors/pl.ron`
- `lexicons/en/lexicon.ron` and `lexicons/pl/lexicon.ron`
- `morphology/en/*.ron` and `morphology/pl/*.ron`

Missing runtime assets are configuration errors. The engine does not replace them with empty lexicons, synthetic descriptors or empty morphology tables.

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

Parser failures inside a valid runtime return `Unknown` for `UserTurn`; they are not reported as translation errors. Configuration and startup failures remain hard `Error` responses or CLI startup errors.

The runtime path is deterministic and does not require an LLM. Every execution receives a monotonic `run_id`; repeated identical requests retain the same deterministic request identity but produce separate trace files.
