#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-generation --test lexical_generation polish_case_features_select_declared_morphological_forms
