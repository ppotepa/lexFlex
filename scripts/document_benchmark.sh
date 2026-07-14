#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/results/document-v1/$(date +%Y%m%d-%H%M%S)}"
CORPUS="${CORPUS:-$ROOT_DIR/benchmarks/document_v1}"
DATA_DIR="${DATA_DIR:-$ROOT_DIR/data}"

cd "$ROOT_DIR"
cargo run -p lexflex-document-benchmark -- validate --corpus "$CORPUS"
cargo run -p lexflex-document-benchmark -- run --corpus "$CORPUS" --data "$DATA_DIR" --split all --output "$OUT_DIR"
echo "$OUT_DIR/summary.txt"
