#!/usr/bin/env bash
set -euo pipefail

required_docs=(
  README.md
  docs/architecture.md
  docs/semantic-model.md
  docs/lingua.md
  docs/language-packages.md
  docs/parser.md
  docs/generation.md
  docs/translation.md
  docs/knowledge.md
  docs/evidence.md
  docs/sessions.md
  docs/conversation.md
  docs/documents.md
  docs/learning.md
  docs/providers.md
  docs/error-codes.md
  docs/cli.md
  docs/testing.md
  docs/benchmarks.md
  docs/migrations.md
)

for path in "${required_docs[@]}"; do
  test -f "$path" || { echo "missing documentation: $path" >&2; exit 1; }
done

test -z "$(git ls-files 'data/sessions/**' 'data/sources/**')"
git diff --check
for command in \
  TextAnalyze TextIngest TextAsk TextGenerate SemanticRealize TextTranslateText \
  DocumentIngest DocumentRemove DocumentReplace DocumentInspect DocumentQuery DocumentAnalyze ProviderIngest ConversationTurn ConversationTextTurn ConversationInspect ConversationResolve \
  LearningObserve LearningPropose LearningApprove LearningReject LearningPromote \
  ModelValidate LanguageValidate SessionInspect SessionClear SessionMigrate
do
  rg -n "^[[:space:]]*${command}[[:space:]]*(\{|,)" apps/lexflex/src/main.rs >/dev/null || {
    echo "missing CLI command variant: $command" >&2
    exit 1
  }
done
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings

for gate in tools/check-*.sh; do
  bash "$gate"
done

cargo run -p lexflex-app --quiet -- model-validate >/dev/null
cargo run -p lexflex-app --quiet -- language-validate >/dev/null
cargo run -p lexflex-benchmark --release -- --iterations 1 --max-p95-ns 1000000000000 >/dev/null

echo "RELEASE_AUDIT=PASS"
