#!/usr/bin/env bash
set -euo pipefail

failed=0

reject() {
  local pattern="$1"
  local message="$2"

  if rg -n \
    --glob '*.rs' \
    "$pattern" \
    crates/lexflex-parser/src \
    crates/lexflex-engine/src/runtime
  then
    echo "ERROR: $message"
    failed=1
  fi
}

reject \
  'apply_lambda_expression|reduce_lambda_expression' \
  'parser-local lambda reduction remains'

reject \
  'substitute_variable|substitute_expression_symbol' \
  'parser-local symbol substitution remains'

reject \
  'lower_text_expression|evaluate_text_expression' \
  'engine-local Lingua evaluator remains'

reject \
  'unsupported residual lexical expression' \
  'manual residual-expression conversion remains'

if [[ "$failed" -ne 0 ]]
then
  exit 1
fi
