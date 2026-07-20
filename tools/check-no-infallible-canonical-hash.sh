#!/usr/bin/env bash
set -euo pipefail

if rg -n \
  --glob '*.rs' \
  'canonical hashing requires serializable value' \
  crates apps tools
then
  echo "ERROR: canonical identity can panic"
  exit 1
fi

if rg -n \
  --glob '*.rs' \
  'pub fn canonical_hash[^{]*->[[:space:]]*String' \
  crates/lexflex-model/src
then
  echo "ERROR: canonical_hash remains infallible"
  exit 1
fi
