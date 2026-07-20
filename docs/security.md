# Security and Resource Boundaries

All persisted semantic, document, conversation, learning, provider, and
session payloads are verified at their deserialize or load boundary. Session
IDs are path-safe ASCII identifiers bounded to 128 bytes. Document ingest,
provider requests, cache entries, parser work, execution, and traces have
explicit resource budgets.

Writes use temporary files followed by atomic rename. Failed writes and failed
mutations do not replace the in-memory or persisted candidate. Provider output
is untrusted and must pass content-hash verification before document ingest.

The repository contains no tracked runtime sessions or source payloads. Fuzz
and property-test expansion remains a release follow-up for parser and serde
boundaries.
