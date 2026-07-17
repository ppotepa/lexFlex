#!/usr/bin/env bash
set -euo pipefail

failed=0
warnings=0

roots=(
  crates/lexflex-engine/src
  crates/lexflex-language/src
  crates/lexflex-lingua/src
  crates/lexflex-model/src
  crates/lexflex-learning/src
  crates/lexflex-provider-llm/src
  crates/lexflex-store/src
  crates/lexflex-parser/src
  apps/lexflex/src
  tools/lexflex-benchmark/src
  tools/lexflex-model-inspect/src
)

while IFS= read -r -d '' file; do
  lines="$(wc -l < "$file")"

  if (( lines > 500 )); then
    printf 'ERROR: %s has %s lines (hard limit >500)\n' "$file" "$lines"
    failed=1
  elif (( lines > 300 )); then
    printf 'WARNING: %s has %s lines (soft threshold >300, consider splitting)\n' "$file" "$lines"
    warnings=1
  fi
done < <(
  find "${roots[@]}" -type f -name '*.rs' -print0
)

if (( warnings )); then
  printf '\nWarning: some files exceed the 300-line soft threshold.\n'
  printf 'Consider splitting them to improve maintainability.\n'
fi

exit "$failed"
