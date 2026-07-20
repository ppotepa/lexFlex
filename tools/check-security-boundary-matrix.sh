#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-model --test security_boundary_matrix
