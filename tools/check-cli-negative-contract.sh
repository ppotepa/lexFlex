#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-app --test exit_codes

if rg -n 'Command::new\("cargo"\)|cargo run' apps/lexflex/tests; then
  echo "ERROR: CLI tests must execute CARGO_BIN_EXE_lexflex-app, not nested cargo"
  exit 1
fi
