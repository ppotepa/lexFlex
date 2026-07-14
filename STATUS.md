# Status

The current repository contains a working offline-first runtime for Polish and English:

- deterministic `ConversationEngine` request/response boundary;
- local-first Wikipedia source provider;
- document compilation, graph, entity resolution and knowledge extraction;
- evidence-backed natural-language answers;
- CLI and TUI clients;
- immutable session snapshots and JSONL traces;
- workspace checks and test suites passing.

The implementation is an MVP. Extraction quality is corpus-dependent and the supported language/query surface is intentionally narrower than the long-term architecture described in the reference documents.
