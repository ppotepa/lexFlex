# Data management

lexFlex uses repository-local, content-addressed data for language resources, source snapshots and session artifacts.

## Repository data

```text
data/
  lexicons/<lang>/       language lexicons
  concepts/              semantic concept data
  sources/wikipedia/     local source snapshots
  sessions/<session-id>/ immutable session artifacts and traces
```

Language data is read by the parser and generator. The runtime does not silently modify committed lexicons or concepts during a normal request.

## Source snapshots

A snapshot stores source title, language, URI/revision metadata, complete text and a SHA-256 content hash. The source provider validates the hash before returning it to the engine.

```bash
cargo run -- source inspect Paris --lang en --format pretty-json
```

Live fetching is explicit:

```bash
cargo run -- source fetch Paris --lang en --live --format summary
```

## Session artifacts

Ingestion persists:

```text
data/sessions/<session-id>/
  snapshots/current.json
  snapshots/<snapshot-id>.json
  sources/<source-id>.json
  bundles/<bundle-id>.json
  traces/<turn-id>.jsonl
```

Writes use temporary files and rename. Loading validates the snapshot, source artifacts, bundle lineage and checksums. Corrupt or mismatched artifacts are rejected.

## Reproducibility

Use a fixed local snapshot, `--offline`, an explicit session ID and JSON output when producing reproducible results. Runtime timestamps and durations are not part of semantic hashes.

```bash
cargo run -- ingest Paris --lang en --session reproducible --format json
cargo run -- answer "What is the capital of France?" \
  --lang en --session reproducible --format json
```
