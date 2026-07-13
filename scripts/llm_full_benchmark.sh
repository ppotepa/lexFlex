#!/bin/bash
set -euo pipefail

# Full LLM + lexlearn benchmark script
# - Processes all benchmark inputs with LLM priority (if env set)
# - Maximizes data: full processing + recursive base lemma deduction
# - Outputs RON proposals for concept DB building
# - Generates summary report

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
OUTDIR="results/llm-benchmark-${TIMESTAMP}"
mkdir -p "$OUTDIR" "$OUTDIR/proposals" "$OUTDIR/recursive"

echo "=== LLM Full Benchmark starting at $TIMESTAMP ==="
echo "Output: $OUTDIR"

# Check LLM env
if [ -z "${LEXFLEX_LLM_BASE_URL:-}" ]; then
  echo "WARNING: LEXFLEX_LLM_BASE_URL not set. Running with local knowledge only (no LLM)."
  echo "Set it for full LLM benchmark, e.g.:"
  echo "  export LEXFLEX_LLM_BASE_URL=http://192.168.195.88:1234/v1"
  echo "  export LEXFLEX_LLM_MODEL=google/gemma-4-e2b"
  LLM_MODE=false
  OFFLINE_FLAG="--offline"
else
  echo "LLM enabled: $LEXFLEX_LLM_BASE_URL (model: ${LEXFLEX_LLM_MODEL:-default})"
  LLM_MODE=true
  OFFLINE_FLAG=""
fi

# Benchmark inputs (max data)
INPUTS=(
  "benchmarks/input_adam.txt"
  "benchmarks/weak_corpus.txt"
  "benchmarks/dialogue_sample.txt"
  "benchmarks/benchmark_sentences.txt"
)

PROCESSED_WORDS_FILE="$OUTDIR/processed_words.txt"
touch "$PROCESSED_WORDS_FILE"

TOTAL_PROPOSALS=0
TOTAL_LEX_PROPOSALS=0

process_word() {
  local word="$1"
  local lang="${2:-pl}"
  local out_prefix="$3"

  # Skip if already processed
  if grep -qx "$word" "$PROCESSED_WORDS_FILE" 2>/dev/null; then
    return 0
  fi
  echo "$word" >> "$PROCESSED_WORDS_FILE"

  echo "  [deduce] $word"
  local json_out="$OUTDIR/${out_prefix}_${word}.json"
  ./target/debug/lexlearn deduce "$word" --lang "$lang" $OFFLINE_FLAG > /dev/null 2>&1 || true

  # The deduce doesn't save by default, so we capture via temp or modify? 
  # For simplicity, use bulk for main, single for recursive by saving manually? 
  # Workaround: run bulk on single line temp, but to keep simple, use echo and capture? 
  # Better: use the binary with redirect? Wait, deduce prints to stdout.
  # For benchmark, we'll rely on bulk outputs + parse for recursion.

  # To actually get single: temp input
  local tmp_in="/tmp/single_$word.txt"
  echo "$word" > "$tmp_in"
  local rec_dir="$OUTDIR/recursive"
  mkdir -p "$rec_dir"
  ./target/debug/lexlearn bulk --input "$tmp_in" --lang "$lang" --max 1 --output-dir "$rec_dir/single-$word" $OFFLINE_FLAG > /dev/null 2>&1 || true
  rm -f "$tmp_in"

  # Copy any new .concept.ron and .lex.ron to central proposals
  find "$rec_dir/single-$word" -name "*.concept.ron" -exec cp {} "$OUTDIR/proposals/single-${word}-$(basename {})" \; 2>/dev/null || true
  find "$rec_dir/single-$word" -name "*lex.ron" -exec cp {} "$OUTDIR/proposals/single-${word}-$(basename {})" \; 2>/dev/null || true
}

# Phase 1: Full bulk on all inputs (max data)
echo "=== Phase 1: Full bulk processing ==="
for input in "${INPUTS[@]}"; do
  if [ ! -f "$input" ]; then
    echo "Skipping missing $input"
    continue
  fi
  base=$(basename "$input" .txt)
  bulk_out="$OUTDIR/bulk-$base"
  echo "Processing $input -> $bulk_out"
  ./target/debug/lexlearn bulk \
    --input "$input" \
    --lang pl \
    --max 0 \
    --output-dir "$bulk_out" \
    $OFFLINE_FLAG || true

  # Copy proposals to central for easy review
  find "$bulk_out" -name "*.concept.ron" -exec cp {} "$OUTDIR/proposals/bulk-${base}-$(basename {})" \; 2>/dev/null || true
  find "$bulk_out" -name "*lex.ron" -exec cp {} "$OUTDIR/proposals/bulk-${base}-$(basename {})" \; 2>/dev/null || true

  # Count
  count=$(find "$bulk_out" -name "*.concept.ron" | wc -l)
  TOTAL_PROPOSALS=$((TOTAL_PROPOSALS + count))
  lex_count=$(find "$bulk_out" -name "*lex.ron" | wc -l)
  TOTAL_LEX_PROPOSALS=$((TOTAL_LEX_PROPOSALS + lex_count))
done

# Phase 2: Recursive base word processing (for words whose base may also be unknown)
echo "=== Phase 2: Recursive lemma / base word processing ==="
# Collect candidate base words from all generated jsons where there was a proposal or no strong concept
CANDIDATES=$(mktemp)
find "$OUTDIR" -name "*.json" -exec sh -c '
  for f; do
    if jq -e ".concept_proposal != null or .best_concept == null" "$f" > /dev/null 2>&1; then
      lemma=$(jq -r ".surface.lemma // .word // empty" "$f" | tr -d "\n")
      if [ -n "$lemma" ]; then
        echo "$lemma"
      fi
    fi
  done
' _ {} + | sort -u > "$CANDIDATES"

echo "Found $(wc -l < "$CANDIDATES") candidate base/lemma words for recursion"

while read -r word; do
  if [ -n "$word" ] && ! grep -qx "$word" "$PROCESSED_WORDS_FILE" 2>/dev/null; then
    process_word "$word" "pl" "recursive"
  fi
done < "$CANDIDATES"

# Additional pass for any new lemmas discovered in recursive
echo "=== Phase 3: One more recursion pass for new bases ==="
find "$OUTDIR/recursive" -name "*.json" -exec sh -c '
  for f; do
    lemma=$(jq -r ".surface.lemma // .word // empty" "$f" | tr -d "\n")
    if [ -n "$lemma" ]; then echo "$lemma"; fi
  done
' _ {} + | sort -u | while read -r w; do
  if [ -n "$w" ] && ! grep -qx "$w" "$PROCESSED_WORDS_FILE" 2>/dev/null; then
    process_word "$w" "pl" "recursive2"
  fi
done

# Phase 4: Summary and report for RON DB
echo "=== Phase 4: Building RON database summary ==="
REPORT="$OUTDIR/benchmark_report.txt"
{
  echo "LLM + lexlearn Full Benchmark Report - $TIMESTAMP"
  echo "LLM: ${LEXFLEX_LLM_BASE_URL:-none} / ${LEXFLEX_LLM_MODEL:-none}"
  echo "Total concept proposals generated: $TOTAL_PROPOSALS"
  echo "Total lexicon proposals generated: $TOTAL_LEX_PROPOSALS"
  echo ""
  echo "=== Unique proposed concept IDs (from .concept.ron) ==="
  find "$OUTDIR/proposals" -name "*.concept.ron" -exec grep -o 'id: "[^"]*"' {} + | sort -u | sed 's/id: "//;s/"$//' | head -100
  echo ""
  echo "=== Sample concept proposals ==="
  find "$OUTDIR/proposals" -name "*.concept.ron" | head -5 | while read f; do
    echo "--- $f ---"
    cat "$f"
    echo
  done
  echo "=== All proposal files location ==="
  echo "$OUTDIR/proposals/  (copy good ones to data/ after review)"
  echo ""
  echo "To build the RON DB: review files in $OUTDIR/proposals/, then manually merge high-quality ones into data/concepts/concepts.ron and data/lexicons/pl/lexicon.ron etc."
  echo "Recursive processing handled base lemmas for inflected/unknown words."
} > "$REPORT"

echo "=== BENCHMARK COMPLETE ==="
echo "Report: $REPORT"
echo "All RON proposals: $OUTDIR/proposals/"
echo "Full raw outputs: $OUTDIR/"
echo "Run 'cat $REPORT' for summary."
echo "Next: review proposals and integrate into data/ to grow the concept DB."

# Optional: count total unique words touched
TOTAL_WORDS=$(wc -l < "$PROCESSED_WORDS_FILE" 2>/dev/null || echo 0)
echo "Total unique words processed (incl recursive): $TOTAL_WORDS"
