# Document V1 Baseline

The baseline stores the frozen output of the current translator for the document corpus.
`document_v1_baseline.json` is the only canonical Document V1 baseline.

- Commit hash: recorded in the frozen run metadata
- Date created: recorded in the frozen run metadata
- Cases: recorded in the frozen run metadata
- Creation command: `cargo run -p lexflex-document-benchmark -- freeze ...`
- A weak result is expected
- Do not hand-edit the JSON

Update procedure:

1. create a new run
2. compare it to the baseline
3. analyze regressions
4. freeze explicitly
5. commit the baseline with rationale
