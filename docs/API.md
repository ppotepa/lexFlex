# Public API

The public runtime API is exposed by the `lexflex::engine` module. The CLI and TUI use the same contract.

## Constructing the engine

```rust
use lexflex::engine::ConversationEngine;

let mut engine = ConversationEngine::new("data", "example", true)?;
```

The third argument enables offline mode. Use `false` only when live source access is intended.

## Requests

```rust
use lexflex::engine::{EngineRequest, LanguageMode};

let response = engine.handle(EngineRequest::UserTurn {
    text: "What is the capital of France?".into(),
    language: LanguageMode::Auto,
});
```

Available request types are `UserTurn`, `Translate`, `IngestSource`, `Query`, `Inspect` and `ClearSession`.

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

`ResponseMeta` contains status, deterministic request and snapshot IDs, diagnostics, a trace reference and artifact hashes. `DocumentAnswer` additionally contains answer status, kind, rows, evidence, conflicts and an answer hash.

## Source providers

External source access is isolated behind:

```rust
pub trait SourceProvider: Send + Sync {
    fn resolve(&self, request: &SourceRequest)
        -> Result<SourceSnapshot, EngineError>;
}
```

`LocalSnapshotSourceProvider` is the built-in implementation. Tests can inject a deterministic provider with `ConversationEngine::with_provider`.

## Persistence

```rust
engine.save_session()?;
engine.load_session()?;
```

The session store validates immutable snapshots and all referenced source and bundle artifacts. Corrupted or mismatched data returns `EngineError::Persistence`.

## Lower-level document API

`LexFlexAPI` remains available for direct parsing, translation and document-stage operations. Runtime clients should prefer `ConversationEngine` when they need source ingestion, session state, query execution, evidence or trace metadata.
