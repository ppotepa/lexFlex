#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test text_analysis_serde_integrity
