#!/usr/bin/env bash
set -euo pipefail

rg -q 'max_derivations_per_item' crates/lexflex-parser/src/budget.rs
rg -q 'max_total_derivations' crates/lexflex-parser/src/budget.rs
rg -q 'application_attempt_count' crates/lexflex-parser/src/parser/chart_stage.rs
rg -q 'generated_derivation_count' crates/lexflex-parser/src/parser/chart_stage.rs

if rg -n 'max_alternative_derivations_per_item|AlternativeDerivationLimit' crates/lexflex-parser/src; then
  echo "ERROR: legacy parser derivation budget remains"
  exit 1
fi
