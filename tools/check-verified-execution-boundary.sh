#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-lingua --test verified_execution_boundary
cargo test -p lexflex-lingua --test verified_model_identity

if rg -n 'pub fn execute(_with_policy)?[^\n]*CompiledProgram|pub fn execute_verified' \
    crates/lexflex-lingua/src; then
    echo "ERROR: public unverified interpreter entry remains"
    exit 1
fi

if rg -n 'validate_concept_programs|compile\(program\)' \
    crates/lexflex-engine/src/catalog crates/lexflex-engine/src/catalog/model_loader.rs; then
    echo "ERROR: legacy compile-each-program validator remains"
    exit 1
fi

echo "verified execution boundary: PASS"
