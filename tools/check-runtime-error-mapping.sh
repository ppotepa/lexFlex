#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test runtime_error_mapping_matrix
cargo test -p lexflex-engine --lib runtime::error_mapping::tests
cargo test -p lexflex-engine --test lingua_runtime

if rg -n 'EngineError::Runtime\(_\)[[:space:]]*=>[[:space:]]*EngineErrorCode::RuntimeBudget' crates/lexflex-engine/src/runtime; then
  echo "ERROR: RuntimeError variants must not be flattened to RuntimeBudget"
  exit 1
fi

if rg -n 'code: EngineErrorCode::InvalidGoal' crates/lexflex-engine/src/runtime/text_ask.rs; then
  echo "ERROR: text ask solve failures must use central error mapping"
  exit 1
fi
