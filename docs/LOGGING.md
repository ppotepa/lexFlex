# Tracing and logging

The engine exposes deterministic per-request traces. A trace is a JSONL sequence of `TraceEvent` records:

```json
{"request_id":"request:...","stage":"request","payload":{}}
```

## Trace stages

Depending on the request, traces can contain:

- `request`;
- `source.resolved`;
- `language.detected`;
- `interlingua.parsed`;
- `pipeline.bundle_ready`;
- `session.snapshot`;
- `query.interlingua`;
- `answer.selected`;
- `answer.unknown`.

Trace payloads are intended for diagnosis and inspection. Semantic artifact hashes do not include runtime duration, terminal paths or log formatting.

## Storage

Traces are persisted below:

```text
data/sessions/<session-id>/traces/<turn-id>.jsonl
```

Read one with:

```bash
cargo run -- trace --session <session-id> <turn-id>
```

The TUI supports `--trace off`, `--trace brief` and `--trace full` as display modes. Trace persistence remains an engine concern; the TUI only renders it.

Regular process logs use `tracing` and are disabled by default for the chat UI. Set `LEXFLEX_CHAT_LOG=1` when chat process logs are needed.
