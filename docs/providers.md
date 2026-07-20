# Provider Boundary

Provider output is untrusted data. `lexflex-provider-llm` exposes a bounded
`LlmProvider` trait and a deterministic `FixtureProvider` for offline tests.
Requests require a model and prompt-version identity, and prompt/response
budgets are checked before an artifact is returned.
Each artifact also carries a SHA-256 content hash for replay and audit.
Consumers must call `ProviderArtifact::verify` before converting an artifact to
a document; conversion repeats the verification boundary.
`ProviderCache` adds bounded deterministic replay keyed by the canonical request
hash and re-verifies cached artifacts before returning them.

Providers do not create verified semantic objects and do not mutate knowledge.
An eventual production adapter must pass its artifacts through document ingest,
parsing, semantic verification, evidence creation, and a transactional commit.
