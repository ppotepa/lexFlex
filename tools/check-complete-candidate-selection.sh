#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-parser --test parse_metrics
cargo test -p lexflex-parser --test query_type_propagation

if ! rg -n 'classify_complete_item' crates/lexflex-parser/src/parser/finish_stage.rs crates/lexflex-parser/src/parser/complete_candidate.rs >/dev/null; then
  echo "ERROR: complete items must be classified before grouping/scoring"
  exit 1
fi

if ! rg -n 'complete_candidate_count|complete_rejection_count' crates/lexflex-parser/src/parser/finish_stage.rs >/dev/null; then
  echo "ERROR: complete candidate metrics must be written in finish stage"
  exit 1
fi
