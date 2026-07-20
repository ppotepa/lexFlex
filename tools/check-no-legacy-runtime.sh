#!/usr/bin/env bash
set -euo pipefail

fail=0

roots=()

add_root() {
  local root="$1"
  if [[ -e "$root" ]]; then
    roots+=("$root")
  fi
}

add_root crates
add_root apps
add_root tools
add_root scripts

reject() {
  local pattern="$1"
  local description="$2"

  if rg -n \
    --glob '*.rs' \
    --glob '*.md' \
    --glob '*.sh' \
    --glob '*.toml' \
    "$pattern" \
    "${roots[@]}" \
    --glob '!target/**' \
    --glob '!.git/**' \
    --glob '!tools/check-no-legacy-runtime.sh'
  then
    echo
    echo "ERROR: $description"
    fail=1
  fi
}

reject 'lexflex-learner|lexlearn|lexflex-document-benchmark' 'legacy binary or workspace reference detected'
reject 'cargo run --[^\\n]* translate|cargo run --[^\\n]* chat|cargo run --[^\\n]* explain' 'legacy runtime command detected'
reject 'verify_21pts' 'legacy verification command detected'

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
