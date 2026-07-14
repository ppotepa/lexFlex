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

Translate text:

```bash
cargo run -- translate "Tomek dał jabłko Izie" --from pl --to en --format answer
```

Ingest a local Wikipedia snapshot and ask a question:

```bash
cargo run -- ingest Paris --lang en --session paris --format summary
cargo run -- answer "What is the capital of France?" \
  --lang en --session paris --format pretty-json
```

The default source policy is snapshot-only. Add `--live` to `ingest` when a network fetch is explicitly required.

## Interactive chat

```bash
cargo run -- chat --offline
```

In the TUI, ingest a source before asking factual questions:

```text
/ingest Paris
What is the capital of France?
```

Useful commands are `/lang auto`, `/lang en`, `/lang pl`, `/translate en pl`, `/query <json>`, `/inspect`, `/trace`, `/clear` and `/quit`.

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

The current implementation is an offline-first MVP for PL/EN document knowledge QA and translation. Benchmark corpora and integration tests live under `benchmarks/` and `tests/`.
