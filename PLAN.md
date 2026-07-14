# Plan

## Chapter 01 Document MVP Contract

- [x] Add the document contract docs
- [x] Add the machine-readable `DocumentProfile`
- [x] Add profile validation tests
- [x] Verify workspace checks after implementation

## Chapter 02 Corpus and Baseline

- [x] Add the benchmark workspace crate
- [x] Add the document corpus and manifest
- [x] Freeze the first baseline

## Chapter 02.1 Benchmark Hardening

- [x] Add metrics coverage tests
- [x] Add compare coverage tests
- [x] Harden glossary consistency
- [x] Add provenance hashes
- [x] Freeze the canonical Document V1 baseline
- [x] Self-compare the canonical Document V1 baseline

## Chapter 03.1 Foundation Hardening

- [x] Harden document IDs and UTF-8 line index
- [x] Split document model, validation, and reconstruction
- [x] Add source SHA-256 to `Document`
- [x] Make benchmark freeze hermetic
- [x] Freeze and compare the canonical Document V1 baseline

## Chapter 03.2 Lossless Paragraph Segmentation

- [x] Validate exact top-level block partitions
- [x] Make `DocumentBuilder` append contiguous regions safely
- [x] Preserve LF, CRLF, CR, Unicode, and edge whitespace exactly
- [x] Gate deterministic reconstruction on all 30 corpus documents
- [x] Begin lossless sentence range segmentation

## Chapter 03.3 Lossless Sentence Range Segmentation

- [x] Split structural validation into focused modules
- [x] Add configurable PL/EN sentence segmentation rules
- [x] Preserve exact sentence raw and content byte spans
- [x] Keep paragraph-only segmentation as a separate stage
- [x] Gate 82 sentence records across all 30 corpus documents
- [ ] Begin per-sentence document compilation

References:
- [docs/document/README.md](./docs/document/README.md)
- [data/profiles/document_mvp_v1.ron](./data/profiles/document_mvp_v1.ron)
- [benchmarks/document_v1/README.md](./benchmarks/document_v1/README.md)
