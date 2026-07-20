#!/usr/bin/env bash
set -euo pipefail

rg -q 'verify_with_catalog' crates/lexflex-model/src/assertion.rs
rg -q 'verify_with_catalog' crates/lexflex-engine/src/knowledge/snapshot.rs
rg -q 'verify_with_catalog' crates/lexflex-lingua/src/solve/solver.rs
rg -q 'state.verify' crates/lexflex-engine/src/session/runtime.rs
rg -q 'try_from_state' crates/lexflex-engine/src/runtime/mod.rs
