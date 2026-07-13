# lexFlex

A universal meaning representation framework built on **Interlingua** — a language-neutral semantic core for translating between natural languages, with future support for formal and programming languages.

## How It Works

```
Source Language → [Parser] → Interlingua → [Generator] → Target Language
```

**Interlingua** is the universal protocol — a semantic representation richer than any single language. Language engines parse into and generate from this intermediary, enabling any-to-any translation without direct language pairs.

The system has a **generic Interlingua concept DB** (`data/concepts/concepts.ron` + `ontology/`) that language-specific lexicons map to. This allows clean PL ↔ EN translation via the shared base.

**Automatic learning / RON DB extension (new):** Unknown words encountered during parsing are handled by the integrated `lexflex-learner` (with optional local LLM first). It proposes new concepts and lexicon entries, which are appended on-the-fly directly to the live `data/concepts/` and `data/lexicons/{pl,en}/` RON files. This builds/extends the database automatically while preserving the generic base + matching lexicons structure.

**Example:**

```bash
export LEXFLEX_LLM_BASE_URL=http://localhost:1234/v1
export LEXFLEX_LLM_MODEL=google/gemma-4-e2b
cargo run -- parse "Mieszkam z wielkimi domem." --lang pl
# Unknowns trigger learner → proposals appended to RON DB on the fly
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
# Build (includes learner)
cargo build

# Run tests
cargo test

# Translate Polish → English
cargo run -- translate "Kot śpi na kanapie" --from pl --to en

# With automatic learning (LLM + on-the-fly RON proposals)
export LEXFLEX_LLM_BASE_URL=http://localhost:1234/v1
export LEXFLEX_LLM_MODEL=google/gemma-4-e2b
cargo run -- parse "Mieszkam z wielkimi domem." --lang pl
# → proposals appended to data/concepts/ and data/lexicons/pl/

# Bulk learning to build RON DB (max data + recursive lemma handling)
./scripts/llm_full_benchmark.sh
```

See `scripts/llm_full_benchmark.sh` for full LLM-powered benchmark that generates proposals for the concept/lexicon DB (handles recursion for base forms of inflected unknowns).

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
├── src/                    # Main lexflex app (parse, translate, generate)
│   ├── core/               # Interlingua, ontology, deduction, unknown resolution + learner integration (on-the-fly RON)
│   ├── engines/{pl,en}/    # Parsers, generators, morphology (data-driven via RON)
│   ├── generation/         # Unified pipeline
│   ├── data/               # Loaders
│   ├── api.rs main.rs      # Public API + CLI
├── lexflex-learner/        # Learner crate (LLM + LocalKnowledge deduction)
│   ├── src/                # service (LLM priority), bulk (RON DB builder), sources/
│   └── (run via `lexlearn` binary or lib)
├── data/
│   ├── concepts/concepts.ron   # Generic Interlingua concept DB (source of truth)
│   ├── ontology/ontology.ron   # Hierarchy (SIZE > BIG, etc.)
│   ├── lexicons/{pl,en}/       # Language lexicons mapping to generic concepts
│   ├── morphology/             # Paradigms
│   └── descriptors/
├── docs/
├── scripts/                    # llm_full_benchmark.sh (max data + recursive proposals)
├── results/learned/            # Generated proposals (ignored in git)
└── Cargo.toml (workspace with lexflex-learner)
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
