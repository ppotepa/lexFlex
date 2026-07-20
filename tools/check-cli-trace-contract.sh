#!/usr/bin/env bash
set -euo pipefail

rg -q 'include_trace' apps/lexflex/src/cli/lingua_eval.rs
rg -q 'serde_json::to_string_pretty\(&result\)' apps/lexflex/src/cli/lingua_eval.rs
rg -q 'serde_json::to_string_pretty\(&result.value\)' apps/lexflex/src/cli/lingua_eval.rs
