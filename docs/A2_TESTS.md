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
`apps/lexflex/tests/translation_matrix.rs`. It covers eight distinct
assertion translations:

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

## Planned 200-Case Distribution

The matrix grows by adding real cases, not by repeating identical commands.

| Group | Cases | Required coverage |
| --- | ---: | --- |
| Entity and lexical lookup | 20 | names, countries, cities, lexical senses |
| `Satisfies` assertions | 25 | subject/predicate and argument roles |
| `Apply` and valency | 25 | required slots, direction, surface relations |
| `Equals` | 15 | copula and natural realization |
| Event assertions | 25 | agent, patient, ordering, morphology |
| Questions and projections | 25 | assertion/goal kind and variables |
| Negation and coordination | 20 | `Not`, `And`, `Or` |
| Quantification | 10 | `Exists`, `ForAll` |
| English morphology | 15 | forms, agreement, query forms |
| Polish morphology | 20 | cases, gender, number, agreement |
| Cross-language round trips | 20 | EN -> PL -> EN and PL -> EN -> PL |
| **Total** | **220** | planned A2 coverage |

The first release gate is 200 passing cases. The additional 20 cases are
reserved for regressions discovered while implementing the matrix.

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

