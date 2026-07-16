#!/usr/bin/env bash
set -euo pipefail

semantic_expression_count="$(
  rg -l --glob '*.rs' 'pub enum SemanticExpression' crates apps tools | wc -l
)"

if [[ "$semantic_expression_count" -ne 1 ]]; then
  echo "Expected one SemanticExpression enum, found $semantic_expression_count"
  exit 1
fi

semantic_type_count="$(
  rg -l --glob '*.rs' 'pub enum SemanticType' crates apps tools | wc -l
)"

if [[ "$semantic_type_count" -ne 1 ]]; then
  echo "Expected one SemanticType enum, found $semantic_type_count"
  exit 1
fi

if rg -n --glob '*.rs' '\bNormalizedExpression\b' crates apps tools; then
  echo "NormalizedExpression remains in active code"
  exit 1
fi
