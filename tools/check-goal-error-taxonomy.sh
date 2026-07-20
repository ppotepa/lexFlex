#!/usr/bin/env bash
set -euo pipefail

if rg -n 'GoalValidationError|NormalizationError' crates/lexflex-lingua/src/solve/goal_hash.rs | rg 'CanonicalHashError::Serialization'; then
  echo "ERROR: goal validation/normalization errors are flattened to canonical serialization"
  exit 1
fi

if rg -n 'NonBooleanExpression\(SemanticType::Concept\)|_ => GoalValidationError::NonBooleanExpression' crates/lexflex-lingua/src/solve; then
  echo "ERROR: goal type errors are flattened to Concept"
  exit 1
fi
