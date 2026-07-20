#!/usr/bin/env bash
set -euo pipefail

if rg -n 'SemanticType::Unknown|RuntimeValue::Unknown|CategoryType::Unknown' \
  crates/lexflex-model/src \
  crates/lexflex-language/src \
  crates/lexflex-lingua/src \
  crates/lexflex-engine/src \
  crates/lexflex-parser/src
then
  echo "ERROR: semantic Unknown/wildcard remains"
  exit 1
fi
