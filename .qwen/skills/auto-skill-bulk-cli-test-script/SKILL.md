---
name: bulk-cli-test-script
description: Bash script pattern for bulk-testing CLI tools against a file of inputs with direction prefixes, producing per-line results and a summary
source: auto-skill
extracted_at: '2026-07-11T10:02:15.370Z'
---

# Bulk CLI Test Script Pattern

A reusable bash script pattern for testing any CLI tool that processes text input line-by-line from a file, supports direction/prefix routing, and produces a pass/fail summary.

## When to Use

- You have a CLI tool that takes text input (e.g., a translator, classifier, parser)
- You want to test hundreds of inputs from a file
- Input lines have direction/type prefixes (e.g., `PL->EN:sentence`, `JSON:input`)
- You need pass/fail tracking and a summary report
- You want options for limiting, filtering, and failure-only display

## Input File Format

```
# Comments start with #
PL->EN:Tomek widzi kota
PL->EN Tomek widzi kota
EN->PL:We see a cat.
EN->PL We see a cat.
```

Both colon-separated (`PREFIX:text`) and space-separated (`PREFIX text`) formats should be supported for flexibility.

## Script Template

```bash
#!/bin/bash
set -euo pipefail

INPUT_FILE="${1:-input.txt}"
LIMIT=""
DIRECTION="all"
FAIL_ONLY=false

# Parse remaining args
shift 2>/dev/null || true
while [[ $# -gt 0 ]]; do
    case "$1" in
        --limit) LIMIT="$2"; shift 2 ;;
        --direction) DIRECTION="$2"; shift 2 ;;
        --fail-only|-f) FAIL_ONLY=true; shift ;;
        *) shift ;;
    esac
done

# Count total testable lines
TOTAL=$(grep -cE '^(PL->EN|EN->PL)[ :]' "$INPUT_FILE" 2>/dev/null || echo "0")

OK=0
FAIL=0
COUNT=0
FAILURES=()

while IFS= read -r line; do
    line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    [[ -z "$line" ]] && continue
    [[ "$line" == \#* ]] && continue

    # CRITICAL: Quote glob patterns to prevent > being interpreted as comparison
    direction=""
    sentence=""
    if [[ "$line" == "PL->EN:"* ]]; then
        direction="PL->EN"
        sentence="${line#PL->EN:}"
        sentence="$(echo "$sentence" | sed 's/^[[:space:]]*//')"
    elif [[ "$line" == "PL->EN "* ]]; then
        direction="PL->EN"
        sentence="${line#PL->EN }"
    elif [[ "$line" == "EN->PL:"* ]]; then
        direction="EN->PL"
        sentence="${line#EN->PL:}"
        sentence="$(echo "$sentence" | sed 's/^[[:space:]]*//')"
    elif [[ "$line" == "EN->PL "* ]]; then
        direction="EN->PL"
        sentence="${line#EN->PL }"
    else
        continue
    fi

    # Direction filter
    if [[ "$DIRECTION" != "all" && "$direction" != "$DIRECTION" ]]; then
        continue
    fi

    COUNT=$((COUNT + 1))
    # CRITICAL: Use -n test for LIMIT, not bare [[ "$LIMIT" && ... ]]
    if [[ -n "$LIMIT" && $COUNT -gt $LIMIT ]]; then
        break
    fi

    # Run the CLI tool (adapt this to your tool)
    output=$(your-cli-tool --input "$sentence" --direction "$direction" 2>/dev/null || echo "ERROR")
    exit_code=$?

    if [[ $exit_code -eq 0 && "$output" != "ERROR" && -n "$output" ]]; then
        OK=$((OK + 1))
        status="✓"
    else
        FAIL=$((FAIL + 1))
        status="✗"
        FAILURES+=("$COUNT|$direction|$sentence|$output")
    fi

    if [[ "$FAIL_ONLY" == "false" || "$status" == "✗" ]]; then
        printf "  %3d  %-7s  %-30s  →  %-30s  %s\n" "$COUNT" "$direction" "${sentence:0:30}" "${output:0:30}" "$status"
    fi

done < "$INPUT_FILE"

# Summary
TOTAL_RUN=$((OK + FAIL))
echo ""
echo "  Total: ${OK}/${TOTAL_RUN} ($([ $TOTAL_RUN -gt 0 ] && echo "$(( OK * 100 / TOTAL_RUN ))" || echo "0")%)"

# List failures
if [[ ${#FAILURES[@]} -gt 0 ]]; then
    echo ""
    echo "  Failed:"
    for f in "${FAILURES[@]}"; do
        IFS='|' read -r num dir src out <<< "$f"
        echo "    #$num [$dir] \"$src\" → \"$out\""
    done
fi
```

## Bash Pitfalls Discovered

### 1. `>` in `[[ ]]` glob patterns is a string comparison operator

```bash
# WRONG - bash treats > as string comparison, not glob wildcard
if [[ "$line" == PL->EN:* ]]; then

# CORRECT - quoting the pattern prevents operator interpretation
if [[ "$line" == "PL->EN:"* ]]; then
```

### 2. Empty variable in `[[ ]]` with `&&`

```bash
# WRONG - bare $LIMIT when empty causes "too many arguments"
if [[ "$LIMIT" && $COUNT -gt $LIMIT ]]; then

# CORRECT - use -n test explicitly
if [[ -n "$LIMIT" && $COUNT -gt $LIMIT ]]; then
```

### 3. `cargo run --quiet` for clean output

When invoking Rust CLI tools in a loop, use `--quiet` to suppress compilation warnings:
```bash
output=$(cargo run --quiet --bin mytool -- translate -f "$from" -t "$to" "$sentence" 2>/dev/null)
```

## Usage Examples

```bash
# Full test suite
./scripts/bulk_test.sh input.txt

# First 20 sentences only
./scripts/bulk_test.sh input.txt --limit 20

# Only EN->PL direction
./scripts/bulk_test.sh input.txt --direction EN->PL

# Show only failures
./scripts/bulk_test.sh input.txt --fail-only
```

## Adapting for Other CLI Tools

Replace the tool invocation line:
```bash
# Translator
output=$(cargo run --quiet --bin lexflex -- translate -f "$from" -t "$to" "$sentence" 2>/dev/null)

# Classifier
output=$(python classify.py --input "$sentence" 2>/dev/null)

# Parser
output=$(./my-parser "$sentence" 2>/dev/null)
```

Adjust the prefix patterns to match your tool's routing mechanism (language pairs, file types, command modes, etc.).
