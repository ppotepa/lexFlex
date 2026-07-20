#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-model --test assertion_serde_integrity
cargo test -p lexflex-model --test assertion_verified_creation

if rg -n 'pub (id|expression|evidence|world|canonical_hash):' crates/lexflex-model/src/assertion.rs; then
  echo "ERROR: SemanticAssertion fields must be private"
  exit 1
fi

if rg -n 'pub fn create\([^)]*world:[^)]*\)[[:space:]]*->' crates/lexflex-model/src/assertion.rs; then
  echo "ERROR: public SemanticAssertion::create must require a catalog"
  exit 1
fi

if rg -n '#\[derive\([^]]*Deserialize[^]]*\)\][[:space:]]*pub struct SemanticAssertion' crates/lexflex-model/src/assertion.rs; then
  echo "ERROR: SemanticAssertion must use custom Deserialize"
  exit 1
fi

if ! rg -n "impl<'de> Deserialize<'de> for SemanticAssertion" crates/lexflex-model/src/assertion.rs >/dev/null; then
  echo "ERROR: SemanticAssertion must implement custom Deserialize"
  exit 1
fi
