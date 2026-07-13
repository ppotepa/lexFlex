# COMPLETE IMPLEMENTATION GUIDE — lexFlex v0.1

**Version:** 1.0 (Final Practical Edition)  
**Focus:** How to actually build the system  
**Goal:** Turn the existing architecture into working, maintainable code with minimal wasted effort.

This document consolidates and significantly expands all previous specifications. It focuses on **practical implementation** — the "how" rather than just the "what". It addresses the main weaknesses identified in earlier reviews (especially the Generator and Deduction Engine).

---

## 1. Philosophy and Key Implementation Principles

### 1.1 Core Principles

1. **Interlingua is the single source of truth**  
   The parser can (and should) produce incomplete structures. The Deduction Engine completes them. The Generator should never guess meaning.

2. **Deduction is more important than the Parser**  
   It is better to have a relatively simple parser + strong deduction than an extremely complex parser.

3. **The Generator is the hardest component**  
   Especially for Polish. In v0.1, accept somewhat stiff but correct sentences. Naturalness can be improved later with an optional post-processing step.

4. **Data-driven over hardcoded logic**  
   Morphology rules, verb frames, and lexical information should live in RON files as much as possible.

5. **Iterative development**  
   First make a working pipeline on simple sentences. Then expand coverage.

6. **LLM usage (via lexflex-learner)**  
   Local LLM (e.g. via LM Studio/Ollama) is used **first** in the learner for semantic decisions on unknowns (lemma recovery, is_a hierarchies, parent suggestions for concept trees). It is integrated into the main pipeline (unknown_concept resolution) and appends proposals on-the-fly to the live RON DB. Set `LEXFLEX_LLM_BASE_URL` + `MODEL`. Use for building/extending the generic concept DB + lang lexicons. Post-processing for fluency is secondary.

### 1.2 Final v0.1 Scope

**In scope:**
- Polish ↔ English translation of simple sentences
- Full pipeline: Parse → Deduction → Generate
- Core frames: Transfer, Motion, Perception, Consumption, Destruction
- Algorithmic Polish morphology (cases, aspect, agreement)
- Deduction for case/role resolution, basic pronoun binding (within sentence), temporal anchoring
- Basic quantification and temporal expressions

**Explicitly out of scope for v0.1:**
- Multi-turn dialogue
- Long-term memory
- Speech act / intent recognition
- Cross-sentence coreference
- Complex subordinate clauses

---

## 2. Recommended Project Structure

```bash
lexflex/
├── src/
│   ├── core/
│   │   ├── interlingua.rs          # All Interlingua types
│   │   ├── deduction.rs            # Core deduction logic
│   │   ├── ontology.rs
│   │   ├── temporal.rs
│   │   ├── traits.rs
│   │   └── mod.rs
│   ├── engines/
│   │   ├── pl/
│   │   │   ├── parser.rs
│   │   │   ├── generator.rs        # Most complex file in v0.1
│   │   │   ├── morphology.rs
│   │   │   └── data.rs
│   │   └── en/
│   │       ├── parser.rs
│   │       ├── generator.rs
│   │       └── morphology.rs
│   ├── translator.rs
│   ├── api.rs
│   ├── data/loader.rs
│   └── error.rs
├── data/
│   ├── concepts/concepts.ron
│   ├── lexicons/pl/lexicon.ron
│   ├── lexicons/en/lexicon.ron
│   ├── morphology/pl/*.ron
│   ├── descriptors/pl.ron
│   └── ontology/ontology.ron
├── tests/
│   └── golden/                     # Use insta crate
└── docs/
    └── COMPLETE_IMPLEMENTATION_GUIDE_v0.1.md
```

**Recommendation:** Keep language-specific code strictly inside `engines/<lang>/`. Core semantic logic must stay in `core/`.

---

## 3. Implementation Roadmap (Phased)

| Phase | Focus                              | Estimated Time | Milestone |
|-------|------------------------------------|----------------|---------|
| 1     | Core types + Ontology              | 2–3 weeks      | Can create and validate basic InterlinguaNodes |
| 2     | Polish Morphology                  | 3–4 weeks      | Working algorithmic declension for 4+ paradigms |
| 3     | Parser + Deduction Engine          | 4–5 weeks      | Correct Frame resolution on simple sentences |
| 4     | Polish + English Generator         | 5–7 weeks      | End-to-end translation works on basic Transfer sentences |
| 5     | Integration + API                  | 2–3 weeks      | Working `LexFlexAPI` with builder |
| 6     | Testing + Hardening                | 2 weeks        | Golden tests + basic coverage |

**Critical advice:** Do **not** leave the Generator until the very end. Start sketching it during Phase 3.

---

## 4. Core Layer Implementation

### 4.1 Interlingua Types

Start by defining clean, strongly typed structures in `src/core/interlingua.rs`.

Key recommendations:
- Use `enum Frame { Transfer {..}, Motion {..}, ... }` instead of dynamic structures.
- `Entity` should contain `concept: ConceptId` + `features: FeatureBundle`.
- Keep `FeatureBundle` as a struct with many `Option<T>` fields (easy to extend).

### 4.2 Ontology

Implement at least:
- `is_a` hierarchy
- `validate_semantic_types(&frame)` 
- `inherit_features(&mut entity)`

This is used heavily inside the Deduction Engine.

---

## 5. Polish Morphology – Step by Step

This is foundational for the Generator.

**Implementation order:**
1. Define `MorphOperation` enum (`ReplaceSuffix`, `AddSuffix`, `RemoveSuffix`).
2. Define `MorphRule` and `MorphParadigm`.
3. Implement a `apply_rules()` function.
4. Start with 4 noun paradigms + 2 verb paradigms (as shown in `DATA_SAMPLES.md`).
5. Add unit tests for every paradigm.

**Important:** Morphology must be purely rule-based. Avoid hardcoding word forms.

---

## 6. Polish Parser Implementation Guide

Recommended architecture:

```rust
pub struct PolishParser {
    lexicon: Lexicon,
    morphology: PolishMorphology,
}

impl PolishParser {
    pub fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
        let tokens = self.tokenize(input);
        let morph_analyzed = self.analyze_morphology(&tokens);
        let partial = self.build_partial_structure(morph_analyzed)?;
        
        // Always run deduction
        let context = DeductionContext::new(...);
        let final_utterance = deduction::deduce(partial, &LanguageId::PL, &context)?;
        
        Ok(final_utterance)
    }
}
```

**Start simple:**
- Assume basic SVO order on first implementation.
- Use verb subcategorization frames early to resolve cases.

---

## 7. Deduction Engine – Detailed Implementation Guide

This is one of the most important components in v0.1.

### Recommended Structure

Create `src/core/deduction.rs` with the following main function:

```rust
pub fn deduce(
    mut utterance: Utterance,
    language: &LanguageId,
    context: &DeductionContext,
) -> Result<Utterance, DeductionError> {
    for sentence in &mut utterance.sentences {
        apply_verb_frames(sentence, context)?;
        resolve_cases_and_roles(sentence)?;
        resolve_pronouns_within_sentence(sentence)?;
        anchor_temporals(sentence, context.current_time)?;
        validate_and_inherit_from_ontology(sentence, context.ontology)?;
        normalize_features(sentence)?;
    }
    Ok(utterance)
}
```

### Step-by-Step Implementation Order

1. **apply_verb_frames**  
   Look up the verb in `concepts.ron`, get required roles, and try to assign NPs/PPs to those roles.

2. **resolve_cases_and_roles** (Most important for Polish)  
   Priority:
   - Explicit case wins
   - Verb frame + position
   - Ontology constraints (e.g. only Animate can be Agent)
   - Default heuristics

3. **resolve_pronouns_within_sentence**  
   Handle reflexives (`się`, `sobie`) and basic 3rd person pronouns inside one sentence.

4. **anchor_temporals**  
   Convert deictic expressions (`wczoraj`, `jutro`) into relative references.

5. **validate_and_inherit_from_ontology**  
   Run semantic type checking and inherit features.

**Recommendation:** Implement this engine iteratively. Start with verb frame matching + case resolution. Add pronoun and temporal logic later.

---

## 8. Polish Generator – The Most Critical Component

This is currently the weakest area in the documentation and the hardest part to implement.

### Recommended Strategy

Do **not** try to build a perfect generator from day one.

**Phase approach inside the Generator:**

**Stage 1 (Minimum Viable):**
- Only support `Frame::Transfer`
- Hardcode basic word order (SVO)
- Use morphology module for inflection
- Ignore style, register, and pronoun choice for now

**Stage 2:**
- Add support for more frames
- Implement basic pronoun selection (enclitic vs full form)
- Add simple article logic for English

**Stage 3:**
- Use `LanguageDescriptor` to drive generation decisions
- Improve naturalness

### Key Generation Steps (for Transfer frame)

1. Determine verb lemma + aspect/tense → inflect it.
2. Place subject (agent) and inflect it.
3. Place indirect object (recipient) and inflect it.
4. Place direct object (theme) and inflect it.
5. Assemble the sentence.

**Important decision:**  
In v0.1, it is acceptable if the generator produces slightly stiff but grammatically correct Polish. You can later add an optional LLM post-processing step only for fluency improvement.

---

## 9. English Engine

The English side is significantly easier:
- No case inflection on nouns
- Fixed SVO order
- Main complexity = articles (`a` / `the`)

Start with a simpler generator than the Polish one.

---

## 10. Integration Layer

Implement `UniversalTranslator` after both engines are working.

Key responsibilities:
- Call `to_interlingua()` on source engine (which internally runs deduction)
- Perform capability check
- Call `from_interlingua()` on target engine

---

## 11. Error Handling Strategy

Define clear error types early:

- `ParseError`
- `DeductionError`
- `GenerateError`
- `TranslateError`

Use `thiserror`. Return detailed errors with suggestions when possible (especially for unknown words).

---

## 12. Data Files – Practical Advice

- Start small (50–80 concepts, 80–100 Polish words).
- Focus on quality of `concepts.ron` and verb frames.
- Write validation tests for RON files (`tests/data_validation_test.rs`).

---

## 13. Testing Recommendations

- Use **golden tests** (`insta` crate) from the beginning.
- Test morphology in isolation first.
- Create end-to-end golden tests for the most common sentence patterns.
- Add property-based tests later for robustness.

---

## 14. Common Pitfalls to Avoid

- Trying to make the parser too smart too early.
- Delaying work on the Generator.
- Putting too much logic inside the parser instead of deduction.
- Ignoring ontology validation during deduction.
- Over-engineering pronoun placement before having a working basic generator.

---

## 15. Full End-to-End Example (Recommended to Implement Early)

Target example:

**Input:** `"Tomek dał jabłko Izie"`

Expected flow:
1. Tokenization + morphological analysis
2. Partial structure creation
3. Deduction resolves `jabłko` as Theme + Accusative
4. Final Interlingua: `Frame::Transfer { agent: Tomek, recipient: Iza, theme: Apple }`
5. Generation produces correct output in target language

Implement this example as soon as Phase 3–4 is reached. It will serve as your main integration test.

---

## 16. Next Steps After Core v0.1

Once basic translation works reliably:
- Add more frames gradually
- Improve Generator naturalness (optional LLM post-processor)
- Begin light discourse tracking (v0.2 direction)
- Expand test coverage significantly

---

## Summary

This document provides a clear, phased path to implement lexFlex v0.1. The biggest challenges will be:

- Building a reliable **Deduction Engine**
- Creating a usable **Polish Generator**

Both are now described with recommended internal structure and implementation order.

Start with **Phase 1 and 2**, then move into Parser + Deduction. Begin sketching the Generator early rather than leaving it until the end.

This guide should allow you to begin serious implementation with significantly fewer uncertainties than before.

---

## 17. Key Documentation References

This guide provides the **how**, but for detailed specifications refer to these documents:

| Implementation Phase | Primary References |
|---------------------|-------------------|
| Phase 1: Core Types | [INTERLINGUA.md](./INTERLINGUA.md), [ONTOLOGY.md](./ONTOLOGY.md) |
| Phase 2: Polish Morphology | [MORPHOLOGY.md](./MORPHOLOGY.md), [DATA_SAMPLES.md](./DATA_SAMPLES.md) |
| Phase 3: Parser + Deduction | [DEDUCTION.md](./DEDUCTION.md), [ENGINE.md](./ENGINE.md), [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) |
| Phase 4: Generators | [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md), [PRONOUNS.md](./PRONOUNS.md) |
| Phase 5: Integration | [API.md](./API.md), [ERROR_HANDLING.md](./ERROR_HANDLING.md), [LOGGING.md](./LOGGING.md) |
| Phase 6: Testing | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [EXAMPLES.md](./EXAMPLES.md) |

**Architecture overview:** [ARCHITECTURE.md](./ARCHITECTURE.md)  
**Cross-language features:** [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md)  
**Terminology:** [GLOSSARY.md](./GLOSSARY.md)

**Feature v0.2+ (future):** [DISCOURSE.md](./DISCOURSE.md), [SPEECH_ACTS.md](./SPEECH_ACTS.md), [INTENTS.md](./INTENTS.md), [DIALOGUE.md](./DIALOGUE.md), [RESPONSE_PLANNING.md](./RESPONSE_PLANNING.md), [MEMORY.md](./MEMORY.md)

---

**End of Document**
