# Implementation Roadmap — lexFlex v0.1

This document provides a phased implementation plan with clear milestones, dependencies, and deliverables.

## Overview

**Total estimated time:** 18-24 weeks  
**Target:** Working Polish ↔ English translation of simple sentences

---

## Phase 1: Core Types + Ontology (2-3 weeks)

### Goals
- Define all core data structures
- Implement basic ontology with type validation
- Create test infrastructure

### Deliverables
- `src/core/interlingua.rs` - All InterlinguaNode types
- `src/core/ontology.rs` - Type hierarchy and validation
- `src/core/traits.rs` - IMeaningRepresentation, INaturalLanguage
- `src/error.rs` - Error types
- Unit tests for core types

### Detailed Tasks

#### Week 1: Core Data Structures

**Day 1-2: FeatureBundle and Entity**
```rust
pub struct FeatureBundle {
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub person: Option<Person>,
    pub animacy: Option<Animacy>,
    pub definiteness: Option<Definiteness>,
    pub countability: Option<Countability>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub case: Option<Case>,
}

pub struct Entity {
    pub concept: ConceptId,
    pub name: Option<String>,
    pub features: FeatureBundle,
}
```

**Day 3-4: Frame types**
```rust
pub enum Frame {
    Transfer { agent: Entity, recipient: Entity, theme: Entity },
    Motion { agent: Entity, goal: Option<Entity>, source: Option<Entity> },
    Perception { experiencer: Entity, stimulus: Entity },
    Cognition { cognizer: Entity, content: Entity },
    Destruction { agent: Entity, patient: Entity },
}
```

**Day 5: Sentence and Utterance**
```rust
pub struct Sentence {
    pub frames: Vec<Frame>,
    pub temporal: Option<TemporalReference>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub polarity: Polarity,
}

pub struct Utterance {
    pub sentences: Vec<Sentence>,
}
```

#### Week 2: Ontology and Traits

**Day 1-2: Ontology structure**
```rust
pub struct Ontology {
    concepts: HashMap<ConceptId, ConceptDefinition>,
    type_hierarchy: TypeHierarchy,
}

impl Ontology {
    pub fn validate_semantic_types(&self, frame: &Frame) -> Result<(), DeductionError>;
    pub fn inherit_features(&self, entity: &mut Entity);
}
```

**Day 3-4: Load ontology from RON**
- Parse `data/ontology/ontology.ron`
- Build type hierarchy
- Implement validation logic

**Day 5: Core traits**
```rust
pub trait IMeaningRepresentation {
    type Input;
    type Output;
    
    fn to_interlingua(&self, input: &Self::Input) -> Result<Interlingua, ParseError>;
    fn from_interlingua(&self, il: &Interlingua) -> Result<Self::Output, GenerateError>;
}
```

#### Week 3: Testing Infrastructure

**Day 1-2: Set up test framework**
- Configure `insta` for golden tests
- Create test utilities
- Set up CI/CD (optional)

**Day 3-5: Unit tests for core types**
- Test FeatureBundle operations
- Test Frame construction
- Test Ontology validation
- Test error cases

### Milestone Criteria
✅ All core types compile without errors  
✅ Ontology loads from RON file  
✅ Ontology validation works for Transfer frame  
✅ All unit tests pass  
✅ Can create InterlinguaNode manually in tests  

### Dependencies
- None (starting phase)

---

## Phase 2: Polish Morphology (3-4 weeks)

### Goals
- Implement algorithmic noun declension
- Implement algorithmic verb conjugation
- Support 4+ noun paradigms and 2+ verb paradigms

### Deliverables
- `src/engines/pl/morphology.rs` - Morphological rules
- `data/morphology/pl/*.ron` - Paradigm definitions
- Unit tests for each paradigm

### Detailed Tasks

#### Week 4: Morphology Framework

**Day 1-2: Define morphology structures**
```rust
pub struct MorphRule {
    pub conditions: Vec<Condition>,
    pub operations: Vec<Operation>,
}

pub enum Operation {
    ReplaceSuffix { from: String, to: String },
    AddSuffix(String),
    RemoveSuffix(String),
}

pub struct MorphParadigm {
    pub name: String,
    pub rules: Vec<MorphRule>,
}
```

**Day 3-5: Implement rule application**
```rust
impl MorphologyEngine {
    pub fn inflect_noun(
        &self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<String, MorphError>;
    
    pub fn inflect_verb(
        &self,
        lemma: &str,
        tense: Tense,
        aspect: Aspect,
        person: Person,
        number: Number,
    ) -> Result<String, MorphError>;
}
```

#### Week 5: Noun Paradigms

**Day 1-2: Neuter nouns (-o)**
- "jabłko" paradigm
- Handle all 7 cases
- Test with: jabłko, dziecko, okno

**Day 3-4: Feminine nouns (-a)**
- "książka" paradigm
- Handle consonant changes (k→c, g→dz)
- Test with: książka, kobieta, szkoła

**Day 5: Masculine nouns (consonant)**
- "dom" paradigm
- Test with: dom, kot, stół

#### Week 6: Verb Paradigms

**Day 1-2: -ać verbs (imperfective)**
- "czytać" paradigm
- Handle all persons and tenses
- Test with: czytać, pisać, szukać

**Day 3-4: -eć verbs (imperfective)**
- "widzieć" paradigm
- Handle vowel changes
- Test with: widzieć, siedzieć, leżeć

**Day 5: Aspect pairs**
- Implement perfective/imperfective pairing
- "czytać/przeczytać", "pisać/napisać"

#### Week 7: Integration and Testing

**Day 1-3: Load paradigms from RON**
- Parse `data/morphology/pl/*.ron`
- Build paradigm index
- Implement paradigm selection

**Day 4-5: Comprehensive testing**
- Golden tests for each paradigm
- Test edge cases
- Test error handling

### Milestone Criteria
✅ Can inflect "jabłko" in all 7 cases correctly  
✅ Can inflect "książka" with consonant changes  
✅ Can conjugate "czytać" in present tense  
✅ Aspect pairs work correctly  
✅ 90% of unit tests pass  
✅ Golden tests for key paradigms exist  

### Dependencies
- Phase 1 complete (need core types)

---

## Phase 3: Parser + Deduction Engine (4-5 weeks)

### Goals
- Implement basic Polish parser (SVO sentences)
- Implement full Deduction Engine
- Resolve case ambiguities
- Handle basic pronouns

### Deliverables
- `src/engines/pl/parser.rs` - Polish parser
- `src/core/deduction.rs` - Deduction engine
- Integration tests for simple sentences

### Detailed Tasks

#### Week 8: Basic Parser

**Day 1-2: Tokenization**
```rust
impl PolishParser {
    pub fn tokenize(&self, input: &str) -> Vec<Token>;
}
```

**Day 3-4: Morphological analysis**
```rust
impl PolishParser {
    pub fn analyze_morphology(&self, tokens: &[Token]) -> Vec<MorphAnalysis>;
}
```

**Day 5: Build partial structure**
```rust
impl PolishParser {
    pub fn build_partial_structure(
        &self,
        morph_analyzed: &[MorphAnalysis],
    ) -> Result<Utterance, ParseError>;
}
```

#### Week 9: Deduction Engine - Verb Frames

**Day 1-2: apply_verb_frames()**
- Identify main verb
- Look up frame template
- Collect NP candidates
- Assign roles using priorities

**Day 3-4: Test verb frame matching**
- Test Transfer frame
- Test Motion frame
- Test Perception frame

**Day 5: Handle edge cases**
- Missing required roles
- Unknown verbs
- Multiple verbs (coordination)

#### Week 10: Deduction Engine - Case Resolution

**Day 1-2: resolve_cases_and_roles()**
- Implement priority-based case assignment
- Handle polarity effects (ACC→GEN in negative)
- Validate with ontology

**Day 3-4: resolve_pronouns_within_sentence()**
- Handle reflexives (się, sobie)
- Handle 1st/2nd person pronouns
- Handle 3rd person pronouns (simple heuristics)

**Day 5: anchor_temporals()**
- Convert deictic expressions
- Handle "wczoraj", "dzisiaj", "jutro"
- Handle temporal adverbs

#### Week 11: Deduction Engine - Validation

**Day 1-2: validate_and_inherit_from_ontology()**
- Check semantic type constraints
- Inherit features from ontology
- Report violations

**Day 3-4: normalize_features()**
- Normalize aspect and tense
- Propagate polarity
- Handle modality

**Day 5: Integration testing**
- Test full deduction pipeline
- Test with complex sentences
- Test error cases

#### Week 12: Parser Integration

**Day 1-2: Integrate parser and deduction**
```rust
impl PolishParser {
    pub fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
        let tokens = self.tokenize(input);
        let morph_analyzed = self.analyze_morphology(&tokens);
        let partial = self.build_partial_structure(&morph_analyzed)?;
        
        let context = DeductionContext::new(...);
        let final_utterance = deduction::deduce(partial, &LanguageId::PL, &context)?;
        
        Ok(final_utterance)
    }
}
```

**Day 3-5: Comprehensive testing**
- Golden tests for key sentences
- Test "Tomek dał jabłko Izie"
- Test "Tomek nie dał jabłka Izie"
- Test "Student widzi profesora"

### Milestone Criteria
✅ Parser tokenizes and analyzes morphology correctly  
✅ Deduction resolves case ambiguities  
✅ Deduction handles negative sentences (ACC→GEN)  
✅ Deduction validates semantic types  
✅ Can parse "Tomek dał jabłko Izie wczoraj" correctly  
✅ 80% of integration tests pass  

### Dependencies
- Phase 1 complete (core types)
- Phase 2 complete (morphology)

---

## Phase 4: Polish + English Generator (5-7 weeks)

### Goals
- Implement Polish generator
- Implement English parser (simple)
- Implement English generator
- Achieve end-to-end translation

### Deliverables
- `src/engines/pl/generator.rs` - Polish generator
- `src/engines/en/parser.rs` - English parser
- `src/engines/en/generator.rs` - English generator
- End-to-end tests

### Detailed Tasks

#### Week 13: Polish Generator - Phase 1

**Day 1-2: Frame analysis**
```rust
impl PolishGenerator {
    fn analyze_frame(&self, frame: &Frame) -> Vec<(SemanticRole, Entity)>;
}
```

**Day 3-4: Lexical selection**
```rust
impl PolishGenerator {
    fn select_lexicon(
        &self,
        entity: &Entity,
        lexicon: &Lexicon,
    ) -> Result<String, GenerateError>;
}
```

**Day 5: Morphological inflection**
```rust
impl PolishGenerator {
    fn inflect(
        &self,
        lemma: &str,
        features: &FeatureBundle,
        morphology: &MorphologyEngine,
    ) -> Result<String, GenerateError>;
}
```

#### Week 14: Polish Generator - Phase 2

**Day 1-2: Word order determination**
```rust
impl PolishGenerator {
    fn determine_word_order(
        &self,
        roles: &[(SemanticRole, String)],
        descriptor: &LanguageDescriptor,
    ) -> Vec<String>;
}
```

**Day 3-4: Case assignment for generation**
```rust
impl PolishGenerator {
    fn assign_cases(
        &self,
        frame: &Frame,
        polarity: Polarity,
    ) -> Vec<(SemanticRole, Case)>;
}
```

**Day 5: Surface realization**
```rust
impl PolishGenerator {
    fn surface_realize(&self, words: Vec<String>) -> String;
}
```

#### Week 15: Polish Generator - Testing

**Day 1-2: Test Transfer frame**
- Generate "Tomek dał jabłko Izie"
- Generate "Student dał książkę profesorowi"
- Test with different tenses

**Day 3-4: Test negative sentences**
- Generate "Tomek nie dał jabłka Izie" (GEN case)
- Test polarity effects

**Day 5: Test other frames**
- Motion frame
- Perception frame
- Handle edge cases

#### Week 16: English Parser

**Day 1-2: Simple tokenizer**
```rust
impl EnglishParser {
    pub fn tokenize(&self, input: &str) -> Vec<Token>;
}
```

**Day 3-4: Simple morphological analysis**
- Handle basic verb tenses
- Handle plural nouns
- Handle articles (a, the)

**Day 5: Build InterlinguaNode**
- Map English SVO to semantic roles
- Handle articles as definiteness

#### Week 17: English Generator

**Day 1-2: Frame analysis and lexical selection**
- Similar to Polish generator
- Handle articles

**Day 3-4: Word order and surface realization**
- Strict SVO order
- Add articles where needed
- Handle prepositional phrases

**Day 5: Testing**
- Test "John gave Mary the book"
- Test "John gave the book to Mary"
- Test negative sentences

#### Week 18-19: End-to-End Integration

**Day 1-3: UniversalTranslator**
```rust
impl UniversalTranslator {
    pub fn translate(
        &self,
        input: &str,
        from: LanguageId,
        to: LanguageId,
    ) -> Result<String, TranslateError>;
}
```

**Day 4-5: Capability checking**
```rust
impl UniversalTranslator {
    pub fn check_capability(
        &self,
        il: &Interlingua,
        target: &dyn IMeaningRepresentation,
    ) -> Result<(), TranslateError>;
}
```

**Day 6-7: Comprehensive testing**
- Test PL→EN translation
- Test EN→PL translation
- Test roundtrip translation
- Create golden tests

### Milestone Criteria
✅ Polish generator produces correct sentences  
✅ English parser handles simple SVO sentences  
✅ English generator produces correct sentences  
✅ PL→EN translation works for Transfer frame  
✅ EN→PL translation works for Transfer frame  
✅ End-to-end test "Tomek dał jabłko Izie" → "Tomek gave an apple to Iza" passes  

### Dependencies
- Phase 1, 2, 3 complete

---

## Phase 5: Integration + API (2-3 weeks)

### Goals
- Implement LexFlexAPI
- Implement builder pattern
- Add logging
- Create CLI interface

### Deliverables
- `src/api.rs` - Public API
- `src/translator.rs` - UniversalTranslator (refined)
- `src/main.rs` - CLI interface
- API documentation

### Detailed Tasks

#### Week 20: LexFlexAPI

**Day 1-2: API structure**
```rust
pub struct LexFlexAPI {
    translator: UniversalTranslator,
    logger: Logger,
}

impl LexFlexAPI {
    pub fn translate(
        &self,
        input: &str,
        from: LanguageId,
        to: LanguageId,
    ) -> Result<String, TranslateError>;
}
```

**Day 3-4: Builder pattern**
```rust
pub struct LexFlexBuilder {
    // ... fields
}

impl LexFlexBuilder {
    pub fn new() -> Self;
    pub fn with_polish_engine(mut self) -> Self;
    pub fn with_english_engine(mut self) -> Self;
    pub fn with_logging(mut self) -> Self;
    pub fn build(self) -> LexFlexAPI;
}
```

**Day 5: Error handling**
- Implement graceful degradation
- Add detailed error messages
- Handle capability mismatches

#### Week 21: Logging and CLI

**Day 1-2: Logging infrastructure**
- Implement per-request logging
- Add performance metrics
- Log deduction steps

**Day 3-4: CLI interface**
```bash
lexflex translate "Tomek dał jabłko Izie" --from pl --to en
lexflex parse "Tomek dał jabłko Izie" --lang pl
lexflex languages
```

**Day 5: Documentation**
- API documentation
- Usage examples
- Error handling guide

#### Week 22: Polish and Testing

**Day 1-2: Refine API**
- Add TranslateOptions
- Add best_effort mode
- Add return_interlingua option

**Day 3-4: Integration testing**
- Test full API
- Test error cases
- Test performance

**Day 5: Documentation review**
- Update all documentation
- Add examples
- Review for consistency

### Milestone Criteria
✅ LexFlexAPI works correctly  
✅ Builder pattern is easy to use  
✅ CLI interface works  
✅ Logging captures all relevant information  
✅ API documentation is complete  

### Dependencies
- Phase 4 complete

---

## Phase 6: Testing + Hardening (2 weeks)

### Goals
- Comprehensive test coverage
- Performance optimization
- Documentation completion
- Bug fixing

### Deliverables
- Golden test suite
- Performance benchmarks
- Complete documentation
- Stable v0.1 release

### Detailed Tasks

#### Week 23: Testing

**Day 1-2: Golden tests**
- Create golden tests for all key sentences
- Test PL→EN and EN→PL
- Test edge cases

**Day 3-4: Unit test coverage**
- Identify untested code paths
- Add missing tests
- Aim for 80% coverage

**Day 5: Integration tests**
- Test full pipeline
- Test error handling
- Test capability checking

#### Week 24: Hardening

**Day 1-2: Performance optimization**
- Profile critical paths
- Optimize morphology lookup
- Optimize deduction

**Day 3-4: Bug fixing**
- Fix all known bugs
- Handle edge cases
- Improve error messages

**Day 5: Documentation completion**
- Review all documentation
- Add missing examples
- Update IMPLEMENTATION_GUIDE.md

### Milestone Criteria
✅ 80% test coverage  
✅ All golden tests pass  
✅ Performance is acceptable (<100ms per sentence)  
✅ No critical bugs  
✅ Documentation is complete and accurate  

### Dependencies
- Phase 5 complete

---

## Summary

| Phase | Duration | Key Deliverables | Success Criteria |
|-------|----------|------------------|------------------|
| 1. Core Types | 2-3 weeks | Core data structures, ontology | Can create InterlinguaNode |
| 2. Polish Morphology | 3-4 weeks | Noun/verb inflection | Can inflect key paradigms |
| 3. Parser + Deduction | 4-5 weeks | Parser, deduction engine | Can parse simple sentences |
| 4. Generators | 5-7 weeks | PL/EN generators | End-to-end translation works |
| 5. Integration + API | 2-3 weeks | API, CLI | API is usable |
| 6. Testing + Hardening | 2 weeks | Tests, bug fixes | Stable release |

**Total: 18-24 weeks**

## Risk Mitigation

### High-Risk Areas

1. **Polish Generator** - Most complex component
   - **Mitigation:** Start early (Phase 4, Week 13)
   - **Fallback:** Accept slightly stiff sentences initially

2. **Deduction Engine** - Critical for correctness
   - **Mitigation:** Implement iteratively, test thoroughly
   - **Fallback:** Use best_effort mode for ambiguous cases

3. **Morphology** - Foundation for everything
   - **Mitigation:** Start with simple paradigms, expand gradually
   - **Fallback:** Hardcode irregular forms if needed

### Critical Path

```
Phase 1 (Core) → Phase 2 (Morphology) → Phase 3 (Parser/Deduction)
                                              ↓
                                         Phase 4 (Generators)
                                              ↓
                                         Phase 5 (Integration)
                                              ↓
                                         Phase 6 (Testing)
```

**Critical path duration:** 18-24 weeks

## Success Metrics

### Functional Requirements
✅ PL→EN translation works for simple sentences  
✅ EN→PL translation works for simple sentences  
✅ Handles Transfer, Motion, Perception frames  
✅ Handles negative sentences  
✅ Handles basic temporal expressions  

### Non-Functional Requirements
✅ Translation time < 100ms per sentence  
✅ Memory usage < 50MB  
✅ Test coverage > 80%  
✅ No critical bugs  

### Quality Metrics
✅ Golden tests for 20+ key sentences  
✅ Unit tests for all core components  
✅ Integration tests for full pipeline  
✅ Documentation is complete and accurate  

## Next Steps

1. **Start with Phase 1** - Implement core types
2. **Build iteratively** - Test each phase before moving to next
3. **Document as you go** - Update documentation continuously
4. **Test early and often** - Write tests alongside implementation
5. **Refactor when needed** - Don't let technical debt accumulate

## References

- [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Detailed implementation guidance
- [GENERATOR.md](./GENERATOR.md) - Generator implementation details
- [DEDUCTION.md](./DEDUCTION.md) - Deduction engine details
- [END_TO_END_EXAMPLE.md](./END_TO_END_EXAMPLE.md) - Complete pipeline example
