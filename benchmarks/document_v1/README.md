# Document V1 Benchmark

This corpus measures whole-document translation behavior for the Document MVP contract.

It differs from `scripts/bulk_test.sh` and `benchmarks/benchmark_sentences.txt` in three ways:

- it uses full multi-sentence documents with paragraph boundaries
- it evaluates semantic invariants, glossary constraints, and determinism
- it freezes a baseline that records current behavior honestly, including failures

The corpus is versioned and loaded through `manifest.ron`; the runner does not scan directories heuristically.

Canonical baseline:
- `baseline/document_v1_baseline.json`
