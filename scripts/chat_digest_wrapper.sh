#!/bin/bash
# Non-interactive wrapper mirroring chat.sh phase-1 digest invocation.
set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"
INPUT="${1:-Tomek poszedł do sklepu. Kupił mleko.}"
LANG="${2:-pl}"
cargo build --quiet --bin lexflex 2>/dev/null || cargo build --bin lexflex
./target/debug/lexflex explain "$INPUT" --lang "$LANG" --format human 2>/dev/null