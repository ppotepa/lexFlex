# Error handling

The runtime uses typed errors and explicit response statuses. It does not silently replace failed semantic stages with guessed data.

## Engine errors

`EngineError` currently covers:

- `InvalidRequest` — malformed or unsupported request;
- `SourceUnavailable` — requested snapshot or live source is unavailable;
- `Source` — source read, cache or integrity failure;
- `Pipeline` — compilation, graph, resolution, temporal or knowledge failure;
- `Query` — invalid query or execution failure;
- `Persistence` — invalid, corrupted or mismatched session data.

The CLI reports errors to stderr and exits with a non-zero status. Library clients receive `EngineResponse::Error` with the error and response metadata.

## Answer statuses

`Unknown` is a valid result, not an exception. It is returned when the engine has no evidence-backed answer. Other answer-level states include `Exact`, `No`, `Supported`, `Conflicting`, `Unsupported` and `InvalidQuery`.

The engine does not infer `No` merely because a claim is absent. Positive and negative claims, incompatible values and conflicts remain represented in the answer.

## Diagnostics

Responses contain a diagnostics list and a trace reference. For an answer without evidence, diagnostics normally include `no_evidence` and `answer_unknown`. Diagnostics are informational metadata; the status remains authoritative.

## Source integrity

Source snapshots are verified against their content SHA-256 before ingestion. Bundle lineage and stage checksums are validated before publication. A mismatch is a typed source or pipeline error.

## Persistence integrity

Session loading validates the current snapshot, source artifacts, bundle artifacts, hashes and safe paths. Corruption is rejected; no partial session is exposed.
