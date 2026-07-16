#!/usr/bin/env bash
set -euo pipefail

roots=(
  crates/lexflex-parser/src
  crates/lexflex-engine/src/runtime
  crates/lexflex-engine/src/api/text.rs
)

failed=0

reject() {
  local pattern="$1"
  local message="$2"

  if rg -n \
    --glob '*.rs' \
    "$pattern" \
    "${roots[@]}" \
    --glob '!target/**' \
    --glob '!.git/**'
  then
    echo
    echo "ERROR: $message"
    failed=1
  fi
}

reject '"CAPITAL"|"POPULATION"|"PARIS"|"FRANCE"|"WARSAW"|"POLAND"' 'domain symbol in parser runtime'
reject 'parse_[a-z_]*(capital|population|location)' 'domain parser function'
reject 'sentence_template|phrase_template|question_pattern' 'sentence template architecture'
reject 'fallback.*Text|Unknown.*Text' 'unknown word semantic fallback'

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi
