#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test formal_alternative_budget

if rg -n 'pub max_formal' crates/lexflex-engine/src/runtime/text_budget.rs; then
  echo "ERROR: TextRuntimeBudget limits must be private validated fields"
  exit 1
fi

if ! rg -n 'max_formal_alternatives\(\)' crates/lexflex-engine/src/runtime/ambiguity.rs >/dev/null; then
  echo "ERROR: max_formal_alternatives must be wired in production ambiguity lowering"
  exit 1
fi
