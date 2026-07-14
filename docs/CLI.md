# CLI and user guide

This is the operational guide for the current `lexflex` binary. All examples assume the repository root as the working directory.

## Build and verify

```bash
cargo check --workspace --all-targets
cargo test --workspace
```

Use `--release` for longer-running document ingestion:

```bash
cargo run --release -- ingest Paris --lang en --session paris
```

## Translation

```bash
cargo run -- translate "Tomek dał jabłko Izie" \
  --from pl --to en --format answer
```

Supported language identifiers are currently `pl` and `en`. Output formats accepted by engine commands are `json`, `pretty-json`, `summary` and, where applicable, `answer`.

## Sources

Inspect a local snapshot without network access:

```bash
cargo run -- source inspect Paris --lang en --format summary
```

Resolve a source using the cache and, only when requested, live Wikipedia access:

```bash
cargo run -- source fetch Paris --lang en --format summary
cargo run -- source fetch Paris --lang en --live --format summary
```

Local Wikipedia snapshots are stored below:

```text
data/sources/wikipedia/<language>/<slug>/<revision>.json
```

Snapshot content is validated against its SHA-256 hash. In offline mode a missing snapshot is an explicit error.

## Ingest and answer

Ingestion creates a complete document artifact bundle and persists the session:

```bash
cargo run -- ingest Paris \
  --lang en \
  --session paris \
  --format summary
```

Ask a question against that saved session:

```bash
cargo run -- answer "What is the capital of France?" \
  --lang en \
  --session paris \
  --format pretty-json
```

The answer contains status, request and snapshot identifiers, diagnostics, artifact hashes, answer rows and evidence. `Unknown` means that the engine did not find an evidence-backed answer; it is not a fallback guess.

The question language can be detected automatically:

```bash
cargo run -- answer "Jaka jest stolica Francji?" \
  --lang auto \
  --session paris \
  --format summary
```

## Structured queries

`query` executes a serialized `QueryInterlingua` against a saved session:

```bash
cargo run -- query '<QueryInterlingua JSON>' \
  --session paris \
  --format pretty-json
```

For normal use, prefer `answer`: it parses the natural-language question and constructs the structured query through the engine.

## Sessions and inspection

```bash
cargo run -- inspect --session paris --target session --format pretty-json
cargo run -- session-load --session paris
cargo run -- session-save --session paris
```

Session data is persisted below `data/sessions/<session-id>/`. Snapshots are immutable artifacts; `current.json` points to the latest validated snapshot.

Read a persisted trace by its turn identifier:

```bash
cargo run -- trace --session paris <turn-id>
```

Traces are deterministic JSONL records of request, language detection, pipeline stages, query planning, execution and answer selection.

## Interactive chat

```bash
cargo run -- chat --offline
```

The first factual operation in a new chat should be:

```text
/ingest Paris
```

Available commands:

| Command | Purpose |
|---|---|
| `/chat` | Return to normal message mode |
| `/ingest <title>` | Ingest a Wikipedia snapshot |
| `/lang auto` | Detect PL/EN automatically |
| `/lang pl`, `/lang en` | Force chat language |
| `/translate <from> <to>` | Enter translation mode |
| `/query <json>` | Execute a structured query |
| `/inspect [sources\|bundles]` | Inspect session state |
| `/trace` | Show trace visibility information |
| `/clear` | Clear the current session |
| `/quit` | Exit the TUI |

## Common causes of `Unknown`

- no source has been ingested in the current session;
- the requested fact is not present in the source;
- the question refers to the wrong entity or relation;
- the parser cannot produce supported question semantics;
- matching claims do not have usable evidence.

The engine reports these conditions instead of fabricating a value.
