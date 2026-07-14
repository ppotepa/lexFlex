# End-to-end example

This example uses the local English Wikipedia snapshot for Paris. It does not require network access.

## 1. Verify the snapshot

```bash
cargo run -- source inspect Paris --lang en --format summary
```

If the snapshot is missing, obtain it explicitly with:

```bash
cargo run -- source fetch Paris --lang en --live --format summary
```

## 2. Ingest the document

```bash
cargo run -- ingest Paris \
  --lang en \
  --session paris-demo \
  --format summary
```

The command resolves the source, runs the document pipeline and saves the resulting session snapshot.

## 3. Ask a question

```bash
cargo run -- answer "What is the capital of France?" \
  --lang en \
  --session paris-demo \
  --format pretty-json
```

Inspect `Conversation.meta.status`, `Conversation.text`, `Conversation.answer.rows` and `Conversation.answer.evidence` in the response. The answer must have evidence to be considered factual.

## 4. Inspect the session

```bash
cargo run -- inspect \
  --session paris-demo \
  --target session \
  --format pretty-json
```

The response exposes the session ID, current snapshot ID, source count and bundle count.

## 5. Read the trace

Natural-language answers persist a deterministic JSONL trace. The response contains a `trace_ref`; pass its request ID to:

```bash
cargo run -- trace --session paris-demo <request-id>
```

The trace records source resolution, language detection, Interlingua parsing, query construction, execution and answer selection.

## 6. Repeat in the TUI

```bash
cargo run -- chat --offline
```

Then enter:

```text
/ingest Paris
What is the capital of France?
```

The TUI submits the same engine requests as the CLI. It does not implement a separate QA path.
