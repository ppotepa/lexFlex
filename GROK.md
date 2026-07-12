# Grok Token Optimization — lexFlex

This project uses a focused set of tools and workflows to minimize token consumption when working with Grok on lexFlex development.

## Tools Overview

### RTK — Shell Output Compression (CLI)

Rust proxy that intercepts shell command output and compresses it 60-94% before it reaches the model.

**Reduction:** 60-94% fewer tokens on shell output (git, cargo, tests, builds, rg, etc.).

**Current stats on this machine:** ~94.5% average savings across 240+ commands.

**Install / update:**
```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh
```

**How to use with Grok:**
- Always prefix verbose commands.
- Before running an unfamiliar command, check what wrapper exists:
  ```bash
  rtk rewrite "cargo test -- --quiet"
  rtk rewrite "rg 'some pattern' src/"
  ```
- If no specific wrapper exists, use the generic ones:
  - `rtk test <cmd>` — only show failures
  - `rtk err <cmd>` — stderr + warnings only
  - `rtk summary <cmd>` — very short summary

**Common prefixes in this project:**
```bash
rtk git status
rtk git diff
rtk git log
rtk cargo build
rtk cargo test
rtk cargo clippy
rtk rg "pattern" src/
rtk fd -e rs
rtk cargo run --bin lexflex -- ...
```

**Do NOT prefix** for:
- Interactive commands (vim, less, debuggers)
- Commands where you need the raw exact output for piping/parsing
- Trivial one-line commands (pwd, date, echo)

### CodeGraph — Semantic Code Intelligence

Builds a local symbol + dependency graph. One good `codegraph explore` or `codegraph query` often replaces 5-15 `grep` + `read_file` calls.

**.codegraph/ already exists** in this repository — use it.

**Key commands:**
```bash
codegraph status                  # Check index health
codegraph explore "translation"   # Symbols + call paths for an area
codegraph query "parse"           # Find symbols matching "parse"
codegraph callers "some_function"
codegraph callees "some_function"
codegraph node "LexFlexAPI"
codegraph files                   # Project structure from index
```

**When to use:**
- "Where is X defined / used?"
- "What calls this?"
- "Show me the flow for Y"
- Understanding architecture before editing

**Fallback:** Only fall back to `rtk rg` + targeted `read_file` when you need raw text for editing or CodeGraph doesn't have the answer.

### Grok Native Tooling (Built-in Advantages)

Grok has direct, high-precision tools that complement RTK + CodeGraph:

- **Parallel tool calls** — batch independent `read_file`, `grep`, or `run_terminal_command` in a single turn (biggest token/time saver).
- **`read_file` with offset/limit** — never read entire large files (lexicons, generators).
- **`search_replace`** — precise, minimal diffs instead of full-file rewrites.
- **Built-in `grep`** — fast, but still prefer `rtk rg` or CodeGraph for most structural questions.
- **RON awareness** — treat `.ron` data files as structured data; use targeted reads or `jq`-style extraction where possible.

## Agent Behavior Rules (Grok)

### 1. Exploration Priority (in order)

1. **CodeGraph first** (`codegraph explore` / `codegraph query` / `codegraph callers`)
2. **Narrow `rtk rg` / `rtk fd`** with globs (`-t rs`, `-e ron`, `src/engines/pl/**`)
3. **Targeted `read_file`** with `offset` + `limit` (only the part you need to edit)
4. **Direct full reads** — last resort, only for small files or when you will rewrite large sections

**Never** do broad `grep -r` or re-read the same file multiple times in one session.

### 2. Shell Commands

- Prefix with `rtk` by default for anything that can produce more than a few lines.
- When in doubt: run `rtk rewrite "your command"` first.
- Use `rtk test "cargo test ..."` when you only care about pass/fail + failures.
- For RON data exploration: `rtk rg` or `rtk read` on specific files.

### 3. Parallel Execution

Batch independent work:
- Read 3-4 unrelated files at once.
- Run `codegraph explore "X"` + `rtk rg "Y"` + `rtk cargo check` in parallel.
- Multiple narrow searches instead of one giant one.

### 4. Editing Discipline

1. Read the minimal relevant section first (`read_file` + range or `rtk read`).
2. Make the smallest possible `search_replace`.
3. Verify with a narrow diff or targeted re-read.

### 5. Data Files (RON)

- Lexicons and paradigms live in `data/`. They are the source of truth.
- Never hardcode surface forms or morphology rules in Rust.
- For inspection: use `rtk read -m 80 data/lexicons/pl/lexicon.ron` or `rtk rg` with context.
- Paradigm changes belong in the `.ron` files under `data/morphology/`.

## Project Rules (Must Follow)

### Architecture

lexFlex is a universal meaning representation framework. Translation pipeline:

```
Source Language → [Parser] → Interlingua → [Generator] → Target Language
```

Interlingua is the single source of truth. Language engines (pl, en) are plugins that map to/from it.

### Key Directories

- `src/core/` — Interlingua types, ontology, deduction engine, traits, temporal
- `src/engines/pl/` — Polish parser, generator, morphology
- `src/engines/en/` — English parser, generator, morphology
- `src/generation/` — Unified pipeline, realizer trait
- `src/data/` — RON data loaders
- `data/` — All linguistic data (concepts, lexicons, morphology paradigms, descriptors, ontology)
- `docs/` — Extensive architecture and design documentation (start with `DOCUMENTATION_SUMMARY.md`)
- `tests/` — Integration + golden tests
- `scripts/` — Benchmarking and bulk testing scripts

### Conventions

- **Data-driven:** Linguistic knowledge lives in RON files under `data/`, **not** in Rust code.
- **Algorithmic morphology:** All word forms are computed from paradigm rules. No `contains()`, string hacks, or hardcoded inflected forms.
- **No surface-form hacks:** Use the morphology engine for everything inflection-related.
- **Layered abstraction:** Future support for formal languages (Layer 2) and programming languages (Layer 3).
- **Capability-based:** Check what the target language can express before generating.

### Build, Test, Run (always use rtk where possible)

```bash
rtk cargo check
rtk cargo test                 # Shows "43 passed" style summary
rtk cargo clippy
rtk cargo run --bin lexflex -- languages
rtk cargo run --bin lexflex -- parse "Tomek dał Izie jabłko." --lang pl
```

Quick help for any command:
```bash
tldr cargo
tldr rg
tldr sd
tldr fd
```

Bulk edit data safely:
```bash
sd 'pattern' 'replacement' data/lexicons/pl/lexicon.ron
```

Find files the smart way:
```bash
rtk fd -e rs src/engines
rtk fd -e ron data/
```

### Documentation Reading Order (when onboarding or doing architecture work)

1. `README.md`
2. `docs/DOCUMENTATION_SUMMARY.md`
3. `docs/END_TO_END_EXAMPLE.md`
4. `docs/ARCHITECTURE.md` + `docs/INTERLINGUA.md`
5. Specific deep docs as needed (`GENERATOR.md`, `IMPLEMENTATION_GUIDE.md`, etc.)

## Coverage Matrix

| Goal                              | Preferred Tool(s)                          | Avoid |
|-----------------------------------|--------------------------------------------|-------|
| Structural questions, call graph  | `codegraph explore`, `codegraph query`, `codegraph callers` | Broad rg + multiple reads |
| Find files / symbols by name      | `codegraph query`, `rtk fd`, `rtk rg -l`   | `find` / raw grep |
| View source for editing           | `read_file` (with range) or `rtk read`     | Dumping whole large files |
| Run builds / tests                | `rtk cargo build`, `rtk cargo test`        | Raw cargo |
| Git operations                    | `rtk git ...`                              | Raw git |
| Inspect RON data                  | `rtk rg`, `rtk read -m N`, targeted `read_file` | `cat` on full lexicon |
| Diffs / reviews                   | `git diff \| delta` or `rtk git diff`      | Plain unified diff |
| Quick command help                | `tldr <cmd>`                               | `man` |

## Current Environment (as of 2026-07-12 setup)

Core token savers already available and symlinked:
- `rtk`, `codegraph`, `fd`, `bat`, `rg`, `delta`, `hyperfine`, `jq`, `fzf`

Newly installed during setup:
- `sd` and `tldr` (tealdeer) via `cargo install` → located in `~/.cargo/bin/`

**Important:** Make sure `~/.cargo/bin` is in your `PATH`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
(Added to `~/.bashrc` during setup for future shells.)

### CodeGraph note
`.codegraph/` exists but the SQLite DB is currently owned by root (readonly for ppotepa user). Queries will fail until fixed:

```bash
sudo chown -R $(whoami):$(whoami) .codegraph
codegraph sync     # or codegraph index
```

After fixing, `codegraph explore "..."` becomes extremely powerful for this codebase.

## Additional Recommended Tools (Current Status)

| Tool       | Status          | Why it helps lexFlex                                           | Usage |
|------------|-----------------|----------------------------------------------------------------|-------|
| `fd`       | Installed      | Fast, gitignore-respecting finder                              | `rtk fd -e rs` or `rtk fd -e ron` |
| `bat`      | Installed      | Syntax + ranges for large RON / source files                   | `rtk bat -p --line-range 100:150 file.ron` |
| `delta`    | Installed      | Readable diffs for generator / morphology changes              | `git diff \| delta` or `rtk git diff` |
| `hyperfine`| Installed      | Statistically sound benchmarking of the pipeline               | `hyperfine "rtk cargo run --bin lexflex -- ..."` |
| `sd`       | **Just installed** | Clean search/replace in data files without regex escaping hell | `sd 'old' 'new' data/**/*.ron` |
| `tldr`     | **Just installed** | Practical one-line examples instead of man                     | `tldr cargo` , `tldr rg` , `tldr sd` |
| `jq`       | Installed      | Any structured side-output                                     | `jq` on JSON reports |

## Project-local rtk configuration

`.rtk/` directory created with:
- `.rtk/README.md`
- `.rtk/filters/cargo.toml` — custom filter that tries to reduce repetitive "help: underscore" warnings from the known 18 warnings in this codebase.

After changes to filters, run:
```bash
rtk trust
rtk verify
```

You can extend with more filters (e.g. for benchmark scripts or data loading).

## Quick Start Checklist for Grok Sessions

- [x] Tools verified (`rtk`, `codegraph`, `fd`, `bat`, `sd`, `tldr`, `delta` etc.)
- [ ] Fix CodeGraph ownership if needed: `sudo chown -R $(whoami):$(whoami) .codegraph`
- [ ] `codegraph status` (or `codegraph sync` after chown)
- [ ] `rtk rewrite "..."` when unsure about any shell command
- [ ] Batch independent tool calls (reads, searches, codegraph queries)
- [ ] Start with `docs/DOCUMENTATION_SUMMARY.md` + `README.md` for architecture questions
- [ ] Always keep changes data-driven (edit `.ron` files, not ad-hoc strings in Rust)

## Verification Results (setup run)

- `rtk cargo check` → clean (only pre-existing warnings)
- `rtk cargo test` → **43 passed** (0 failures)
- `rtk cargo run --bin lexflex -- languages` → works (en, pl)
- New tools (`sd`, `tldr`) functional after PATH update

This file should be kept in sync with `QWEN.md` for the shared project rules and conventions.

