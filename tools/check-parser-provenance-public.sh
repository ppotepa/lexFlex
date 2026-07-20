#!/usr/bin/env bash
set -euo pipefail

rg -q 'pub use explain::\{ApplicationRule, DerivationNode, DerivationSet\}' crates/lexflex-parser/src/lib.rs
rg -q 'pub derivations: DerivationSet' crates/lexflex-parser/src/output.rs
rg -q 'pub derivations: Option<DerivationSet>' crates/lexflex-engine/src/api/text.rs

if rg -n --glob '*.rs' 'derivation:[[:space:]]*Option<DerivationNode>|\.primary\(\)\.cloned\(\)' crates/lexflex-engine/src; then
  echo "ERROR: engine provenance is still primary-only"
  exit 1
fi
