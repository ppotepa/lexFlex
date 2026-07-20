#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test mutation_rollback

if ! rg -n 'ask_capital|query_before|query_after' crates/lexflex-engine/tests/mutation_rollback.rs >/dev/null; then
  echo "ERROR: rollback tests must query the knowledge index before and after failure"
  exit 1
fi
