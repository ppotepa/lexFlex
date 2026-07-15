# Tracing and logging

The engine exposes deterministic per-request traces. A trace is a JSONL sequence of `TraceEvent` records:

```json
{"run_id":"run:00000001","request_id":"request:...","sequence":0,"stage":"request","payload":{}}
```

## Trace stages

Depending on the request, traces can contain:

- `request`;
- `runtime.data_root`;
- `source.discovery` and `source.selected`;
- `source.resolve.started`;
- `source.resolved`;
- `language.detected`;
- `language.parse_failed`;
- `language.parse_fallback`;
- `interlingua.parsed`;
- `pipeline.started`, `pipeline.bundle_ready`, `pipeline.compilation`, `pipeline.graph`, `pipeline.resolution`, `pipeline.temporal_discourse` and `pipeline.knowledge`;
- `session.snapshot`;
- `query.interlingua`;
- `query.structured`;
- `query.plan`;
- `query.execution`;
- `answer.source_sentence_fallback`;
- `answer.selected`;
- `answer.unknown`.

Trace payloads are intended for diagnosis and inspection. Semantic artifact hashes do not include runtime duration, terminal paths or log formatting.

## Storage

Traces are persisted below:

```text
data/sessions/<session-id>/traces/<run-id>.jsonl
```

Read one with:

```bash
cargo run -- trace --session <session-id> <turn-id>
```

`request_id` is deterministic for the semantic request and snapshot. `run_id` is a monotonic session-local execution identifier, so repeated identical questions never overwrite each other. The TUI receives the same events that are persisted: full-stack mode renders them live, while `/trace` reads the completed JSONL run.

`/trace` reports corrupt JSONL lines explicitly instead of silently skipping them.

Regular process logs use `tracing` and are disabled by default for the chat UI. Set `LEXFLEX_CHAT_LOG=1` when chat process logs are needed.
