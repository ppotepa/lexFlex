# B1 Next 50 Summary

| Metric | Result |
| --- | ---: |
| English source cases | 25 / 25 |
| Polish source cases | 25 / 25 |
| Total cases | 50 / 50 |
| Exact target text | 50 / 50 |
| Semantic hash equality | 50 / 50 |
| Failed cases | 0 |

The generated machine-readable and Markdown report is written to:

```text
target/lexflex-reports/b1-next-fifty.json
target/lexflex-reports/b1-next-fifty.md
```

Alias normalization is intentional. For example, `Thomas`, `Tommy`,
`Tomasz`, and `Tomcio` resolve to the canonical `Tom`/`Tomek` entities before
target realization. The report compares the canonical semantic hash, not the
surface alias.
