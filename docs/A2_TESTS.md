# A2 Test Matrix

This document is the working acceptance matrix for incremental English and
Polish development. A case is complete only when the source is parsed into
the interlingua, the target is generated, and reparsing the target produces
the same canonical semantic hash.

## Case Contract

Every case records:

- source and target language;
- source text and expected target text;
- assertion or goal kind;
- expected semantic expression/hash;
- source parser metadata and diagnostics;
- generation trace and selected morphology;
- target reparse result and semantic hash.

Expected output text alone is insufficient evidence.

## Current Seed

The first executable seed is in
`apps/lexflex/tests/translation_matrix.rs`. It covers twelve distinct
assertion and goal translations:

| ID | Source | Target | Construction | Status |
| --- | --- | --- | --- | --- |
| A2-001 | Paris is the capital of France. | Paryż jest stolicą Francji | `Satisfies` + `CAPITAL` | PASS |
| A2-002 | Paryż jest stolicą Francji. | Paris is the capital of France | `Satisfies` + `CAPITAL` | PASS |
| A2-003 | Warsaw is the capital of Poland. | Warszawa jest stolicą Polski | `Satisfies` + genitive | PASS |
| A2-004 | Warszawa jest stolicą Polski. | Warsaw is the capital of Poland | `Satisfies` + genitive | PASS |
| A2-005 | Tom sees Iza. | Tomek widzi Izę | `SEE_EVENT` agent/patient | PASS |
| A2-006 | Tomek widzi Izę. | Tom sees Iza | `SEE_EVENT` agent/patient | PASS |
| A2-007 | Iza sees Tom. | Iza widzi Tomka | `SEE_EVENT` object inflection | PASS |
| A2-008 | Iza widzi Tomka. | Iza sees Tom | `SEE_EVENT` object inflection | PASS |
| A2-009 | What is the capital of Poland? | Jaka jest stolica Polski? | goal projection + `CAPITAL` | PASS |
| A2-010 | Jaka jest stolica Polski? | What is the capital of Poland? | goal projection + `CAPITAL` | PASS |
| A2-011 | Who sees Tom? | Kto widzi Tomka? | goal projection + agent | PASS |
| A2-012 | Kto widzi Tomka? | Who sees Tom? | goal projection + agent | PASS |
| A2-013 | Tom sees who? | Tomek widzi Kogo? | goal projection + patient | PASS |
| A2-014 | Tomek widzi Kogo? | Tom sees who? | goal projection + patient | PASS |

The executable matrix test `a2_reaches_fifty_english_and_polish_source_sentences`
adds 36 event combinations, six agent questions, six patient questions and
two capital assertions per source language. All 100 generated cases use the
CLI translation path and compare source/target semantic hashes.

## Planned 200 + 200 Distribution

The acceptance target is 200 source sentences in English and 200 source
sentences in Polish. A translation case is counted once for its source
language and once when the reverse-language fixture is executed. The matrix
grows by adding real cases, not by repeating identical commands.

| Group | Cases | Required coverage |
| --- | ---: | --- |
| Entity and lexical lookup | 20 + 20 | names, countries, cities, lexical senses |
| `Satisfies` assertions | 25 + 25 | subject/predicate and argument roles |
| `Apply` and valency | 25 + 25 | required slots, direction, surface relations |
| `Equals` | 15 + 15 | copula and natural realization |
| Event assertions | 25 + 25 | agent, patient, ordering, morphology |
| Questions and projections | 25 + 25 | assertion/goal kind and variables |
| Negation and coordination | 20 + 20 | `Not`, `And`, `Or` |
| Quantification | 10 + 10 | `Exists`, `ForAll` |
| English/Polish morphology | 15 + 15 | forms, agreement, query forms |
| Cross-language round trips | 20 + 20 | EN -> PL -> EN and PL -> EN -> PL |
| **Total** | **200 + 200** | A2 acceptance target |

## Failure Classification

Each failing case must be assigned to exactly one layer before implementation:

1. semantic model or concept missing;
2. language lexicon or lexical sense missing;
3. paradigm/form/morphology missing;
4. valency or surface relation missing;
5. parser/category/composition missing;
6. interlingua metadata lost;
7. generation/linearization missing;
8. CLI or persistence defect.

The fix belongs in the owning layer. Sentence-specific branches and literal
surface shortcuts are prohibited.

## Execution

```bash
rtk cargo test -p lexflex-app --test translation_matrix
rtk cargo run -p lexflex-app -- text-analyze --language pl --text 'Paryż jest stolicą Francji.'
rtk cargo run -p lexflex-app -- text-translate-text \
  --language pl --target-language en \
  --text 'Paryż jest stolicą Francji.'
```
