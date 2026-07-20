# LexFlex A2 Translation Report

## Batch

```text
Commit: 8df9109
Branch: agent/strict-composition-kernel-closure
Languages: English, Polish
Source cases: 50 EN + 50 PL
Executed translations: 100
```

## Result

Szczegółowa tabela każdego przypadku (źródło, oczekiwany tekst, rzeczywisty tekst,
hash semantyczny i status) znajduje się w [A2_TEST_CASES.md](A2_TEST_CASES.md).
Raport generowany bezpośrednio przez test jest zapisywany w
`target/lexflex-reports/a2.md` oraz `target/lexflex-reports/a2.json`.

| Check | Result |
| --- | ---: |
| English source cases | 50 / 50 |
| Polish source cases | 50 / 50 |
| Exact target realizations | 100 / 100 |
| Source/target semantic hash equality | 100 / 100 |
| Assertion cases | 76 |
| Goal cases | 24 |
| Failed cases | 0 |
| Pending cases toward 200 + 200 | 300 |

The matrix is executed through the production `lexflex-app` CLI. It does not
call generation or parsing helpers directly from the test.

## Covered Constructions

- `Satisfies` assertions;
- `Apply(CAPITAL)` with English and Polish genitive realization;
- `Apply(SEE_EVENT)` with agent and patient valency;
- English and Polish nominative/accusative person forms;
- source lexical aliases preserving canonical entity identity;
- subject questions (`Who` / `Kto`);
- postverbal object questions (`who` / `Kogo`);
- assertion and goal kind preservation;
- EN -> PL and PL -> EN translation;
- alpha-normalized goal hash comparison.

## Lexical Variants

The batch added source-language aliases which resolve to existing semantic
entities rather than creating duplicate entities:

| Semantic entity | English forms | Polish forms |
| --- | --- | --- |
| `TOM` | Tom, Thomas, Tommy | Tomek, Tomasz, Tomcio |
| `IZA` | Iza, Isabelle, Izzy | Iza, Izabela, Izka |

Target generation remains canonical and deterministic. For example, aliases
of `TOM` generate `Tom` in English and `Tomek` in Polish.

## Verification Contract

Each case verifies:

1. source text is accepted by the CLI parser;
2. source is lowered into the canonical interlingua;
3. target text is generated through the loaded target language model;
4. target text is reparsed by the CLI runtime;
5. assertion/goal kind remains unchanged;
6. source and target semantic hashes are equal;
7. generated text equals the fixture expectation.

For goals, comparison uses Lingua's alpha-normalized goal hash so parser-local
query variable IDs do not create false mismatches between languages.

## Commands

```bash
cargo fmt --all
cargo check --workspace --all-targets
cargo test -p lexflex-app --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p lexflex-app --quiet -- model-validate
cargo run -p lexflex-app --quiet -- language-validate
bash tools/check-generation-morphology.sh
bash tools/check-acceptance-matrix.sh
```

## Results

```text
cargo check                 PASS
cargo test -p lexflex-app   PASS (37 tests)
cargo clippy                PASS
model-validate              PASS
language-validate           PASS
generation gate             PASS
acceptance gate             PASS
git diff --check             PASS
```

## Remaining Work

The following are not counted as complete by this report:

- 200 EN and 200 PL source cases;
- concepts beyond `CAPITAL` and `SEE_EVENT`;
- fronted object questions (`Who does Tom see?`, `Kogo widzi Tomek?`);
- negation, coordination and quantification;
- broader tense, number, gender and case coverage;
- ambiguity alternatives in the translation API;
- document, conversation and learning-specific A2 fixtures.

The next batch must add new semantic or morphological coverage and must keep
all 100 existing cases green.
