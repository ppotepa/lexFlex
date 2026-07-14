# Wikipedia Paris corpus v1

This directory is the controlled source corpus for the engine-level Paris QA
tests. Source files are complete immutable Wikipedia extract snapshots; tests
must use `SnapshotOnly` and must never fetch Wikipedia during a benchmark run.

The EN and PL revisions contain the full plain-text article extract, source
metadata, content SHA-256 and deterministic annotations. Runtime fetch time is
intentionally absent from the semantic snapshot.

Required annotations per revision:

- source title, language, URI, revision and content hash;
- sentence/entity/relation spans;
- population, dates, locations, transport, landmarks and aliases;
- must-not-extract and unknown-answer cases.
