# Document MVP V1

## Purpose

Document MVP V1 freezes the first document-translation contract for lexFlex. It defines the initial blocking direction, input form, preserved structure, linguistic scope, and quality gates without reworking the runtime translator.

## Definition of Document

Document is not a page.

Document is not `Vec<String>`; it will become a structured unit in Chapter 03.

During Chapter 01 and 02 the current multi-sentence `Utterance` pipeline is only a baseline adapter.

## First Direction

The blocking direction is `pl` -> `en`.

`en` -> `pl` remains architecturally allowed and measured, but it is non-blocking for Document MVP V1.

## Accepted Input

Document MVP V1 accepts UTF-8 plain text with LF or CRLF line endings. Paragraphs are separated by at least one blank line.

Document MVP V1 does not directly accept PDF, DOCX, HTML DOM, OCR, images, tables, source code, mathematical notation, tracked changes, page layout, footnotes, or repeated headers and footers.

## Preserved Structure

The system must preserve paragraph order, paragraph count when no explicit transformation is needed, blank lines between paragraphs, sentence order, final sentence punctuation, names, numbers, negation, basic temporal reference, participant relations, and terminology consistency.

Exact sentence count, paragraph length, information order, and quotation marks are tracked but not blocking in V1.

## Supported Content Classes

Required content classes are neutral everyday narrative, factual description, simple informational text, simple instructional text without code, and descriptions of objects, people, and events.

Popular science, simple technical prose with glossary support, and simple business prose are tracked but non-blocking.

## Length Tiers

Document has no A4-based unit. Length is tracked through tiers: Micro, Short, Standard, and Stress.

Micro and Short are blocking tiers. Standard is partially covered and non-blocking. Stress is informational only.

## Required Linguistic Capabilities

Required capabilities include multi-sentence input, multi-paragraph input, repeated proper names, pronoun chains, Polish zero subject resolution, repeated entities realized as noun or pronoun, English definiteness, negation, numbers, quantifiers, basic tense continuity, temporal sequencing, Goal/Source/Location relations, coordination, cause/result relations, terminology consistency, preservation of critical entities and events, deterministic output, and explicit unresolved reporting in diagnostics.

## Tracked Capabilities

Tracked capabilities include competing antecedents, definite descriptions, basic multi-word expressions, collocations, complement clauses, temporal clauses, relative clauses, paragraph topic shifts, quotation and attribution, enumerations, passive voice, degree marking, formal or neutral style, and longer prepositional phrases.

## Deferred Capabilities

Deferred capabilities include full VP ellipsis, sluicing, general gapping, complete binding theory for embedded clauses, irony, metaphor, extended reported speech, literary paraphrase, author rhythm preservation, world knowledge completion, dialogue management, response planning, user memory, external tools, mathematics, and programming languages.

## Semantic Invariants

No critical semantic invariant may regress silently. No new critical entity or event may be invented, and negation, numbers, participant roles, and proper names must be preserved in blocking cases.

## Failure Policy

Baseline failures are expected and must be preserved honestly. A capability is not supported only because an enum or method exists. It must have corpus cases and measurable passing expectations.

## Determinism

The same input, profile, and data must produce the same result. Reports, manifests, snapshots, and serialized maps must be deterministic.

## Performance Reporting

Document MVP V1 uses explicit baseline metrics and release gates. Exact reference equality is not the primary quality metric.

## Release Gate

PL -> EN is blocking. EN -> PL is non-blocking. Chapter 01 freezes the contract; Chapter 02 freezes the baseline; neither chapter is allowed to improve translation quality by rewriting runtime semantics.

## Non-goals

Document MVP V1 does not add DocumentGraph, parser redesign, generator redesign, new frames, new runtime LLM paths, or broad lexicon expansion.

## Versioning

The contract is versioned as `document_v1`. The machine-readable profile is authoritative for automation; Markdown explains the profile.

Chapter 02.1 freezes `benchmarks/document_v1/baseline/document_v1_baseline.json` as the canonical Document V1 baseline.

Chapter 03.2 represents every source byte through an ordered partition of paragraph and preserved-whitespace blocks. Segmentation preserves LF, CRLF, CR, Unicode, leading whitespace, and trailing whitespace exactly; sentence records are introduced in Chapter 03.3.

Chapter 03.3 adds configurable, lossless sentence ranges without invoking the parser or translator. Sentence raw spans partition each paragraph exactly, content spans remain valid UTF-8 byte ranges, and all 30 Document V1 corpus cases produce 82 deterministic sentence records while preserving the original source byte for byte.
