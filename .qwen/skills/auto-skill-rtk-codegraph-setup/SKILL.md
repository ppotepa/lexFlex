---
name: rtk-codegraph-setup
description: Install and configure RTK and Codegraph tools to optimize AI coding assistant sessions with token reduction and code knowledge graphs
source: auto-skill
extracted_at: '2026-07-11T10:02:15.370Z'
---

# RTK + Codegraph Setup for AI Coding Assistants

This skill sets up two complementary tools that optimize AI coding assistant sessions:

- **RTK**: CLI proxy that reduces token consumption by 60-90% on common dev commands (cargo test, git status, etc.)
- **Codegraph**: Pre-indexed code knowledge graph that provides structured code understanding with call graphs, dependencies, and blast radius analysis

## When to Use

- Setting up a new project for AI-assisted development
- Optimizing long coding sessions to reduce token costs
- Needing fast code navigation and dependency analysis
- Working with AI coding tools (Claude Code, Qwen, Cursor, Codex, etc.)

## Installation

### RTK

**Option 1: Curl installer (recommended for Linux/macOS)**
```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh
```

**Option 2: Homebrew**
```bash
brew install rtk-ai/tap/rtk
```

### Codegraph

**Option 1: npm (recommended)**
```bash
npm i -g @colbymchenry/codegraph
```

**Option 2: Curl installer**
```bash
curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh
```

## Configuration

### RTK Setup

1. Initialize global hooks with auto-patch:
   ```bash
   rtk init --global --auto-patch
   ```
   This sets up PreToolUse hooks that automatically compress CLI output for supported AI agents.

2. Disable telemetry (optional but recommended):
   ```bash
   rtk telemetry disable
   ```

3. Verify installation:
   ```bash
   rtk --version
   ```

### Codegraph Setup

1. Navigate to project directory:
   ```bash
   cd /path/to/your/project
   ```

2. Initialize and build the code graph:
   ```bash
   codegraph init
   ```
   This creates `.codegraph/` directory with SQLite backend and indexes all supported source files.

3. Verify indexing:
   ```bash
   codegraph status
   ```

## Usage

### RTK Commands

Wrap common dev commands to get compressed output:

```bash
rtk cargo test                    # Compresses test output to summary
rtk cargo build                   # Compresses build output
rtk git status                    # Compresses git status
rtk git log                       # Compresses git log
```

Example: `cargo test` with 37 tests produces verbose output, but `rtk cargo test` returns:
```
cargo test: 37 passed (1 suite, 0.17s)
```

### Codegraph Commands

Explore code structure and dependencies:

```bash
codegraph explore "function_name"     # Get source, call paths, blast radius
codegraph callers <symbol>            # Find what calls this symbol
codegraph callees <symbol>            # Find what this symbol calls
codegraph impact <symbol>             # Analyze change impact
codegraph query "search term"         # Search symbols
codegraph status                      # Show indexing statistics
```

Example output from `codegraph explore "translate"`:
- Shows 54 symbols across 4 files
- Lists callers and dependencies
- Provides verbatim source code with line numbers
- Marks symbols without test coverage

## Integration with AI Coding Assistants

Once configured, these tools work automatically:

- **RTK**: The global hook intercepts shell commands and compresses output before it reaches the AI context window
- **Codegraph**: Available as MCP tools (if configured) or via CLI for the AI to query code structure

For Claude Code, the setup automatically adds:
- `@RTK.md` reference in CLAUDE.md
- PreToolUse hook in settings.json

For other agents (Cursor, Codex, etc.), use:
```bash
rtk init --global --agent <agent-name>
```

## Supported Languages

**Codegraph** supports full structural extraction for:
- Rust, TypeScript, JavaScript, Python, Go, Java, C#, PHP, Ruby
- C, C++, Objective-C, Swift, Kotlin, Scala, Dart
- And 20+ more languages

**RTK** optimizes 30+ commands including:
- cargo test, pytest, go test, git diff/status/log
- grep, find, ls, pnpm list, tsc, eslint
- docker, kubectl, and more

## Troubleshooting

### RTK hook not working
- Restart your AI coding tool after `rtk init`
- Check `rtk init --show` to verify configuration
- For Claude Code, ensure settings.json has the hook

### Codegraph indexing issues
- Large projects (>1000 files) may take time on first `init`
- Check `codegraph status` for indexing progress
- Use `codegraph.json` to exclude paths: `{"exclude": ["node_modules/", "target/"]}`

### Bash scripting pitfall
When parsing direction-prefixed input files (e.g., `PL->EN:sentence`), quote glob patterns in `[[ ]]`:

**Wrong** (interprets `>` as string comparison):
```bash
if [[ "$line" == PL->EN:* ]]; then
```

**Correct** (quotes prevent operator interpretation):
```bash
if [[ "$line" == "PL->EN:"* ]]; then
```

Also use `-n` test for non-empty checks:
```bash
[[ -n "$LIMIT" && $COUNT -gt $LIMIT ]]
```

## Benefits

- **Token savings**: 60-90% reduction on CLI output
- **Longer sessions**: 3x more context available
- **Faster navigation**: Instant code structure queries
- **Better understanding**: Call graphs and dependency analysis
- **Cost reduction**: Lower API costs for AI coding tools

## Example Workflow

1. Start coding session with AI assistant
2. AI runs `rtk cargo test` to check test results (compressed output)
3. AI uses `codegraph explore "function"` to understand code structure
4. AI makes changes based on structured understanding
5. AI runs `rtk cargo build` to verify (compressed output)
6. Repeat with full context preserved

This setup is particularly valuable for:
- Large codebases where grep is insufficient
- Long debugging sessions that exhaust context windows
- Projects with complex dependency graphs
- Cost-sensitive development workflows
