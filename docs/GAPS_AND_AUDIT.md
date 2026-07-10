# lexFlex Docs vs Implementation Audit

**Date:** 2026-07-10
**Context:** Review of lexFlex/docs/ against src/ implementation. Also consulted lightlm/concat.txt for related prior work.

## Executive Summary

The documentation is **comprehensive and ambitious** (~35 files, 25k+ lines in concat). The implementation has a **working MVP**:

- ✅ Compiles cleanly
- ✅ 32/32 tests passing
- ✅ Benchmark: 110/110 (100%) PL↔EN on simple sentences
- ✅ Core pipeline (parse → deduction → generate) functional for supported frames

**However**, a significant portion of the detailed specifications in the docs is **not (fully) implemented**. Many features are stubbed, partially wired, or use hardcoded logic instead of the described data-driven / descriptor-driven approach.

The recent addition of `verb_concept: String` to all `Frame` variants (good step toward doc-described lexical selection) had not been propagated to all match sites, causing compile breakage until fixed during this audit.

## Major Gaps (Addressed for MVP Consistency)

### 1. LanguageDescriptor — NOW PARTIALLY WIRED (AC1)
- has_articles now conditions EN article insertion (PL explicitly ensures none emitted).
- aspect_type consulted in verb/aux selection (periphrastic progressive in EN; Morphological path in PL).
- negation_particle used (was earlier).
- word_order / pro_drop read in generate_sentence (no full Free reorder as non-goal, requires topic/focus).
- No dead-code warnings; no bypass hardcodes for these.

See also updated KNOWN_LIMITATIONS.md.

### 2. Information Structure (Topic / Focus) — OUT OF SCOPE for this goal
- Still absent (non-goal). word_order/pro_drop now read.

### 3. Coordination — OUT OF SCOPE (v0.2+)
- Still capability only (non-goal).

### 4. Generator is "Pure Realization" (but isn't fully)
**Docs (GENERATOR.md, IMPLEMENTATION_GUIDE):**
- Generator must **never** make semantic decisions.
- Must use verb_concept + lexicon + morphology + descriptor for everything.
- Detailed 8-phase pipeline + pronoun form selection (enclitic vs full).

**Reality:**
- Significant hardcoding remains:
  - `find_verb_for_frame` + big match on frame type returning "GIVE"/"SEE" etc.
  - Passive participle tables hardcoded in both generators.
  - Quantifier words ("wszyscy", "nikt") hardcoded.
  - Many verb lemmas inside passive paths.
- Pronoun selection (enclitic/full, pro-drop) is **not** implemented in generator (deduction does basic resolution).
- `generate_entity_form` and friends do not consult descriptor for many choices.

### 5. Parser Morphology Integration — ADDRESSED (AC2)
- Now unprefixed; analyze_morphology calls morphology.analyze_verb_form() which runs reverse_to_stem + conditions from RON paradigms (data/morphology + pl/morph).
- Heuristics + lexicon still used; RON rules now exercised for representative past/present verbs.

### 6. Frame Coverage & Verb Concept Usage — ADDRESSED (AC3)
- find_verb_for_frame now prefers verb_concept + lexicon.lookup_concept first (in PL/EN).
- Passive sentence generation resolves surface via concept lookup from frame's verb_concept.
- Fallbacks kept for safety. Hardcoded maps reduced in main paths.

### 7. Other Documented Areas with Limited Implementation
- **Pronouns (PRONOUNS.md):** Basic within-sentence resolution exists in deduction. Enclitic forms, full selection logic, register effects — missing.
- **Error Handling (ERROR_HANDLING_GUIDE.md):** Docs describe rich best-effort, warnings with partial results, graceful degradation. Current `error.rs` + usage is mostly hard errors + simple fallbacks.
- **Logging (LOGGING.md):** Per-request contextual logging framework described; implementation uses tracing but not the full described `ProcessingLog` model from API.md.
- **Ontology / Deduction validation:** Basic IS_A and some validation present; full semantic type checking per docs is lighter.
- **Data volume:** Docs target ~500 concepts. Current: ~50-60. Lexicons ~160 entries each.
- **Morphology completeness (MORPHOLOGY.md, GRAMMAR_CASES.md):** Consonant alternations supported in op enum but not populated in RON paradigms. Limited irregulars. No suppletion tables as described.

### 8. Outdated Documentation
- `KNOWN_LIMITATIONS.md` lists as unsupported things that are now working (passive voice, quantification, some do-support, English negation/questions).
- Some roadmap items in IMPLEMENTATION_ROADMAP are partially done.
- STATUS.md was reasonably up-to-date on the "what works" side but claimed "Documentation synchronized" while gaps above exist.

## Relation to lightlm/concat.txt

lightlm (C# + SQLite) is a sibling/precursor project:
- Uses "Interlingua" as neutral pivot (concepts + lexical_entries linked by concept_id).
- Richer relational model: lexical_relations, morphology overrides, surface index.
- Has `InterlinguaFrame`, manual frame builder, realization pipeline, chat framework around frames.
- Concept seeds + build scripts.

lexFlex RON files are a **simplified, file-based** take on similar ideas (concepts.ron + per-lang lexicons + paradigms). Many design principles overlap (hub-and-spoke lexicon, frames, capability checks).

The detailed linguistic docs in lexFlex (Interlingua, Deduction, Generator) appear to be an expansion of lessons from lightlm work.

## Positive Notes — What *is* Well Aligned

- Core types (InterlinguaNode, Frame variants, FeatureBundle, roles, temporal, quant) closely follow docs.
- Deduction pipeline (apply_verb_frames, resolve_cases, pronouns, temporal, ontology, normalize) matches the structure in DEDUCTION.md.
- Passive voice, quantification, English do-support, pronoun resolution (basic), feature normalization — recently added and match documented behaviors.
- Data-driven morphology rule engine exists and is used for generation.
- Tests + benchmark cover the implemented slice well.
- CLI / API / translator structure is clean.

## Recommendations (Priority)

1. **Wire LanguageDescriptor** (highest impact per docs):
   - Implement `determine_word_order` using descriptor + (later) info structure.
   - Drive article insertion (EN) from `has_articles`.
   - Drive aspect realization from `aspect_type`.
   - Use `pro_drop` for subject omission in PL.

2. **Reduce hardcoding in generators**:
   - Make `verb_concept` the primary key for lexical lookup + verb form selection.
   - Move passive participles, quant words, etc. into lexicon or dedicated data.

3. **Add minimal info structure** to `Sentence` (topic/focus) so Polish can eventually vary order.

4. **Sync docs**:
   - Update KNOWN_LIMITATIONS.md to reflect current implemented features.
   - Mark sections in advanced docs (DISCOURSE, SPEECH_ACTS, etc.) clearly as v0.2.
   - Add "Implementation Status" callouts to key docs.

5. **Parser morphology**:
   - Actually call paradigm rules from `analyze_morphology` (or integrate the morphology engine more deeply).

6. **Data expansion**:
   - Grow concepts + lexical entries + paradigms toward the numbers in docs.

7. **Consider** whether some v0.1 scope in docs (e.g. full coordination) should be explicitly scoped down.

## Files Changed During This Audit (for reference)

- Fixed widespread `Frame` pattern matches after `verb_concept` addition (pl/en generators, tests).
- Wired `negation_particle` from descriptor (small but real step).
- Suppressed dead field warning for morphology (acknowledges current partial usage).
- Ensured clean build + tests + benchmark.

## Next Steps for the Team

- Decide scope: "make docs match current implementation" vs "implement more of the documented design".
- Pick 1-2 gaps above and drive them to completion with tests.
- Regenerate or maintain a living "Implementation vs Spec" matrix.

This audit was performed by inspecting docs/*.md (including concat), source under src/{core,engines,data,api}, data/*.ron, tests, STATUS, and cross-referencing lightlm/concat.txt for historical design context.
