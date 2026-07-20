#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test knowledge_upsert_transaction
cargo test -p lexflex-engine --test rollback_index_exactness

if rg -n 'self\.rebuild_hash\(\)\?' crates/lexflex-engine/src/knowledge/snapshot.rs; then
  echo "ERROR: snapshot transactions must rebuild candidate state, not self"
  exit 1
fi

if rg -n 'pub assertions|pub snapshot_hash' crates/lexflex-engine/src/knowledge/snapshot.rs; then
  echo "ERROR: KnowledgeSnapshot fields must remain private"
  exit 1
fi
