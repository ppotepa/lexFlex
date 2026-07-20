#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings

cargo run -p lexflex-benchmark --release -- --iterations 1 --max-p95-ns 1000000000000 >/dev/null

if git ls-files 'data/sessions/**' 'data/sources/**' | grep -q .; then
    echo "release acceptance: tracked runtime data found" >&2
    exit 1
fi

echo "release acceptance: PASS"
