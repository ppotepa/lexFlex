# CLI Contract

The application emits JSON on stdout and human-readable diagnostics on stderr.
Input files are read explicitly; no command silently mutates a state directory.

Core commands include `text-analyze`, `text-ingest`, `text-ask`,
`text-generate`, `text-translate`, and `text-translate-text`. Context commands are explicit:

- `document-ingest DOCUMENT_ID FILE [--store PATH]` segments and verifies a
  document, optionally committing it through the atomic `DocumentStore` file
- `document-query EXPRESSION --store PATH` returns document segments whose
  verified semantic analysis matches the canonical expression
- `document-replace DOCUMENT_ID FILE --store PATH` atomically replaces an
  existing document and refreshes its source provenance
- `document-analyze --store PATH --language ID` analyzes every document segment
  through the engine and commits all canonical analyses atomically
- `conversation-text-turn TURN_ID --language ID [--text TEXT|--file PATH]`
  parses a natural-language turn before committing it to verified conversation state
- `conversation-resolve ENTITY_TYPE --state PATH` returns the deterministic
  reference-resolution result, including ambiguity instead of guessing
- `provider-ingest DOCUMENT_ID ARTIFACT --store PATH` verifies a provider artifact
  and commits it through the transactional document boundary
  boundary.
- `conversation-turn TURN_ID EXPRESSION [--state PATH]` appends a verified
  semantic turn and optionally reloads/saves the conversation state atomically.
- `learning-propose`, `learning-approve`, `learning-reject`, and
  `learning-promote` operate on explicit JSON overlay artifacts. Promotion is
  impossible until approval.

Invalid input, integrity failures, store failures, and internal failures use
the existing typed exit-code mapping. A command never repairs a corrupted
document, conversation, or learning artifact during read.
