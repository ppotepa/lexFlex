# Public API

The public runtime API is exposed by the `lexflex::engine` module. The CLI and TUI use the same contract.

## Constructing the engine

```rust
use lexflex::engine::LexFlexEngine;

let mut engine = LexFlexEngine::new("data", "example", true)?;
```

The third argument enables offline mode. Use `false` only when live source access is intended.

## Requests

```rust
use lexflex::engine::{ConversationRequest, EngineRequest, LanguageMode};

let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn {
    text: "What is the capital of France?".into(),
    language: LanguageMode::Auto,
}));
```

The top-level areas are `Conversation`, `Translation`, `Session` and `Debug`. Each area has its own typed request enum. Translation keeps multilingual Interlingua context independently while publishing successfully compiled original input as Conversation knowledge.

Read evidence-backed facts from the current session without fetching or mutating sources:

```rust
use lexflex::engine::{DebugCommand, DebugFactRole, EngineRequest, FactsDebugQuery};

let response = engine.handle(EngineRequest::Debug {
    command: DebugCommand::Facts(FactsDebugQuery {
        selector: "Poland".into(),
        role: DebugFactRole::Any,
        limit: Some(50),
    }),
});
```

## Responses

```rust
use lexflex::engine::EngineResponse;

match response {
    EngineResponse::Conversation(value) => {
        println!("{:?}: {:?}", value.meta.status, value.text);
    }
    EngineResponse::Error { meta, error } => {
        eprintln!("{:?}: {:?}", meta.status, error);
    }
    _ => {}
}
```

`ResponseMeta` contains status, deterministic request ID, monotonic run ID, snapshot ID, diagnostics, a trace reference and artifact hashes. `DocumentAnswer` additionally contains answer status, kind, rows, evidence, conflicts and an answer hash. `DebugResponse` contains selector resolution, counts and typed facts with claim, occurrence, source sentence and exact source spans.

## Source providers

External source access is isolated behind:

```rust
pub trait SourceProvider: Send + Sync {
    fn resolve(&self, request: &SourceRequest)
        -> Result<SourceSnapshot, EngineError>;
}
```

`LocalSnapshotSourceProvider` is the built-in implementation. Tests can inject a deterministic provider with `LexFlexEngine::with_provider`.

## Persistence

```rust
engine.save_session()?;
engine.load_session()?;
```

The session store validates immutable snapshots and all referenced source and bundle artifacts. Corrupted or mismatched data returns `EngineError::Persistence`.

## Lower-level document API

`LexFlexAPI` remains available for direct parsing, generation and document-stage operations. Runtime clients should prefer `LexFlexEngine` when they need contextual translation, source ingestion, session state, query execution, evidence or trace metadata.

`LexFlexAPI::builder()` now validates the runtime data root before engine construction. If `.data_dir(...)` is omitted, the builder auto-discovers a valid project `data/` root. If `.data_dir(...)` is provided, that path is treated strictly and must already contain the full runtime asset layout.
