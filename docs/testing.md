# Testing

The normal validation sequence is:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

`tools/check-*.sh` executes the repository gates. Tests cover semantic hashes,
verified model loading, parser budgets, candidate rejection, transactional
knowledge mutations, session corruption, document provenance, conversation
reload, learning approval, provider replay, generation, translation, and CLI
contracts. Private invariants are tested inside library modules rather than by
making internal constructors public.

Remaining release work is an explicit property/fuzz matrix for deeply nested
serde and parser inputs.
