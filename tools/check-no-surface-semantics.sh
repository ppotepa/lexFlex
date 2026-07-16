#!/usr/bin/env bash
set -euo pipefail

roots=(
  crates/lexflex-model/src
  crates/lexflex-lingua/src
  crates/lexflex-language/src
  crates/lexflex-engine/src
  apps/lexflex/src
  tools/lexflex-model-inspect/src
)

fail=0

check() {
  local pattern="$1"
  local description="$2"

  if rg -n --glob '*.rs' "$pattern" "${roots[@]}"; then
    echo
    echo "ERROR: $description"
    fail=1
  fi
}

check '\bQueryPattern\b|\bQueryProjection\b' 'old query model detected'
check '\bAssertExpression\b|\bApplyExpression\b' 'old semantic expression model detected'
check 'question-capital|question-population' 'domain-specific question pattern detected'
check 'answer_[a-z_]+_question' 'concept-specific question handler detected'
check 'extract_[a-z_]+_scope' 'concept-specific extractor detected'
check 'source_sentence_answer' 'raw source answer fallback detected'
check 'entity_aliases' 'hardcoded entity aliases detected'
check 'seed_en\(|seed_pl\(|seed_for\(' 'hardcoded language seeds detected'
check 'pub[[:space:]]+fn[[:space:]]+of\(' 'global of() semantic binding detected'
check 'template:[[:space:]]*String' 'whole sentence template detected'
check 'pattern:[[:space:]]*Vec<String>' 'whole phrase pattern detected'
check 'QueryConstraint::TextContains' 'text containment used as semantic query'
check 'match[[:space:]]+[^\n]*concept[^\n]*as_str' 'concept string dispatch detected'

if rg -n --glob '*.rs' '"CAPITAL"|"POPULATION"|"LOCATED_IN"|"IS_A"' \
  crates/lexflex-lingua/src crates/lexflex-engine/src
then
  echo
  echo "ERROR: domain concept leaked into runtime."
  fail=1
fi

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
