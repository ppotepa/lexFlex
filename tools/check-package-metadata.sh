#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r file; do
  if ! rg -q 'version\.workspace = true' "$file"; then
    echo "ERROR: $file does not inherit version"
    failed=1
  fi

  if ! rg -q 'edition\.workspace = true' "$file"; then
    echo "ERROR: $file does not inherit edition"
    failed=1
  fi

  if ! rg -q 'publish\.workspace = true' "$file"; then
    echo "ERROR: $file does not inherit publish"
    failed=1
  fi

  if ! rg -q 'license\.workspace = true' "$file"; then
    echo "ERROR: $file does not inherit license"
    failed=1
  fi
done < <(find apps crates tools -name Cargo.toml -type f | sort)

versions="$(
  cargo tree -d 2>/dev/null || true \
    | rg '^thiserror v' \
    | sort -u \
    | wc -l
)"

if [[ "$versions" -gt 1 ]]; then
  echo "ERROR: multiple thiserror versions"
  failed=1
fi

if rg -n \
  --glob 'Cargo.toml' \
  'serde[[:space:]]*=[[:space:]]*\{|serde_json[[:space:]]*=[[:space:]]*"|sha2[[:space:]]*=[[:space:]]*"|thiserror[[:space:]]*=[[:space:]]*"|ron[[:space:]]*=[[:space:]]*"|clap[[:space:]]*=[[:space:]]*\{' \
  apps crates tools
then
  echo "ERROR: shared dependency version duplicated outside workspace.dependencies"
  failed=1
fi

exit "$failed"
