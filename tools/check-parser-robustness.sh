#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-parser --test robustness_matrix
