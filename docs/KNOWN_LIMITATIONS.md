# Known limitations

This document describes the current MVP boundary, not the long-term design target.

## Languages

The runtime supports Polish (`pl`) and English (`en`). Other languages are not registered in the current CLI and parser runtime.

## Sources

Wikipedia is the first built-in source provider. The default workflow is local snapshot-only and offline-capable. Live fetching is explicit and requires `--live`. Arbitrary web pages and general web search are outside the current source contract.

## Knowledge QA

Questions are answered only over sources already ingested into the current session. The query surface is intentionally limited to the implemented `QuestionSemantics` and `QueryInterlingua` operations. Unsupported constructions return an explicit unsupported/unknown result instead of a generated fallback.

Extraction quality depends on parser coverage, graph construction and entity resolution. Long Wikipedia articles may contain ambiguous or noisy claims; evidence and diagnostics should be inspected for important results.

## Translation

Translation is deterministic and Interlingua-based. The supported grammar and lexicon are not a complete model of Polish or English, so complex syntax, rare morphology, idioms and stylistic nuance can be incomplete or awkward.

## Runtime boundaries

- No LLM is required by the core runtime.
- No answer is produced without evidence.
- No global cross-session entity merge is performed.
- Session snapshots are local files; distributed storage is not implemented.
- The TUI is a thin client and does not provide a separate semantic fallback.

The authoritative behavior is covered by CLI examples, engine contract tests and benchmark fixtures under `tests/` and `benchmarks/`.
