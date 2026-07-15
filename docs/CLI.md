# CLI and user guide

This is the operational guide for the current `lexflex` binary. All examples assume the repository root as the working directory.

## Runtime data root

`lexflex` requires a valid runtime `data/` root containing concepts, ontology, language descriptors, lexicons and morphology tables.

- Without `--data`, the binary auto-discovers the nearest valid `data/` directory by walking upward from the current working directory, then falls back to the project manifest root.
- With `--data`, the path is strict. A relative path is resolved from the current working directory and must already contain the full runtime layout.
- Missing assets are a startup error, not a parser error and not a silent fallback to empty language resources.

Examples:

```bash
cargo run -- answer "What is Paris?" --lang en --format summary
cd src && cargo run -- answer "What is Paris?" --lang en --format summary
cd src && cargo run -- answer "What is Paris?" --lang en --data /home/ppotepa/git/lexFlex/data --format summary
```

This fails intentionally because `src/data` is not a valid runtime root:

```bash
cd src && cargo run -- answer "What is Paris?" --lang en --data data --format summary
```

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

Resolve a source using the configured source policy:

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

Ask a question. If the session has no matching source, the engine automatically resolves Wikipedia:

```bash
cargo run -- answer "What is the capital of France?" \
  --lang en --source-policy live \
  --session paris \
  --format pretty-json
```

The answer contains status, request and snapshot identifiers, diagnostics, artifact hashes, answer rows and evidence. `Unknown` means that the engine did not find an evidence-backed answer; it is not a fallback guess.

`Unknown` is a runtime answer state. Invalid `--data`, missing runtime assets, corrupted session files or invalid trace files are hard errors reported before or outside answering.

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
cargo run -- chat --source-policy live
```

The first factual operation in a new chat can be an ordinary question:

```text
What is Paris?
```

`/ingest` remains available for explicit source loading. Automatic discovery uses the current language, Wikipedia and the configured policy. `--offline` forces local snapshots.

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
| `/trace` | Show the latest persisted engine trace |
| `/clear` | Clear transcript state and reset the engine session |
| `/quit` | Exit the TUI |

Trace display follows `--trace off|brief|full`. The default is `full`; `Alt+V` cycles transcript verbosity between compact, normal, detailed and full-stack. Full-stack shows structured stage payloads as they arrive, while the busy indicator remains active until the final response.

On startup the chat header and system message show the resolved data root, the active source policy and whether the session is offline. If runtime assets cannot be validated, chat exits instead of starting with degraded language resources.

## Common causes of `Unknown`

- no matching Wikipedia source is available and live access/cache lookup failed;
- the requested fact is not present in the source;
- the question refers to the wrong entity or relation;
- the parser cannot produce supported question semantics;
- matching claims do not have usable evidence.

The engine reports these conditions instead of fabricating a value.

## Common hard errors

- `Runtime data root error`: invalid `--data` path or missing required runtime assets.
- `Engine initialization error`: runtime assets validated, but API or engine startup still failed.
- `Session load error`: persisted snapshot or artifacts are corrupted or mismatched.
- `Trace load error`: requested JSONL trace is missing or unreadable.
