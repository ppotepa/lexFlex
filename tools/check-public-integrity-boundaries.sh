#!/usr/bin/env bash
set -euo pipefail

if rg -n 'pub assertions:|pub snapshot_hash:|pub knowledge:|pub model_hash:|pub language_hash:' crates/lexflex-engine/src/knowledge crates/lexflex-engine/src/session; then
  echo "ERROR: knowledge/session integrity fields must not be publicly mutable"
  exit 1
fi

if rg -n 'pub use knowledge::\{[^}]*KnowledgeIndex|pub use index::KnowledgeIndex|pub struct KnowledgeIndex' crates/lexflex-engine/src; then
  echo "ERROR: KnowledgeIndex must remain internal derived state"
  exit 1
fi

if rg -n 'pub fn verify\(&self\) -> Result<\(\), KnowledgeSnapshotError>' crates/lexflex-engine/src/knowledge; then
  echo "ERROR: public KnowledgeSnapshot verify must require a catalog"
  exit 1
fi
