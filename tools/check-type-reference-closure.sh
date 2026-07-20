#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-model --test semantic_type_reference
cargo test -p lexflex-lingua --test goal_validation
cargo test -p lexflex-lingua --test expression_annotation_type_references
cargo test -p lexflex-language --test meaning_validation

if rg -n 'fn validate_value_type' crates/lexflex-language/src/validation.rs; then
  echo "ERROR: language validation must not duplicate recursive semantic type validation"
  exit 1
fi

if ! rg -n 'validate_semantic_type_references\(semantic_type, catalog\)' crates/lexflex-language/src/validation.rs >/dev/null; then
  echo "ERROR: language semantic type checks must delegate to the model validator"
  exit 1
fi

if ! rg -n 'validate_semantic_type_references' crates/lexflex-lingua/src/compiler crates/lexflex-language/src/validation.rs >/dev/null; then
  echo "ERROR: compiler and language layers must call the model semantic type validator"
  exit 1
fi
