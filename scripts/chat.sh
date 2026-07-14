#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

export LANG=C.UTF-8
export LC_ALL=C.UTF-8
export LC_CTYPE=C.UTF-8

export LEXFLEX_LLM_BASE_URL="${LEXFLEX_LLM_BASE_URL:-http://192.168.195.88:1234/v1}"
export LEXFLEX_LLM_MODEL="${LEXFLEX_LLM_MODEL:-google/gemma-4-e2b}"
export LEXFLEX_LLM_NOTHINK="${LEXFLEX_LLM_NOTHINK:-1}"
export LEXFLEX_NO_AUTO_WRITE="${LEXFLEX_NO_AUTO_WRITE:-1}"

exec cargo run --quiet -- chat "$@"
