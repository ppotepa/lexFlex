# Fronted Object Questions Report

The first fronted-question slice is executed through the production CLI and
reparses every generated target.

| Metric | Result |
| --- | ---: |
| English source cases | 6 / 6 |
| Polish source cases | 6 / 6 |
| Exact target realizations | 12 / 12 |
| Semantic hash equality | 12 / 12 |
| Assertion/goal kind preserved | 12 / 12 |
| Failed cases | 0 |

Cases cover the six English and Polish person aliases in both directions:

```text
Who does Tom see?  -> Tomek widzi Kogo?
Kogo widzi Tomek?  -> Tom sees who?
```

The generated per-case report is written to:

```text
target/lexflex-reports/fronted-questions.md
target/lexflex-reports/fronted-questions.json
```

The slice is intentionally separate from the 50+50 A2/B1 matrices. Its next
iteration will expand the corpus to 200 English and 200 Polish source cases
using additional valid lexical and argument-role variants.
