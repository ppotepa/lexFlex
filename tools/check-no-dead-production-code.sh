#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r -d '' file; do
  if rg -q '#\[allow\(dead_code\)\]' "$file" || rg -q '#!\[allow\(dead_code\)\]' "$file"; then
    printf 'ERROR: %s contains #[allow(dead_code)] in production code\n' "$file"
    rg -n '#\[allow\(dead_code\)\]|#!\[allow\(dead_code\)\]' "$file"
    failed=1
  fi
done < <(
  find crates apps tools -name '*.rs' -type f ! -path '*/tests/*' ! -path '*/target/*' -print0
)

exit "$failed"
