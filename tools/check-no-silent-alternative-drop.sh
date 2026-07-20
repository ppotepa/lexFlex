#!/usr/bin/env bash
set -euo pipefail

if rg -n \
  --glob '*.rs' \
  'Err\(_\)[[:space:]]*=>[[:space:]]*continue' \
  crates/lexflex-engine/src/runtime \
  crates/lexflex-parser/src
then
  echo "ERROR: invalid parse alternatives are silently discarded"
  exit 1
fi

if rg -n \
  --glob '*.rs' \
  '\.ok\(\)\?' \
  crates/lexflex-parser/src/chart \
  crates/lexflex-parser/src/parser
then
  echo "ERROR: parser composition drops errors with .ok()?"
  exit 1
fi

if rg -n \
  --glob '*.rs' \
  'alternative\.expression[^;]+BTreeMap::new\(\)' \
  crates/lexflex-engine/src/runtime
then
  echo "ERROR: ambiguous question alternative loses query context"
  exit 1
fi
