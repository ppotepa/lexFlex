#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-app --test exit_codes
cargo test -p lexflex-app --test validation_commands

if rg -n 'Cli(Error|Exit)::Command|Command\(String\)|Result<[^>]+, String>|map_err\(Cli(Error|Exit)::Command\)' apps/lexflex/src; then
  echo "ERROR: CLI must use typed input/local/internal/validation errors"
  exit 1
fi

if ! rg -n 'CliInternalError|ValidationCommandError' apps/lexflex/src/cli >/dev/null; then
  echo "ERROR: CLI internal and validation error types must be present"
  exit 1
fi
