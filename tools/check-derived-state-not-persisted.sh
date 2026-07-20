#!/usr/bin/env bash
set -euo pipefail

if rg -n 'pub knowledge_index:' crates/lexflex-engine/src/session/state.rs
then
  echo "ERROR: derived knowledge index is persisted"
  exit 1
fi

if rg -n 'filter_map' crates/lexflex-engine/src/runtime crates/lexflex-engine/src/knowledge
then
  echo "ERROR: missing indexed assertions may be hidden"
  exit 1
fi
