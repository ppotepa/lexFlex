#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test formal_ambiguity
cargo test -p lexflex-engine --test formal_alternative_budget
cargo test -p lexflex-engine --test formal_alternative_fatality
cargo test -p lexflex-engine --lib runtime::alternative_failure::tests

if rg -n 'map_err\(\|error\| error\.to_string\(\)\)' crates/lexflex-engine/src/runtime/ambiguity.rs; then
  echo "ERROR: ambiguity lowering must not stringify formal failures before classification"
  exit 1
fi

if ! rg -n 'AlternativeFailure::Fatal|classify_alternative_failure|with_diagnostics' crates/lexflex-engine/src/runtime/ambiguity.rs crates/lexflex-engine/src/runtime/alternative_failure.rs >/dev/null; then
  echo "ERROR: ambiguity lowering must propagate fatal alternative failures with diagnostics"
  exit 1
fi
