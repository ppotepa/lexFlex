#!/usr/bin/env bash
set -euo pipefail

failed=0

# This batch gate is intentionally scoped to the newly introduced runtime/core
# modules. The legacy monolith and older entrypoints remain out of scope until
# the final cutover batch.
roots=(
  crates/lexflex-engine/src
  crates/lexflex-language/src
  crates/lexflex-lingua/src
  crates/lexflex-model/src
  crates/lexflex-learning/src
  crates/lexflex-provider-llm/src
  crates/lexflex-store/src
  apps/lexflex/src
  tools/lexflex-benchmark/src
  tools/lexflex-model-inspect/src
)

while IFS= read -r -d '' file; do
  lines="$(wc -l < "$file")"

  if (( lines > 500 )); then
    printf 'ERROR: %s has %s lines\n' "$file" "$lines"
    failed=1
  fi
done < <(
  find "${roots[@]}" -type f -name '*.rs' -print0
)

exit "$failed"
