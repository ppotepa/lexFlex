#!/usr/bin/env bash
set -euo pipefail

if rg -n 'Paris is the|Paryż jest|What is the|Jaka jest|\{subject\}.*\{scope\}' data/languages; then
  echo "Whole sentence patterns detected"
  exit 1
fi

if rg -n 'question-capital|question-population' data/languages; then
  echo "Domain-specific question construction detected"
  exit 1
fi
