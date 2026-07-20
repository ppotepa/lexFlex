#!/usr/bin/env bash
set -euo pipefail

for gate in \
  tools/check-transactional-knowledge.sh \
  tools/check-goal-success-tests.sh \
  tools/check-cli-negative-contract.sh \
  tools/check-engine-error-mapping.sh \
  tools/check-assertion-deserialize-integrity.sh \
  tools/check-category-transactionality.sh \
  tools/check-cli-typed-error-closure.sh \
  tools/check-complete-candidate-selection.sh \
  tools/check-evidence-value-integrity.sh \
  tools/check-formal-alternative-fatality.sh \
  tools/check-session-constructor-integrity.sh \
  tools/check-text-budget-closure.sh \
  tools/check-type-reference-closure.sh \
  tools/check-precompiled-model-context.sh \
  tools/check-text-analysis-integrity.sh \
  tools/check-runtime-error-mapping.sh \
  tools/check-rollback-index-consistency.sh \
  tools/check-acceptance-matrix.sh \
  tools/check-verified-execution-boundary.sh
do
  if ! test -f "$gate"; then
    echo "ERROR: missing critical gate $gate"
    exit 1
  fi

  if ! rg -n 'cargo test -p [^ ]+ --test ' "$gate" >/dev/null; then
    echo "ERROR: $gate must execute exact integration tests with --test"
    exit 1
  fi

  if rg -n 'cargo test .*[[:space:]][[:alnum:]_:]+[[:space:]]+--all-targets|cargo test .*--all-targets' "$gate"; then
    echo "ERROR: $gate must not rely on broad filters or all-targets"
    exit 1
  fi
done

for target in \
  apps/lexflex/tests/exit_codes.rs \
  crates/lexflex-engine/tests/formal_alternative_budget.rs \
  crates/lexflex-engine/tests/formal_alternative_fatality.rs \
  crates/lexflex-engine/tests/formal_ambiguity.rs \
  crates/lexflex-engine/tests/knowledge_upsert_transaction.rs \
  crates/lexflex-engine/tests/rollback_index_exactness.rs \
  crates/lexflex-engine/tests/knowledge_snapshot_serde.rs \
  crates/lexflex-engine/tests/runtime_error_mapping_matrix.rs \
  crates/lexflex-engine/tests/text_analysis_serde_integrity.rs \
  crates/lexflex-engine/tests/session_identity_integrity.rs \
  crates/lexflex-engine/tests/session_integrity.rs \
  crates/lexflex-lingua/tests/expression_annotation_type_references.rs \
  crates/lexflex-lingua/tests/precompiled_model_context.rs \
  crates/lexflex-model/tests/evidence_serde_integrity.rs \
  crates/lexflex-model/tests/source_span_integrity.rs \
  crates/lexflex-model/tests/security_boundary_matrix.rs \
  crates/lexflex-parser/tests/robustness_matrix.rs \
  crates/lexflex-generation/tests/lexical_generation.rs \
  crates/lexflex-parser/tests/parse_metrics.rs \
  crates/lexflex-lingua/tests/verified_execution_boundary.rs \
  crates/lexflex-lingua/tests/verified_model_identity.rs
do
  if ! test -f "$target"; then
    echo "ERROR: missing critical test target $target"
    exit 1
  fi
done
