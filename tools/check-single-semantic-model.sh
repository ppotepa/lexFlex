#!/usr/bin/env bash
set -euo pipefail

semantic_expression_count="$(
  rg -l --glob '*.rs' 'pub enum SemanticExpression' crates apps tools | wc -l
)"

if [[ "$semantic_expression_count" -ne 1 ]]; then
  echo "Expected one SemanticExpression enum, found $semantic_expression_count"
  exit 1
fi

semantic_type_count="$(
  rg -l --glob '*.rs' 'pub enum SemanticType\b' crates apps tools | wc -l
)"

if [[ "$semantic_type_count" -ne 1 ]]; then
  echo "Expected one SemanticType enum, found $semantic_type_count"
  exit 1
fi

if rg -n --glob '*.rs' '\bNormalizedExpression\b' crates apps tools; then
  echo "NormalizedExpression remains in active code"
  exit 1
fi

type_relation_count="$(
  rg -l --glob '*.rs' 'pub struct TypeRelation' crates apps tools | wc -l
)"

if [[ "$type_relation_count" -ne 1 ]]; then
  echo "Expected exactly one TypeRelation"
  exit 1
fi

if rg -n --glob '*.rs' '#\\[serde\\(transparent\\)\\]' crates/lexflex-model/src/id.rs; then
  echo "ERROR: canonical model IDs bypass validated serde"
  exit 1
fi

if rg -n \
  --glob '*.rs' \
  'impl From<&str> for (AssertionId|ConceptId|EntityId|EvidenceId|LanguageId|ModelPackageId|ParameterId|QualifierId|VariableId|WorldId)' \
  crates/lexflex-model/src
then
  echo "ERROR: unchecked From<&str> remains for canonical IDs"
  exit 1
fi
