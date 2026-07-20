#!/usr/bin/env bash
set -euo pipefail

cargo test -p lexflex-parser --test category_substitution
cargo test -p lexflex-parser --test category_unification
cargo test -p lexflex-parser --lib category::unify::tests

if rg -n 'CategoryTypeVariableId::new_unchecked\([^)]*(message|to_string|apply-category-resolution)' crates/lexflex-parser/src/category; then
  echo "ERROR: category errors must not be encoded as fake variable IDs"
  exit 1
fi

if rg -n 'pub\(crate\) fn (alias|bind_concrete|require_concrete|resolve|apply_category|apply_query_category_types|unresolved_query_type_count).*ParseError|fn (alias|bind_concrete|require_concrete|resolve|apply_category|apply_query_category_types|unresolved_query_type_count).*ParseError' crates/lexflex-parser/src/category/substitution.rs; then
  echo "ERROR: category mutation APIs must not use ParseError"
  exit 1
fi

if ! rg -n 'function_late_mismatch_does_not_mutate_substitution|successful_unification_commits_substitution' crates/lexflex-parser/src/category/unify.rs >/dev/null; then
  echo "ERROR: category unification must have transactionality unit proofs"
  exit 1
fi
