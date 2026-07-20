# lexFlex

lexFlex is a data-driven semantic language runtime. Natural-language input is
tokenized and parsed into `LinguaExpression`, compiled and verified, then
lowered to the canonical `SemanticExpression` model. Knowledge mutations are
verified and persisted transactionally.

## Commands

Use `cargo run -p lexflex-app -- <command>`. Core commands include
`text-analyze`, `text-ingest`, `text-ask`, `text-generate`,
`text-translate`, `text-translate-text`, `document-ingest`,
`conversation-turn`, and the model/language validation commands.

## Validation

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

All `tools/check-*.sh` scripts are executable contract gates. Linguistic
knowledge belongs in `data/`; semantic shortcuts and sentence templates do
not belong in Rust code.
