# LexFlex B1 Translation Report

## Result

The B1 batch runs through the production `lexflex-app` translation command.
Each case reparses the generated target and compares source and target
semantic hashes.

| Metric | Result |
| --- | ---: |
| English source cases | 50 / 50 |
| Polish source cases | 50 / 50 |
| Exact target realizations | 100 / 100 |
| Semantic hash equality | 100 / 100 |
| Failed cases | 0 |

The detailed generated report is written to:

```text
target/lexflex-reports/b1.md
target/lexflex-reports/b1.json
```

## Covered Difficulty

- all agent/patient combinations for the six supported person aliases;
- English and Polish source aliases resolving to canonical entities;
- Polish nominative and accusative forms;
- English and Polish subject questions;
- English and Polish postverbal object questions;
- capital assertions with genitive realization;
- assertion and goal semantic-kind preservation;
- punctuation variants accepted by the parser;
- exact target text and semantic round-trip validation.

## Current Boundary

This batch does not claim support for negation, coordination, quantifiers,
relative clauses, or pronoun resolution. Those require complete language data
and parser/generator paths before they can be added as passing cases.
