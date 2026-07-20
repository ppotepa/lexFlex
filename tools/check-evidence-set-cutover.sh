#!/usr/bin/env bash
set -euo pipefail

if rg -n \
  --glob '*.rs' \
  --glob '!crates/lexflex-model/src/evidence_set.rs' \
  'pub evidence:[[:space:]]*Vec<|evidence:[[:space:]]*Vec<Evidence>|BTreeMap<EvidenceId,[[:space:]]*Evidence>' \
  crates
then
  echo "ERROR: public/production evidence API bypasses EvidenceSet"
  exit 1
fi
