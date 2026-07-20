#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test event_cross_language
cargo test -p lexflex-engine --test lingua_runtime
cargo test -p lexflex-engine --test session_hash_corruption

if rg -n 'EngineError::Runtime\(_\)[[:space:]]*=>[[:space:]]*EngineErrorCode::RuntimeBudget' crates/lexflex-engine/src/runtime; then
  echo "ERROR: runtime errors must be mapped by variant and origin"
  exit 1
fi

if rg -n 'Err\(error\) => EngineResponse::Error \{[[:space:]]*code: EngineErrorCode::InvalidGoal' crates/lexflex-engine/src/runtime; then
  echo "ERROR: runtime solver errors must use central error mapping"
  exit 1
fi

if rg -n 'map_err\([^)]*InternalInvariant' crates/lexflex-engine/src/runtime/session_ops.rs crates/lexflex-engine/src/runtime/persist.rs; then
  echo "ERROR: persistence/session errors must use central knowledge mapping"
  exit 1
fi
