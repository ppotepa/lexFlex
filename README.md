# lexFlex

A universal meaning representation framework built on **Interlingua**.

## Document Translation

Document translation is defined as a structured multi-sentence input unit, independent of physical pages and layout.

- Contract: [docs/document/README.md](./docs/document/README.md)
- Profile: [data/profiles/document_mvp_v1.ron](./data/profiles/document_mvp_v1.ron)
- Baseline: [benchmarks/document_v1/baseline/document_v1_baseline.json](./benchmarks/document_v1/baseline/document_v1_baseline.json)
- Status: contract/baseline phase

## How It Works

```text
Source Language -> [Parser] -> Interlingua -> [Generator] -> Target Language
```
