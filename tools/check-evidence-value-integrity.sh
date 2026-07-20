#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-model --test source_span_integrity
cargo test -p lexflex-model --test evidence_serde_integrity
cargo test -p lexflex-model --test evidence_set_integrity

if rg -n 'pub (id|source_id|span|source_hash|start|end):' crates/lexflex-model/src/evidence.rs; then
  echo "ERROR: Evidence and SourceSpan fields must be private"
  exit 1
fi

if rg -n '#\[derive\([^]]*Deserialize[^]]*\)\][[:space:]]*pub struct (Evidence|SourceSpan)' crates/lexflex-model/src/evidence.rs; then
  echo "ERROR: Evidence and SourceSpan must use custom Deserialize"
  exit 1
fi
