# Document Baseline V1 Report

Baseline source:
- `results/document-v1/baseline-candidate/run.json`
- `results/document-v1/baseline-candidate/summary.txt`

Observed run summary:
- cases: 30
- successful outputs: 23
- hard failures: 7

This report is a baseline audit, not a quality claim. The runner currently exposes raw outputs and hard parse failures, so several categories are inferred from surface symptoms and are intended as the next implementation targets.

## 1. Segmentation

- Count: 0 confirmed
- Case IDs: none
- Representative evidence: no direct sentence-boundary failure observed in this run
- Likely symbol: `context::split_sentence_boundaries`
- Target chapter: Chapter 03
- Severity: Major

## 2. Unknown Token / Lexical Coverage

- Count: 18 observed outputs with untranslated or malformed lexical surfaces
- Case IDs: `doc-pl-en-004`, `doc-pl-en-005`, `doc-pl-en-007`, `doc-pl-en-008`, `doc-pl-en-010`, `doc-pl-en-012`, `doc-pl-en-013`, `doc-pl-en-016`, `doc-pl-en-019`, `doc-pl-en-020`, `doc-pl-en-021`, `doc-pl-en-023`, `doc-pl-en-024`, `doc-pl-en-025`, `doc-pl-en-027`, `doc-pl-en-028`, `doc-pl-en-029`, `doc-pl-en-030`
- Representative evidence: `This is a ważny zespołu.`, `driveed`, `becomeed`
- Likely symbol: `data::lexicon` and `core::unknown_concept`
- Target chapter: Chapter 04
- Severity: Major

## 3. Morphology

- Count: 6
- Case IDs: `doc-pl-en-004`, `doc-pl-en-005`, `doc-pl-en-007`, `doc-pl-en-016`, `doc-pl-en-020`, `doc-pl-en-024`
- Representative evidence: `prepareed`, `driveed`, `becomeed`
- Likely symbol: `engines::pl::morphology`, `engines::en::morphology`
- Target chapter: Chapter 04
- Severity: Major

## 4. Syntax

- Count: 7 hard failures plus several malformed but non-empty outputs
- Case IDs: `doc-pl-en-002`, `doc-pl-en-006`, `doc-pl-en-011`, `doc-pl-en-015`, `doc-pl-en-022`, `doc-pl-en-026`, `doc-pl-en-029`
- Representative evidence: `Parse error: No verb found in sentence`
- Likely symbol: `engines::{pl,en}::parser`
- Target chapter: Chapter 04
- Severity: Fatal

## 5. Frame Selection

- Count: 4
- Case IDs: `doc-pl-en-009`, `doc-pl-en-010`, `doc-pl-en-027`, `doc-pl-en-028`
- Representative evidence: wrong or underspecified frame realization such as `This is an uszkodzony.`
- Likely symbol: `core::graph::frame_verb_concept`
- Target chapter: Chapter 04
- Severity: Major

## 6. Semantic Role Assignment

- Count: 6
- Case IDs: `doc-pl-en-001`, `doc-pl-en-008`, `doc-pl-en-012`, `doc-pl-en-017`, `doc-pl-en-018`, `doc-pl-en-029`
- Representative evidence: goal/source/recipient roles collapse into generic surface output
- Likely symbol: `core::graph::frame_role_entities`
- Target chapter: Chapter 04
- Severity: Major

## 7. Coreference

- Count: 5
- Case IDs: `doc-pl-en-001`, `doc-pl-en-005`, `doc-pl-en-018`, `doc-pl-en-021`, `doc-pl-en-026`
- Representative evidence: pronoun chains survive in some cases, but not consistently enough for document gates
- Likely symbol: `context::resolve_discourse_context`
- Target chapter: Chapter 04
- Severity: Major

## 8. Zero Anaphora

- Count: 3
- Case IDs: `doc-pl-en-001`, `doc-pl-en-018`, `doc-pl-en-026`
- Representative evidence: subject continuation is partially preserved, but not yet stable
- Likely symbol: `context::track_discourse`
- Target chapter: Chapter 04
- Severity: Major

## 9. Temporal Continuity

- Count: 4
- Case IDs: `doc-pl-en-001`, `doc-pl-en-007`, `doc-pl-en-018`, `doc-pl-en-022`
- Representative evidence: deictic sequencing is present in a few outputs, but not aligned to the corpus contract
- Likely symbol: `core::temporal`
- Target chapter: Chapter 04
- Severity: Major

## 10. Discourse Relation

- Count: 5
- Case IDs: `doc-pl-en-004`, `doc-pl-en-005`, `doc-pl-en-020`, `doc-pl-en-024`, `doc-pl-en-030`
- Representative evidence: paragraph-level topic continuation is not preserved deterministically
- Likely symbol: `context::track_discourse`
- Target chapter: Chapter 05
- Severity: Major

## 11. Definiteness / Article

- Count: 4
- Case IDs: `doc-pl-en-001`, `doc-pl-en-003`, `doc-pl-en-017`, `doc-pl-en-023`
- Representative evidence: `a store`, `a milk`, `a book`, `a car`
- Likely symbol: `engines::en::generator`
- Target chapter: Chapter 04
- Severity: Minor

## 12. Agreement

- Count: 4
- Case IDs: `doc-pl-en-004`, `doc-pl-en-005`, `doc-pl-en-017`, `doc-pl-en-021`
- Representative evidence: surface fragments show agreement drift after translation
- Likely symbol: `engines::{pl,en}::generator`
- Target chapter: Chapter 04
- Severity: Minor

## 13. Terminology

- Count: 6
- Case IDs: `doc-pl-en-004`, `doc-pl-en-009`, `doc-pl-en-010`, `doc-pl-en-020`, `doc-pl-en-028`, `doc-pl-en-029`
- Representative evidence: glossary terms do not stay stable across sentence boundaries
- Likely symbol: `generation::pipeline`
- Target chapter: Chapter 05
- Severity: Major

## 14. Formatting

- Count: 5
- Case IDs: `doc-pl-en-004`, `doc-pl-en-005`, `doc-pl-en-020`, `doc-pl-en-024`, `doc-pl-en-030`
- Representative evidence: multi-paragraph structure is flattened or partially collapsed
- Likely symbol: `generation::realizer`
- Target chapter: Chapter 05
- Severity: Minor

## 15. Omission

- Count: 4
- Case IDs: `doc-pl-en-005`, `doc-pl-en-020`, `doc-pl-en-021`, `doc-pl-en-024`
- Representative evidence: some clause-level content is present only as generic placeholders
- Likely symbol: `generation::pipeline`
- Target chapter: Chapter 05
- Severity: Major

## 16. Hallucination

- Count: 4
- Case IDs: `doc-pl-en-004`, `doc-pl-en-013`, `doc-pl-en-024`, `doc-pl-en-030`
- Representative evidence: placeholder fragments such as `This is a ...` appear where document-specific content is expected
- Likely symbol: `generation::pipeline`
- Target chapter: Chapter 05
- Severity: Major

## 17. Runtime / Error Handling

- Count: 7
- Case IDs: `doc-pl-en-002`, `doc-pl-en-006`, `doc-pl-en-011`, `doc-pl-en-015`, `doc-pl-en-022`, `doc-pl-en-026`, `doc-pl-en-029`
- Representative evidence: `Parse error: No verb found in sentence`
- Likely symbol: `api::translate` and `translator::translate`
- Target chapter: Chapter 02 prerequisite / Chapter 04 stabilization
- Severity: Fatal

## Notes

- The current baseline is intentionally weak.
- `compare` against the frozen run is clean after normalizing runtime measurements.
- The next work item is not translation tuning here; it is better corpus semantics, better runner instrumentation, and better failure classification.
