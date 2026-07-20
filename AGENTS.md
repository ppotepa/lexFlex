# lexFlex Agent Instructions

## Project Structure

lexFlex is a **Rust workspace** with multiple crates:

```
crates/
  lexflex-model      # Core types, ontology, interlingua representation
  lexflex-lingua     # Lingua language (semantic representation)
  lexflex-language   # Language engine traits, morphology, parser/generator
  lexflex-parser     # Parser implementation
  lexflex-store      # Persistence layer
  lexflex-engine     # Execution engine, query planning
  lexflex-learning   # Learning components
  lexflex-provider-llm # LLM provider integration
apps/
  lexflex            # CLI application
tools/
  lexflex-benchmark  # Benchmarking
  lexflex-model-inspect # Model inspection
data/                # Linguistic data (RON files: concepts, lexicons, paradigms)
```

## Tooling

### RTK (Shell Output Compression)
Always prefix noisy commands:
```bash
rtk cargo check
rtk cargo test
rtk cargo clippy
rtk cargo build
rtk cargo run --bin lexflex -- <args>
```

### CodeGraph (Code Intelligence)
```bash
codegraph status          # Check index health
codegraph sync            # Sync after changes
codegraph query "parse"   # Find symbols
codegraph explore "translation"
codegraph callers "function_name"
codegraph node "TypeName"
```

## Common Commands

| Task | Command |
|------|---------|
| Quick check | `rtk cargo check` |
| Tests | `rtk cargo test` |
| Lint | `rtk cargo clippy` |
| Format | `cargo fmt --all` |
| Run CLI | `rtk cargo run --bin lexflex -- --help` |
| Parse text | `rtk cargo run --bin lexflex -- parse "text" --lang pl` |
| Show languages | `rtk cargo run --bin lexflex -- languages` |

## Architecture Principles

- **Interlingua-first**: Core types in `lexflex-model`, language engines map to/from it
- **Data-driven**: All linguistic knowledge in `data/` (RON files), never hardcoded
- **Algorithmic morphology**: Forms computed from paradigm rules, no string hacks
- **Layered**: Layer 1 = natural language, future Layer 2/3 = formal/programming languages

## Key Files to Know

- `Cargo.toml` - Workspace root, all members listed
- `data/` - Source of truth for linguistics (concepts.ron, lexicons/, paradigms/)
- `docs/ARCHITECTURE.md` - Detailed architecture
- `docs/INTERLINGUA.md` - Interlingua specification

## Workflow

1. Edit RON data in `data/` for linguistic changes
2. Rust code changes in `crates/`
3. Run `rtk cargo check` → `rtk cargo test` → `rtk cargo clippy`
4. `codegraph sync` after structural changes

## No-Go Patterns

- ❌ Hardcoding surface forms in Rust
- ❌ String matching for inflection (`contains()`, `ends_with()`)
- ❌ Compatibility aliases or legacy paths
- ❌ V2-suffixed artifacts (this is V1 canonical)