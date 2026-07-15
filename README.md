# lexFlex

lexFlex is a deterministic language and document engine built around a language-neutral Interlingua. It currently provides:

- Polish and English parsing and generation;
- text translation through the Interlingua layer;
- document ingestion into a validated artifact bundle;
- entity resolution and evidence-backed knowledge extraction;
- natural-language questions over an ingested session;
- a terminal chat client and a scriptable CLI;
- deterministic session snapshots, hashes and JSONL traces.

The runtime does not invent facts. A factual answer requires an ingested source and evidence; otherwise the result is `Unknown`.

## Quick start

Requirements: Rust toolchain and a local checkout of the repository.

```bash
cargo check --workspace --all-targets
cargo test --workspace
```

The binary needs a valid runtime `data/` root with lexicons, morphology, descriptors and ontology. By default it auto-discovers the nearest valid project `data/` directory, so running from `.` or `src/` works. If you pass `--data`, that path is treated strictly and must already be valid.

Translate text:

```bash
cargo run -- translate "Tomek dał jabłko Izie" --from pl --to en --format answer
```

Ask a factual question directly. The engine resolves Wikipedia automatically:

```bash
cargo run -- answer "What is the capital of France?" \
  --lang en --session paris --source-policy snapshot-only --format pretty-json
```

The default source policy is live-first with a local snapshot fallback. Use `--source-policy snapshot-only` or `--offline` for reproducible offline runs.

## Interactive chat

```bash
cargo run -- chat --source-policy live
```

In the TUI, ordinary factual questions automatically discover and ingest a Wikipedia source:

```text
What is Paris?
```

Useful commands are `/lang auto`, `/lang en`, `/lang pl`, `/translate en pl`, `/query <json>`, `/inspect`, `/trace`, `/clear` and `/quit`.

Progress is rendered live in the transcript with a Braille spinner and automatic bottom scrolling. Startup shows the resolved data root and source policy. `/trace` shows the latest persisted run; `Alt+V` cycles `compact`, `normal`, `detailed` and `full-stack` verbosity.

If startup cannot validate the runtime assets, chat exits immediately with a configuration error. It does not start with empty lexicons or synthetic fallbacks.

## Runtime model

```text
CLI / TUI
  → EngineRequest
  → ConversationEngine
  → source snapshot
  → document compilation and graph
  → entity resolution
  → knowledge extraction
  → QueryInterlingua and planner
  → evidence-backed EngineResponse
```

The chat layer is a client and renderer. Semantic processing belongs to the engine.

## Documentation

- [CLI and user guide](docs/CLI.md) — commands, sessions, sources and troubleshooting.
- [Engine contract](docs/ENGINE.md) — request/response boundary and runtime behavior.
- [Architecture](docs/ARCHITECTURE.md) — current component boundaries and data flow.
- [End-to-end example](docs/END_TO_END_EXAMPLE.md) — reproducible ingest and query flow.
- [API reference](docs/API.md) — public Rust types and entry points.
- [Document model](docs/document/README.md) — document pipeline details.
- [Documentation index](docs/DOCUMENTATION_SUMMARY.md) — reference material by topic.

## Repository status

The current implementation is a PL/EN document knowledge QA and translation MVP. Chat defaults to live Wikipedia resolution with cache fallback; offline reproducibility is available through `--offline` or `snapshot-only`. Benchmark corpora and integration tests live under `benchmarks/` and `tests/`.
