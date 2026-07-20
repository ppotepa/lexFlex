#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-lingua --test goal_hash

if rg -n 'assert_eq!\([^;]*canonical_goal_semantic_hash' crates/lexflex-lingua/src/solve/goal_hash.rs crates/lexflex-lingua/tests/goal_hash.rs; then
  echo "ERROR: goal hash tests must unwrap expected-success hashes before comparing"
  exit 1
fi
