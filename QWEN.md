# Token Optimization

This project uses two complementary tools to minimize token consumption during AI-assisted development.

## Tools Overview

### CodeGraph — Semantic Code Intelligence (MCP)

Builds a local semantic graph of the codebase. Exposes `codegraph_explore` — one query that answers structural questions (symbol locations, call paths, dependencies) instead of 5-10 grep/read calls.

**Reduction:** ~58% fewer tool calls, near-zero redundant file reads.

**Initialize in project:**

```bash
cd project-root
codegraph init
```

**MCP configuration:**

Add to your agent's MCP config:

```json
{
  "mcpServers": {
    "codegraph": {
      "type": "stdio",
      "command": "codegraph",
      "args": ["serve", "--mcp"],
      "env": {
        "CODEGRAPH_MCP_TOOLS": "explore",
        "CODEGRAPH_TELEMETRY": "0"
      }
    }
  }
}
```

### RTK — Shell Output Compression (CLI)

Rust proxy that intercepts shell command output and compresses it 60-90% before it enters your context window.

**Reduction:** 60-90% fewer tokens on shell output (git, docker, tests, builds).

**Install:**

```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh
```

## Agent Behavior Rules

### When exploring the codebase

1. **Use `codegraph_explore` first** for questions about code structure, dependencies, call paths, or finding symbols — one query replaces multiple file reads.
2. **Fall back to direct file reads only** when you need exact content for editing or reviewing diffs.
3. **Do not re-verify** CodeGraph results with additional grep/read calls — trust the index.

### When running shell commands

1. **Prefix with `rtk`** for commands with verbose output:
   ```bash
   rtk git status
   rtk git log
   rtk git diff
   rtk cargo test
   rtk cargo build
   ```

2. **Do NOT prefix** for:
   - Interactive commands (editors, debuggers, prompts)
   - Commands whose exact output is being parsed or piped
   - Commands with already-short output (pwd, whoami, simple ls)

### Coverage Matrix

| Tool | Replaces | Use for |
|---|---|---|
| CodeGraph | grep + read + glob chains | "Where is X defined?", "What calls Y?", "How does Z work?" |
| RTK | Raw shell output | Any verbose command (git, cargo, tests) |
| Built-in file tools | — | Precise single-file reads and edits |

# Project Rules

## Architecture

lexFlex is a universal meaning representation framework. Translation pipeline:

```
Source Language → [Parser] → Interlingua → [Generator] → Target Language
```

## Key Directories

- `src/core/` — Interlingua types, ontology, deduction engine, traits
- `src/engines/pl/` — Polish parser, generator, morphology
- `src/engines/en/` — English parser, generator, morphology
- `src/generation/` — Unified translation pipeline, realizer trait
- `src/data/` — RON data loaders (lexicon, morphology, descriptors)
- `data/` — All linguistic data in RON format (concepts, lexicons, morphology, ontology, descriptors)
- `docs/` — Architecture and design documentation
- `tests/` — Integration tests
- `scripts/` — Shell scripts for benchmarking and bulk testing

## Conventions

- **Data format:** RON (Rusty Object Notation) — all linguistic knowledge lives in data files, not in code
- **Algorithmic morphology:** Word forms are computed from paradigm rules, never hardcoded string lookups or if-checks
- **Data-driven design:** Linguistic rules belong in `.ron` files under `data/`, not in Rust source
- **No surface-form hacks:** Do not use `contains()`, `replace()`, or string matching on inflected word forms — use the morphology engine
- **Layered abstraction:** Languages are organized in layers (Layer 0: universal, Layer 1: natural, Layer 2: formal, Layer 3: programming)

## Build and Test

```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo run -- translate "Kot śpi na kanapie" --from pl --to en   # Translate
cargo run -- parse "Kot śpi" --lang pl                          # Parse to Interlingua
cargo run -- generate --from interlingua --to en                # Generate from Interlingua
```

## MVP Scope

- **Languages:** Polish ↔ English
- **Sentence types:** Simple declarative, negative, interrogative, coordinated
- **Grammar:** 7 Polish cases (algorithmic declension), basic English morphology
- **Concepts:** ~500 per language
