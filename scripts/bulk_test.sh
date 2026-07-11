#!/bin/bash
# bulk_test.sh - Run bulk translation tests from input file
# Usage: ./scripts/bulk_test.sh [input_file] [--limit N] [--direction PL->EN|EN->PL|all]
#
# Input format (one sentence per line):
#   PL->EN:Tomek widzi kota
#   PL->EN Tomek widzi kota
#   EN->PL:We see a cat.
#   EN->PL We see a cat.
#   # comment lines and empty lines are ignored

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

INPUT_FILE="${1:-$PROJECT_DIR/input.txt}"
LIMIT=""
DIRECTION="all"
VERBOSE=false
FAIL_ONLY=false

shift 2>/dev/null || true

while [[ $# -gt 0 ]]; do
    case "$1" in
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        --direction)
            DIRECTION="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --fail-only|-f)
            FAIL_ONLY=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

if [[ ! -f "$INPUT_FILE" ]]; then
    echo "ERROR: Input file not found: $INPUT_FILE"
    echo ""
    echo "Create an input file with sentences to test, e.g.:"
    echo "  PL->EN:Tomek widzi kota"
    echo "  PL->EN:Iza dała książkę Tomkowi"
    echo "  EN->PL:We see a cat."
    echo "  EN->PL:The student gave the book."
    exit 1
fi

TOTAL=$(grep -cE '^(PL->EN|EN->PL)[ :]' "$INPUT_FILE" 2>/dev/null || echo "0")
echo "═══════════════════════════════════════════════════════"
echo "  lexFlex Bulk Test"
echo "═══════════════════════════════════════════════════════"
echo "  Input:     $INPUT_FILE"
echo "  Sentences: $TOTAL"
echo "  Direction: $DIRECTION"
[[ -n "$LIMIT" ]] && echo "  Limit:     $LIMIT sentences"
[[ "$FAIL_ONLY" == "true" ]] && echo "  Mode:      show failures only"
echo "═══════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_DIR"

if [[ "$VERBOSE" == "true" ]]; then
    echo "Building lexFlex..."
    cargo build --release --bin lexflex 2>&1 | tail -1
fi

PL_EN_OK=0
PL_EN_FAIL=0
EN_PL_OK=0
EN_PL_FAIL=0
COUNT=0
FAILURES=()

while IFS= read -r line; do
    line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"

    [[ -z "$line" ]] && continue
    [[ "$line" == \#* ]] && continue

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

    if [[ "$DIRECTION" != "all" && "$direction" != "$DIRECTION" ]]; then
        continue
    fi

    COUNT=$((COUNT + 1))
    if [[ -n "$LIMIT" && $COUNT -gt $LIMIT ]]; then
        break
    fi

    from=""
    to=""
    if [[ "$direction" == "PL->EN" ]]; then
        from="pl"
        to="en"
    else
        from="en"
        to="pl"
    fi

    output=$(cargo run --quiet --bin lexflex -- translate -f "$from" -t "$to" "$sentence" 2>/dev/null || echo "ERROR")
    exit_code=$?

    status=""
    if [[ $exit_code -eq 0 && "$output" != "ERROR" && -n "$output" ]]; then
        status="✓"
        if [[ "$direction" == "PL->EN" ]]; then
            PL_EN_OK=$((PL_EN_OK + 1))
        else
            EN_PL_OK=$((EN_PL_OK + 1))
        fi
    else
        status="✗"
        if [[ "$direction" == "PL->EN" ]]; then
            PL_EN_FAIL=$((PL_EN_FAIL + 1))
        else
            EN_PL_FAIL=$((EN_PL_FAIL + 1))
        fi
    fi

    if [[ "$FAIL_ONLY" == "false" || "$status" == "✗" ]]; then
        printf "  %3d  %-7s  %-30s  →  %-30s  %s\n" "$COUNT" "$direction" "${sentence:0:30}" "${output:0:30}" "$status"
    fi

    if [[ "$status" == "✗" ]]; then
        FAILURES+=("$COUNT|$direction|$sentence|$output")
    fi

done < "$INPUT_FILE"

PL_EN_TOTAL=$((PL_EN_OK + PL_EN_FAIL))
EN_PL_TOTAL=$((EN_PL_OK + EN_PL_FAIL))
TOTAL=$((PL_EN_TOTAL + EN_PL_TOTAL))
TOTAL_OK=$((PL_EN_OK + EN_PL_OK))

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════"
echo "  PL->EN: ${PL_EN_OK}/${PL_EN_TOTAL} ($([ $PL_EN_TOTAL -gt 0 ] && echo "$(( PL_EN_OK * 100 / PL_EN_TOTAL ))" || echo "0")%)"
echo "  EN->PL: ${EN_PL_OK}/${EN_PL_TOTAL} ($([ $EN_PL_TOTAL -gt 0 ] && echo "$(( EN_PL_OK * 100 / EN_PL_TOTAL ))" || echo "0")%)"
echo "  Total:  ${TOTAL_OK}/${TOTAL} ($([ $TOTAL -gt 0 ] && echo "$(( TOTAL_OK * 100 / TOTAL ))" || echo "0")%)"
echo "═══════════════════════════════════════════════════════"

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    echo ""
    echo "  Failed sentences:"
    for f in "${FAILURES[@]}"; do
        IFS='|' read -r num dir src out <<< "$f"
        echo "    #$num [$dir] \"$src\" → \"$out\""
    done
    echo ""
fi
