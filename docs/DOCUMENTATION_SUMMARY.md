# Documentation index

## Start here

| Document | Purpose |
|---|---|
| [README](../README.md) | Project overview and shortest quick start |
| [CLI guide](CLI.md) | Commands, source policies, sessions and troubleshooting |
| [End-to-end example](END_TO_END_EXAMPLE.md) | Reproducible ingest and answer workflow |

## Runtime reference

| Document | Purpose |
|---|---|
| [Engine](ENGINE.md) | `EngineRequest`, `EngineResponse` and runtime semantics |
| [Architecture](ARCHITECTURE.md) | Current component boundaries and data flow |
| [API](API.md) | Public Rust API and persistence contract |
| [Error handling](ERROR_HANDLING_GUIDE.md) | Typed errors and diagnostics |
| [Logging](LOGGING.md) | Operational logging and trace behavior |
| [Chat client](MODULAR_CHAT_ARCHITECTURE.md) | TUI boundary and responsibilities |

## Language and document reference

| Document | Purpose |
|---|---|
| [Interlingua](INTERLINGUA.md) | Language-neutral semantic model |
| [Morphology](MORPHOLOGY.md) | Morphological data and processing |
| [Generator](GENERATOR.md) | Language generation pipeline |
| [Document model](document/README.md) | Document contract and artifact stages |
| [Glossary](GLOSSARY.md) | Domain terminology |
| [Data management](DATA_MANAGEMENT_GUIDE.md) | Source, session and artifact storage |

## Benchmarks and quality

| Resource | Purpose |
|---|---|
| [Wikipedia Paris corpus](../benchmarks/wikipedia_paris_v1/README.md) | Snapshot-based source and QA fixture |
| [Document corpus](../benchmarks/document_v1/README.md) | Document reconstruction benchmark |
| [Test strategy](TEST_STRATEGY.md) | Test organization and verification approach |
| [Known limitations](KNOWN_LIMITATIONS.md) | Current implementation limits |
| [Implementation guide](IMPLEMENTATION_GUIDE.md) | Contributor workflow and invariants |

The runtime documents above describe the current implementation. Other files in `docs/` contain detailed language, benchmark or historical design notes and should not be treated as a second runtime API.
