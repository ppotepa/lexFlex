#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-lingua --test precompiled_model_context
if rg -n 'base_declarations\(\)\.to_vec\(\)' crates/lexflex-engine/src crates/lexflex-lingua/src; then
  echo "ERROR: request evaluation must not clone base declarations"
  exit 1
fi
