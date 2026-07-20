# Coordination Iteration Status

The first `And` slice is now compositional in both language packages. The
conjunction lexical sense combines two Boolean sentence meanings into the
canonical `And` semantic node.

| Contract | Result |
| --- | ---: |
| English `and` parses to `And` | PASS |
| Polish `i` parses to `And` | PASS |
| EN → PL translation | PASS |
| PL → EN translation | PASS |
| Target reparsing | PASS |
| Semantic hash preservation | PASS |
| Deterministic canonical member order | PASS |
| Full 200 EN / 200 PL corpus | PENDING |

Current verified cases:

```text
Tom sees Iza and Iza sees Tom.
-> Iza widzi Tomka i Tomek widzi Izę

Tomek widzi Izę i Iza widzi Tomka.
-> Iza sees Tom and Tom sees Iza
```

The canonical order is intentional: source member order is normalized before
target generation, while the semantic hash remains stable in both directions.
