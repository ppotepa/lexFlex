# Implementation guide

This guide is for contributors working on the current runtime.

## Development loop

```bash
cargo check --workspace --all-targets
cargo test --workspace
git diff --check
```

Use a local source snapshot for deterministic work. Add or update a benchmark fixture when behavior is intentionally changed.

## Where to make changes

| Concern | Location |
|---|---|
| CLI commands | `src/main.rs` |
| TUI and slash commands | `src/chat/` |
| Engine contract and session | `src/engine/` |
| Interlingua and language-neutral types | `src/core/interlingua/` |
| English parser/generator | `src/engines/en/` |
| Polish parser/generator | `src/engines/pl/` |
| Document compilation and graph | `src/document/` |
| Knowledge and claims | `src/document/knowledge/` |
| Query planning/execution | `src/query/` |
| Runtime bundle assembly | `src/runtime/` |

## Runtime invariants

- Frontends communicate with the engine through typed requests and responses.
- Source text is hashed before analysis.
- Every published bundle has valid stage lineage and checksums.
- Semantic IDs are deterministic.
- Evidence uses source sentence and span references.
- Missing evidence returns `Unknown`; it is never replaced by a guessed value.
- Production semantic values use typed representations, not floating-point persistence.

## Tests

Engine contract tests are in `tests/engine_contract_tests.rs`. Document, resolution, knowledge, query and corpus tests cover their respective stages. Use the CLI examples in [CLI.md](CLI.md) as smoke tests for end-to-end changes.

The core runtime does not require an LLM. External source access is isolated behind `SourceProvider` and should not be implemented in the chat client.
