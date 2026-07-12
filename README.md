# lexFlex

A universal meaning representation framework built on **Interlingua** — a language-neutral semantic core for translating between natural languages, with future support for formal and programming languages.

## How It Works

```
Source Language → [Parser] → Interlingua → [Generator] → Target Language
```

**Interlingua** is the universal protocol — a semantic representation richer than any single language. Language engines parse into and generate from this intermediary, enabling any-to-any translation without direct language pairs.

**Example:**

```bash
cargo run -- translate "Kot śpi na kanapie" --from pl --to en
# → "The cat sleeps on the couch"
```

## Design Principles

- **Data-driven** — linguistic knowledge lives in RON data files, not in Rust code
- **Algorithmic morphology** — word forms are computed from paradigm rules, not lookup tables
- **Layered abstraction** — Layer 0 (universal), Layer 1 (natural languages), Layer 2 (formal), Layer 3 (programming)
- **Capability-based translation** — validates what the target language can express before translating

## Stack

| Component | Technology |
|---|---|
| Language | Rust (edition 2021) |
| Data format | RON (Rusty Object Notation) |
| CLI | clap |
| Serialization | serde |
| MVP languages | Polish ↔ English |

## Quick Start

```bash
# Build
cargo build

# Run tests
cargo test

# Translate Polish → English
cargo run -- translate "Kot śpi na kanapie" --from pl --to en

# Translate English → Polish
cargo run -- translate "The cat sleeps on the couch" --from en --to pl

# Parse to Interlingua
cargo run -- parse "Kot śpi" --lang pl

# Generate from Interlingua
cargo run -- generate --from interlingua --to en
```

## MVP Scope (v0.1)

**In scope:**
- Simple sentences with nouns, verbs, adjectives, pronouns
- 7 Polish cases (algorithmic declension)
- Negation, basic questions, coordination ("and", "or")
- Temporal reasoning and quantification
- Pronoun resolution within single sentences
- ~500 concepts per language

**Future (v0.2+):**
- Conversational AI (dialogue, intents, speech acts)
- Complex/subordinate clauses
- Formal languages (mathematics, logic)
- Programming languages (Python, SQL)

## Project Structure

```
lexFlex/
├── src/
│   ├── core/           # Interlingua types, ontology, deduction, traits
│   ├── engines/
│   │   ├── pl/         # Polish parser, generator, morphology
│   │   └── en/         # English parser, generator, morphology
│   ├── generation/     # Unified translation pipeline
│   ├── data/           # RON data loaders
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library root
│   └── api.rs          # Public API
├── data/
│   ├── concepts/       # Master Interlingua concepts (~50)
│   ├── lexicons/       # Per-language lexicon entries (PL: ~623, EN: ~160)
│   ├── morphology/     # Paradigm rules (nouns, verbs, adjectives)
│   ├── descriptors/    # Language descriptors (PL + EN)
│   └── ontology/       # Concept hierarchy (IS_A relations)
├── docs/               # Architecture and design documentation
├── tests/              # Integration tests
├── scripts/            # Benchmarking and bulk testing
└── Cargo.toml
```

## Documentation

Detailed documentation is in [`docs/`](./docs/). Key starting points:

| Document | Description |
|---|---|
| [Architecture](./docs/ARCHITECTURE.md) | System design and layered abstraction |
| [Interlingua](./docs/INTERLINGUA.md) | Semantic representation format |
| [Implementation Guide](./docs/IMPLEMENTATION_GUIDE.md) | Practical implementation details |
| [End-to-End Example](./docs/END_TO_END_EXAMPLE.md) | Full translation pipeline walkthrough |
| [Test Strategy](./docs/TEST_STRATEGY.md) | Testing approach (unit, integration, golden) |
| [Known Limitations](./docs/KNOWN_LIMITATIONS.md) | What v0.1 cannot do |

## AI Agent Instructions

This project includes agent-specific token optimization guides:

- `GROK.md` — for Grok / xAI agents
- `QWEN.md` — for Qwen Code / similar agents

These files describe the preferred workflow: **CodeGraph first**, **always prefix shell commands with `rtk`**, data-driven edits only, etc.

## License

Private — not licensed for redistribution.
