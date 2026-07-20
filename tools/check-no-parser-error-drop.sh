#!/usr/bin/env bash
set -euo pipefail

if rg -n --glob '*.rs' '\.ok\(\)\?' crates/lexflex-parser/src/chart crates/lexflex-parser/src/parser; then
  echo "ERROR: parser chart/parser code drops errors with .ok()?"
  exit 1
fi

if rg -n --glob '*.rs' 'Result<[[:space:]]*\(\),[[:space:]]*\(\)>' crates/lexflex-parser/src/category crates/lexflex-parser/src/chart; then
  echo "ERROR: parser category/chart code uses untyped Result<(), ()>"
  exit 1
fi

if rg -n --glob '*.rs' 'Err\(_[^)]*\)[[:space:]]*=>[[:space:]]*CategoryOutcome::NotApplicable' crates/lexflex-parser/src/category; then
  echo "ERROR: category invariant errors are flattened to NotApplicable"
  exit 1
fi
