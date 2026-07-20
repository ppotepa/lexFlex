#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test session_integrity
cargo test -p lexflex-engine --test session_identity_integrity
cargo test -p lexflex-engine --test session_hash_corruption

if rg -n 'from_verified_state' crates/lexflex-engine/src; then
  echo "ERROR: RuntimeSession constructor must verify state instead of trusting a name"
  exit 1
fi

if ! rg -n 'validate_session_id' crates/lexflex-engine/src/session/state.rs crates/lexflex-store/src/lib.rs >/dev/null; then
  echo "ERROR: engine and store must share session id validation"
  exit 1
fi
