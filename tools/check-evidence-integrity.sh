#!/usr/bin/env bash
set -euo pipefail

if rg -n \
  --glob '*.rs' \
  'pub evidence:[[:space:]]*Vec<[[:space:]]*Evidence[[:space:]]*>' \
  crates
then
  echo "ERROR: evidence remains order-dependent Vec"
  exit 1
fi

if rg -n \
  --glob '*.rs' \
  'Evidence[[:space:]]*\{' \
  crates/lexflex-engine/src \
  apps/lexflex/src
then
  echo "ERROR: production creates Evidence with unchecked struct literal"
  exit 1
fi
