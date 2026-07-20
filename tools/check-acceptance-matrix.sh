#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-engine --test boolean_text_contract
cargo test -p lexflex-engine --test event_cross_language

for pattern in \
  'ingest_polish_and_ask_english_returns_tom' \
  'multiple_event_answers_are_deterministic' \
  'wrong_polish_case_does_not_mutate_session' \
  'answer_evidence_uses_statement_source_not_question_source'
do
  if ! rg -n "$pattern" crates/lexflex-engine/tests/event_cross_language.rs >/dev/null; then
    echo "ERROR: missing acceptance scenario $pattern"
    exit 1
  fi
done
