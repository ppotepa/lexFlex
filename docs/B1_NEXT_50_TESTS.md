# B1 Next 50 Tests

This matrix contains 25 semantic scenarios in both directions: 50 English
source cases and 50 Polish source cases.

Every case uses the production `text-translate-text` CLI path and requires:

1. source parsing;
2. interlingua analysis;
3. target generation;
4. target reparsing;
5. equal source and target semantic hashes;
6. exact target text.

The matrix covers:

- 5 prefix-negation assertions;
- 5 subject questions;
- 5 postverbal object questions;
- 5 fronted-object questions;
- 2 boolean questions;
- 2 coordination cases (`And` and `Or`);
- 1 boolean-scope case.

Natural auxiliary negation (`does not`) and Polish negative case government are
not included. They require a separate lexical-composition change and must not
be represented as passing cases until their generated text reparses to the
same semantic hash.

The executable test is:

```text
cargo test -p lexflex-app --test translation_matrix \
  b1_next_fifty_per_language_cover_compositional_variants
```
