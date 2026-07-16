#!/usr/bin/env bash
set -euo pipefail

if rg -n 'apply_lambda_expression|substitute_variable|lower_text_expression' \
  crates/lexflex-parser/src \
  crates/lexflex-engine/src
then
  echo "ERROR: manual meaning evaluation remains"
  exit 1
fi
