# Generator Implementation Guide

This document provides detailed guidance on implementing the Generator component, which converts `InterlinguaNode` representations into natural language text.

## Overview

The Generator is the **hardest component** in lexFlex, especially for Polish. It must:
- Convert semantic frames into grammatically correct sentences
- Apply morphological inflection
- Choose appropriate word order
- Select pronoun forms (enclitic vs full)
- Handle aspect and tense
- Respect language-specific rules from `LanguageDescriptor`

## Critical Design Principle

**The Generator is purely realizational - it makes NO semantic decisions.**

The Generator's sole responsibility is to faithfully realize the complete `InterlinguaNode` structure produced by the Deduction Engine. It should:
- ✅ Read features from `InterlinguaNode` and `LanguageDescriptor`
- ✅ Apply morphological rules from `MorphologyEngine`
- ✅ Choose word order based on `LanguageDescriptor`
- ❌ NEVER infer missing semantic information
- ❌ NEVER resolve ambiguities (that's Deduction's job)
- ❌ NEVER make pragmatic decisions (register, emphasis, etc.)

**Why this matters:** If the Generator tries to make semantic decisions, it will duplicate logic from Deduction, leading to inconsistencies. The Deduction Engine must provide a COMPLETE, UNAMBIGUOUS `InterlinguaNode`. The Generator just realizes it.

**Example:**
```rust
// ❌ WRONG: Generator trying to resolve ambiguity
fn generate(il: &InterlinguaNode) -> String {
    // Generator shouldn't do this!
    if il.frames[0].theme.case.is_none() {
        // Try to guess the case
        il.frames[0].theme.case = Some(Case::Accusative);
    }
    // ...
}

// ✅ CORRECT: Generator assumes Deduction already resolved everything
fn generate(il: &InterlinguaNode) -> String {
    // Trust that Deduction provided complete information
    let theme_case = il.frames[0].theme.features.case
        .expect("Deduction should have assigned case");
    // ...
}
```

## Architecture

### Three-Phase Approach

**Phase 1: Minimum Viable Generator**
- Support only `Frame::Transfer`
- Hardcoded SVO word order
- Basic morphological inflection
- No style/register considerations
- **Rationale:** Transfer is the most common frame and tests the entire pipeline

**Phase 2: Extended Generator**
- Support multiple frame types (Motion, Perception, etc.)
- Pronoun selection (enclitic vs full form)
- Simple article logic for English
- Better word order heuristics
- **Rationale:** Once Transfer works, expand coverage incrementally

**Phase 3: LanguageDescriptor-Driven Generator**
- Use `LanguageDescriptor` for generation decisions
- Improved naturalness
- Register/style awareness
- **Rationale:** Move from hardcoded to data-driven generation

### Generation Pipeline

```
InterlinguaNode (from Deduction)
    ↓
1. Frame Analysis (extract roles)
    ↓
2. Lexical Selection (choose words from lexicon)
    ↓
3. Case Assignment (map roles to grammatical cases)
    ↓
4. Morphological Inflection (apply inflection rules)
    ↓
5. Verb Form Selection (choose tense/aspect/person/number)
    ↓
6. Pronoun Form Selection (enclitic vs full)
    ↓
7. Word Order Determination (based on LanguageDescriptor)
    ↓
8. Surface Realization (assemble final sentence)
    ↓
Natural Language Text
```

## Detailed Algorithm for Each Frame Type

### Frame::Transfer (give, send, hand)

**Input:** `Frame::Transfer { agent, recipient, theme }`

**Step-by-step algorithm:**

```rust
fn generate_transfer(
    frame: &Frame::Transfer,
    sentence: &Sentence,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<Vec<String>, GenerateError> {
    
    let mut words = Vec::new();
    
    // Step 1: Extract roles
    let agent = &frame.agent;
    let recipient = &frame.recipient;
    let theme = &frame.theme;
    
    // Step 2: Assign cases based on role and polarity
    let agent_case = Case::Nominative; // Agent is always NOM
    
    let recipient_case = Case::Dative; // Recipient is always DAT
    
    let theme_case = match sentence.polarity {
        Polarity::Positive => Case::Accusative, // ACC in positive
        Polarity::Negative => Case::Genitive,   // GEN in negative (Polish-specific!)
    };
    
    // Step 3: Lexical selection
    let agent_lemma = lexicon.lookup(&descriptor.language, &agent.concept)?;
    let recipient_lemma = lexicon.lookup(&descriptor.language, &recipient.concept)?;
    let theme_lemma = lexicon.lookup(&descriptor.language, &theme.concept)?;
    
    // Step 4: Morphological inflection
    let agent_form = morphology.inflect_noun(
        &agent_lemma,
        agent_case,
        agent.features.number.unwrap_or(Number::Singular),
        agent.features.gender,
    )?;
    
    let recipient_form = morphology.inflect_noun(
        &recipient_lemma,
        recipient_case,
        recipient.features.number.unwrap_or(Number::Singular),
        recipient.features.gender,
    )?;
    
    let theme_form = morphology.inflect_noun(
        &theme_lemma,
        theme_case,
        theme.features.number.unwrap_or(Number::Singular),
        theme.features.gender,
    )?;
    
    // Step 5: Verb form selection
    let verb_concept = "give"; // From frame type
    let verb_lemma = lexicon.lookup_verb(&descriptor.language, verb_concept)?;
    
    let verb_form = morphology.inflect_verb(
        &verb_lemma,
        sentence.tense.unwrap_or(Tense::Present),
        sentence.aspect.unwrap_or(Aspect::Perfective),
        agent.features.person.unwrap_or(Person::Third),
        agent.features.number.unwrap_or(Number::Singular),
    )?;
    
    // Step 6: Determine word order
    match descriptor.syntax.word_order {
        WordOrder::SVO => {
            // Subject (Agent) + Verb + IndirectObject (Recipient) + DirectObject (Theme)
            words.push(agent_form);
            words.push(verb_form);
            words.push(recipient_form);
            words.push(theme_form);
        }
        WordOrder::SOV => {
            // Subject + Objects + Verb
            words.push(agent_form);
            words.push(recipient_form);
            words.push(theme_form);
            words.push(verb_form);
        }
        _ => {
            // Default to SVO
            words.push(agent_form);
            words.push(verb_form);
            words.push(recipient_form);
            words.push(theme_form);
        }
    }
    
    // Step 7: Add temporal modifiers
    if let Some(temporal) = &sentence.temporal {
        let temporal_form = generate_temporal(temporal, lexicon)?;
        words.push(temporal_form);
    }
    
    Ok(words)
}
```

**Example walkthrough:**
```
Input: Frame::Transfer {
    agent: Entity { concept: "person", name: "Tomek", features: { gender: Masc, number: Sg } },
    recipient: Entity { concept: "person", name: "Iza", features: { gender: Fem, number: Sg } },
    theme: Entity { concept: "apple", features: { gender: Neut, number: Sg } }
}
Sentence: { tense: Past, aspect: Perfective, polarity: Positive }
Language: Polish

Step 1: Extract roles
  agent = Tomek
  recipient = Iza
  theme = apple

Step 2: Assign cases
  agent_case = Nominative
  recipient_case = Dative
  theme_case = Accusative (polarity is Positive)

Step 3: Lexical selection
  agent_lemma = "Tomek"
  recipient_lemma = "Iza"
  theme_lemma = "jabłko"

Step 4: Morphological inflection
  agent_form = "Tomek" (NOM.SG.M)
  recipient_form = "Izie" (DAT.SG.F)
  theme_form = "jabłko" (ACC.SG.N)

Step 5: Verb form selection
  verb_lemma = "dać" (perfective)
  verb_form = "dał" (PAST.3SG.M.PERF)

Step 6: Word order (SVO)
  ["Tomek", "dał", "Izie", "jabłko"]

Step 7: No temporal modifier

Output: "Tomek dał Izie jabłko"
```

### Frame::Motion (go, come, walk)

**Input:** `Frame::Motion { agent, goal: Option, source: Option }`

**Key differences from Transfer:**
- Agent is always Nominative
- Goal is Accusative (direction) or Locative (location)
- Source is Genitive (from)
- Verb choice depends on aspect (perfective vs imperfective)

```rust
fn generate_motion(
    frame: &Frame::Motion,
    sentence: &Sentence,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<Vec<String>, GenerateError> {
    
    let mut words = Vec::new();
    
    // Agent is always NOM
    let agent_case = Case::Nominative;
    let agent_form = inflect_entity(&frame.agent, agent_case, lexicon, morphology)?;
    
    // Goal: ACC (direction) or LOC (location)
    if let Some(goal) = &frame.goal {
        let goal_case = if is_directional_motion(sentence) {
            Case::Accusative // "idę do szkoły" (direction)
        } else {
            Case::Locative // "jestem w szkole" (location)
        };
        let goal_form = inflect_entity(goal, goal_case, lexicon, morphology)?;
        
        // Add preposition
        let preposition = match goal_case {
            Case::Accusative => "do", // to
            Case::Locative => "w",   // in
            _ => "",
        };
        
        words.push(agent_form);
        words.push(verb_form);
        words.push(preposition.to_string());
        words.push(goal_form);
    }
    
    // Source: GEN (from)
    if let Some(source) = &frame.source {
        let source_case = Case::Genitive;
        let source_form = inflect_entity(source, source_case, lexicon, morphology)?;
        
        // Add preposition
        let preposition = "z"; // from
        
        words.push(preposition.to_string());
        words.push(source_form);
    }
    
    Ok(words)
}
```

### Frame::Perception (see, hear, notice)

**Input:** `Frame::Perception { experiencer, stimulus }`

**Key differences:**
- Experiencer is Nominative (not Agent)
- Stimulus is Accusative
- No Recipient role

```rust
fn generate_perception(
    frame: &Frame::Perception,
    sentence: &Sentence,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<Vec<String>, GenerateError> {
    
    let mut words = Vec::new();
    
    // Experiencer is NOM
    let experiencer_case = Case::Nominative;
    let experiencer_form = inflect_entity(&frame.experiencer, experiencer_case, lexicon, morphology)?;
    
    // Stimulus is ACC
    let stimulus_case = Case::Accusative;
    let stimulus_form = inflect_entity(&frame.stimulus, stimulus_case, lexicon, morphology)?;
    
    // Verb: "widzieć" (see), "słyszeć" (hear)
    let verb_lemma = lexicon.lookup_verb(&descriptor.language, "see")?;
    let verb_form = morphology.inflect_verb(
        &verb_lemma,
        sentence.tense.unwrap_or(Tense::Present),
        sentence.aspect.unwrap_or(Aspect::Imperfective), // Perception is usually imperfective
        frame.experiencer.features.person.unwrap_or(Person::Third),
        frame.experiencer.features.number.unwrap_or(Number::Singular),
    )?;
    
    // Word order: SVO
    words.push(experiencer_form);
    words.push(verb_form);
    words.push(stimulus_form);
    
    Ok(words)
}
```

## Pronoun Form Selection in Polish

Polish pronouns have multiple forms depending on stress, position, and context.

### Form Types

| Form Type | Examples | Usage |
|-----------|----------|-------|
| **Full form** | "mnie", "tobie", "jego", "ją" | Emphatic, sentence-initial, after preposition |
| **Enclitic form** | "mi", "ci", "go", "ją" | Unstressed, after verb, mid-sentence |
| **Prepositional form** | "niego", "niej", "nich" | After prepositions (special forms) |

### Decision Algorithm

```rust
fn select_pronoun_form(
    entity: &Entity,
    position_in_sentence: usize,
    sentence: &Sentence,
    descriptor: &LanguageDescriptor,
) -> String {
    
    // Rule 1: If entity is not a pronoun, return regular form
    if !entity.is_pronoun() {
        return get_regular_form(entity);
    }
    
    // Rule 2: Sentence-initial position → full form
    if position_in_sentence == 0 {
        return get_full_pronoun_form(entity);
    }
    
    // Rule 3: After preposition → prepositional form
    if is_after_preposition(position_in_sentence, sentence) {
        return get_prepositional_pronoun_form(entity);
    }
    
    // Rule 4: Emphatic context → full form
    if is_emphatic_context(entity, sentence) {
        return get_full_pronoun_form(entity);
    }
    
    // Rule 5: Default → enclitic form (if allowed by LanguageDescriptor)
    if descriptor.pragmatics.pronoun_strategy == PronounStrategy::EncliticPreferred {
        return get_enclitic_pronoun_form(entity);
    }
    
    // Rule 6: Fallback → full form
    get_full_pronoun_form(entity)
}

fn is_emphatic_context(entity: &Entity, sentence: &Sentence) -> bool {
    // Check if entity is contrasted with another entity
    // Example: "Mnie dał, nie tobie" (He gave to ME, not to you)
    
    // Check if entity has emphasis feature
    if entity.features.emphasis == Some(true) {
        return true;
    }
    
    // Check if sentence has contrastive structure
    if sentence.has_contrast() {
        return true;
    }
    
    false
}

fn is_after_preposition(position: usize, sentence: &Sentence) -> bool {
    // Check if previous token is a preposition
    if position > 0 {
        let prev_token = &sentence.tokens[position - 1];
        return prev_token.pos == PartOfSpeech::Preposition;
    }
    false
}
```

### Examples

```
Example 1: Sentence-initial position
Input: Entity { concept: "person", pronoun: true, person: First, number: Sg }
Position: 0
Result: "Ja" (full form)
Output: "Ja dałem jabłko Izie"

Example 2: After verb (enclitic)
Input: Entity { concept: "person", pronoun: true, person: First, number: Sg, case: Dative }
Position: 2 (after verb)
Result: "mi" (enclitic form)
Output: "Tomek dał mi jabłko"

Example 3: After preposition
Input: Entity { concept: "person", pronoun: true, person: Third, number: Sg, gender: Masc, case: Genitive }
Position: 3 (after preposition "z")
Result: "niego" (prepositional form)
Output: "Tomek przyszedł z nim"

Example 4: Emphatic context
Input: Entity { concept: "person", pronoun: true, person: First, number: Sg, emphasis: true }
Position: 1
Result: "mnie" (full form, emphatic)
Output: "Tomek dał mnie, nie tobie"
```

## Interaction with LanguageDescriptor

The `LanguageDescriptor` drives generation decisions at every step.

### Key Descriptor Fields Used by Generator

| Field | Used In | Purpose |
|-------|---------|---------|
| `syntax.word_order` | Word Order Determination | SVO, SOV, VSO, etc. |
| `syntax.pro_drop` | Pronoun Selection | Whether subject can be omitted |
| `morphology.has_cases` | Case Assignment | Whether to assign grammatical cases |
| `morphology.has_articles` | Article Selection | Whether to add articles (a, the) |
| `morphology.aspect_type` | Verb Form Selection | Morphological vs periphrastic aspect |
| `pragmatics.pronoun_strategy` | Pronoun Form Selection | Enclitic vs full form preference |

### Example: How Descriptor Affects Generation

```rust
fn generate_with_descriptor(
    il: &InterlinguaNode,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<String, GenerateError> {
    
    let mut words = Vec::new();
    
    for frame in &il.sentences[0].frames {
        // Step 1: Check if language has cases
        if descriptor.morphology.has_cases {
            // Assign cases based on roles
            assign_cases(frame, &il.sentences[0].polarity)?;
        }
        
        // Step 2: Check word order
        let word_order = descriptor.syntax.word_order;
        let ordered_roles = determine_word_order(frame, word_order);
        
        // Step 3: Generate each role
        for (i, (role, entity)) in ordered_roles.iter().enumerate() {
            // Check pro-drop
            if i == 0 && role == &SemanticRole::Agent {
                if descriptor.syntax.pro_drop && should_drop_subject(entity) {
                    continue; // Omit subject
                }
            }
            
            // Add article if needed
            if descriptor.morphology.has_articles {
                let article = add_article(entity, is_first_mention(entity));
                if !article.is_empty() {
                    words.push(article);
                }
            }
            
            // Generate word form
            let form = generate_entity(entity, lexicon, morphology)?;
            words.push(form);
        }
        
        // Step 4: Generate verb
        let verb_form = select_verb_form(frame, descriptor, lexicon, morphology)?;
        insert_verb(&mut words, verb_form, word_order);
    }
    
    Ok(words.join(" "))
}
```

### Polish vs English Descriptor Comparison

| Feature | Polish Descriptor | English Descriptor | Impact on Generator |
|---------|------------------|-------------------|---------------------|
| `has_cases` | `true` | `false` | Polish assigns cases, English doesn't |
| `has_articles` | `false` | `true` | English adds articles, Polish doesn't |
| `pro_drop` | `true` | `false` | Polish can omit subject, English can't |
| `word_order` | `SVO` (flexible) | `SVO` (strict) | Polish is more flexible |
| `aspect_type` | `Morphological` | `Periphrastic` | Polish uses verb pairs, English uses auxiliaries |
| `pronoun_strategy` | `EncliticPreferred` | `FullFormsOnly` | Polish uses enclitics, English uses full forms |

**Example:**
```
Input: Frame::Transfer { agent: "he", recipient: "her", theme: "book" }

Polish Generator (with Polish descriptor):
  - has_cases = true → assign cases (NOM, DAT, ACC)
  - has_articles = false → no articles
  - pro_drop = true → can omit "he" if context allows
  - pronoun_strategy = EncliticPreferred → use "mu", "jej"
  - Output: "Dał jej książkę" (subject omitted, enclitic pronouns)

English Generator (with English descriptor):
  - has_cases = false → no case assignment
  - has_articles = true → add "the" or "a"
  - pro_drop = false → must include "He"
  - pronoun_strategy = FullFormsOnly → use "him", "her"
  - Output: "He gave her the book" (subject required, full pronouns)
```

## Detailed Implementation

### Step 1: Frame Analysis

Extract semantic roles and their entities from the frame.

```rust
fn analyze_frame(frame: &Frame) -> Vec<(SemanticRole, Entity)> {
    match frame {
        Frame::Transfer { agent, recipient, theme } => {
            vec![
                (SemanticRole::Agent, agent.clone()),
                (SemanticRole::Recipient, recipient.clone()),
                (SemanticRole::Theme, theme.clone()),
            ]
        }
        Frame::Motion { agent, goal, source } => {
            let mut roles = vec![(SemanticRole::Agent, agent.clone())];
            if let Some(g) = goal {
                roles.push((SemanticRole::Goal, g.clone()));
            }
            if let Some(s) = source {
                roles.push((SemanticRole::Source, s.clone()));
            }
            roles
        }
        // ... other frames
    }
}
```

### Step 2: Lexical Selection

Choose appropriate words for each semantic role.

```rust
fn select_lexicon(
    entity: &Entity,
    language: LanguageId,
    lexicon: &Lexicon,
) -> Result<String, GenerateError> {
    // Look up concept in lexicon
    let entry = lexicon.lookup(&language, &entity.concept)
        .ok_or(GenerateError::MissingLexicalEntry {
            concept: entity.concept.clone(),
        })?;
    
    // Choose appropriate form based on features
    entry.lemma.clone()
}
```

### Step 3: Morphological Inflection

Apply morphological rules based on grammatical features.

```rust
fn inflect(
    lemma: &str,
    features: &FeatureBundle,
    morphology: &MorphologyEngine,
) -> Result<String, GenerateError> {
    // Determine required inflection
    let case = features.case.unwrap_or(Case::Nominative);
    let number = features.number.unwrap_or(Number::Singular);
    let gender = features.gender;
    
    // Apply inflection rules
    morphology.inflect_noun(lemma, case, number, gender)
}
```

**Key for Polish:**
- Agent → Nominative case
- Recipient → Dative case
- Theme → Accusative case (or Genitive in negative sentences)

### Step 4: Word Order Determination

Choose word order based on language and frame type.

```rust
fn determine_word_order(
    roles: &[(SemanticRole, String)], // (role, inflected_word)
    language: LanguageId,
    descriptor: &LanguageDescriptor,
) -> Vec<String> {
    match language {
        LanguageId::PL => {
            // Polish: relatively flexible, but SVO is default
            // Agent (subject) first
            // Then verb
            // Then objects
            let mut order = Vec::new();
            
            // Find agent
            if let Some((_, agent)) = roles.iter().find(|(r, _)| *r == SemanticRole::Agent) {
                order.push(agent.clone());
            }
            
            // Add verb (handled separately)
            
            // Add recipient (indirect object)
            if let Some((_, recipient)) = roles.iter().find(|(r, _)| *r == SemanticRole::Recipient) {
                order.push(recipient.clone());
            }
            
            // Add theme (direct object)
            if let Some((_, theme)) = roles.iter().find(|(r, _)| *r == SemanticRole::Theme) {
                order.push(theme.clone());
            }
            
            order
        }
        LanguageId::EN => {
            // English: strict SVO
            // Similar logic but stricter
            // ...
        }
    }
}
```

### Step 5: Surface Realization

Assemble final sentence with proper spacing and punctuation.

```rust
fn surface_realize(words: Vec<String>) -> String {
    words.join(" ")
}
```

## Polish-Specific Considerations

### Case Assignment

For `Frame::Transfer` in Polish:

| Role | Case | Example |
|------|------|---------|
| Agent | Nominative | **Tomek** (NOM) |
| Recipient | Dative | **Izie** (DAT) |
| Theme | Accusative | **jabłko** (ACC) |

**Negative sentences:** Theme becomes Genitive
- "Tomek **nie dał** jabłk**a** Izie" (Genitive instead of Accusative)

### Pronoun Forms

Polish has two pronoun forms:
- **Full form:** "mnie", "tobie", "jego" (emphatic, standalone)
- **Enclitic form:** "mi", "ci", "go" (unstressed, after verb)

**Selection strategy (Phase 2+):**

```rust
fn select_pronoun_form(
    entity: &Entity,
    position_in_sentence: usize,
    is_emphatic: bool,
) -> String {
    if is_emphatic || position_in_sentence == 0 {
        // Use full form
        get_full_pronoun_form(entity)
    } else {
        // Use enclitic form
        get_enclitic_pronoun_form(entity)
    }
}
```

### Aspect and Tense

Polish verbs have aspect pairs:
- **Imperfective:** "dawać" (ongoing/repeated action)
- **Perfective:** "dać" (completed action)

**Selection from InterlinguaNode:**

```rust
fn select_verb_form(
    concept: &str,
    aspect: Aspect,
    tense: Tense,
    lexicon: &Lexicon,
) -> String {
    let verb_entry = lexicon.lookup_verb(concept);
    
    match aspect {
        Aspect::Imperfective => verb_entry.imperfective_lemma,
        Aspect::Perfective => verb_entry.perfective_lemma,
    }
}
```

## English-Specific Considerations

### Articles

English requires articles (`a`, `the`) for countable nouns.

**Simple strategy (Phase 2):**

```rust
fn add_article(
    entity: &Entity,
    is_first_mention: bool,
) -> String {
    if entity.features.definiteness == Some(Definiteness::Definite) {
        "the".to_string()
    } else if is_first_mention && entity.is_countable() {
        "a".to_string()
    } else {
        "".to_string()
    }
}
```

### Fixed Word Order

English has strict SVO order:
- Subject → Verb → Indirect Object → Direct Object
- "John gave Mary the book"

## Using LanguageDescriptor

The `LanguageDescriptor` provides language-specific rules.

```rust
fn generate_with_descriptor(
    il: &InterlinguaNode,
    descriptor: &LanguageDescriptor,
) -> String {
    // Check if language has articles
    if descriptor.has_articles {
        // Add articles logic
    }
    
    // Check default word order
    let word_order = descriptor.default_word_order;
    
    // Check pronoun rules
    let pronoun_strategy = descriptor.pronoun_strategy;
    
    // Generate based on descriptor
    // ...
}
```

## Example: Generating "Tomek dał jabłko Izie"

### Input InterlinguaNode

```rust
InterlinguaNode::Natural(Utterance {
    sentences: vec![Sentence {
        frames: vec![Frame::Transfer {
            agent: Entity {
                concept: ConceptId("person"),
                name: Some("Tomek"),
                features: FeatureBundle {
                    gender: Some(Gender::Masculine),
                    number: Some(Number::Singular),
                    case: None, // Will be assigned
                },
            },
            recipient: Entity {
                concept: ConceptId("person"),
                name: Some("Iza"),
                features: FeatureBundle {
                    gender: Some(Gender::Feminine),
                    number: Some(Number::Singular),
                    case: None,
                },
            },
            theme: Entity {
                concept: ConceptId("apple"),
                name: None,
                features: FeatureBundle {
                    gender: Some(Gender::Neuter),
                    number: Some(Number::Singular),
                    case: None,
                },
            },
        }],
        tense: Some(Tense::Past),
        aspect: Some(Aspect::Perfective),
        polarity: Polarity::Positive,
    }],
})
```

### Generation Steps

1. **Frame Analysis:**
   - Extract roles: Agent (Tomek), Recipient (Iza), Theme (apple)

2. **Case Assignment (Polish):**
   - Agent → Nominative: "Tomek"
   - Recipient → Dative: "Izie"
   - Theme → Accusative: "jabłko"

3. **Lexical Selection:**
   - Agent: "Tomek"
   - Recipient: "Iza" → inflect to "Izie"
   - Theme: "jabłko"

4. **Verb Selection:**
   - Concept: "give"
   - Aspect: Perfective → "dać"
   - Tense: Past → "dał"

5. **Word Order:**
   - SVO: "Tomek dał jabłko Izie"

6. **Surface Realization:**
   - "Tomek dał jabłko Izie"

### Output

**Polish:** "Tomek dał jabłko Izie"

**English:** "Tomek gave an apple to Iza" or "Tomek gave Iza an apple"

## Common Pitfalls

1. **Don't hardcode word forms** - Use morphological rules
2. **Don't ignore case assignment** - Critical for Polish
3. **Don't forget aspect pairs** - Polish verbs require this
4. **Don't assume fixed word order** - Polish is flexible
5. **Don't skip LanguageDescriptor** - Use it for language-specific rules

## Testing the Generator

```rust
#[test]
fn test_transfer_frame_polish() {
    let il = create_transfer_interlingua("Tomek", "Iza", "apple");
    let generator = PolishGenerator::new();
    
    let result = generator.generate(&il);
    
    assert!(result.contains("Tomek"));
    assert!(result.contains("dał"));
    assert!(result.contains("jabłko"));
    assert!(result.contains("Izie"));
}

#[test]
fn test_transfer_frame_english() {
    let il = create_transfer_interlingua("Tomek", "Iza", "apple");
    let generator = EnglishGenerator::new();
    
    let result = generator.generate(&il);
    
    assert!(result.contains("Tomek"));
    assert!(result.contains("gave"));
    assert!(result.contains("apple"));
}
```

## Next Steps

1. Implement Phase 1 (Transfer frame only)
2. Test with simple sentences
3. Add Phase 2 features (pronouns, articles)
4. Implement Phase 3 (LanguageDescriptor-driven)
5. Expand to other frame types

## Implementation Phases with Rationale

### Phase 1: Minimum Viable Generator (Week 13-15)

**Scope:**
- Only `Frame::Transfer`
- Hardcoded SVO word order
- Basic morphological inflection (nouns only)
- No pronoun selection (use full forms only)
- No articles (English)
- No pro-drop (Polish)

**Rationale:**
- Transfer is the most common frame in v0.1 test suite
- SVO is the default for both Polish and English
- Nouns are simpler than verbs (no aspect pairs yet)
- Full pronoun forms are always correct (just less natural)
- This phase validates the entire pipeline without complexity

**Success criteria:**
- ✅ Can generate "Tomek dał jabłko Izie" correctly
- ✅ Can generate "Tomek gave an apple to Iza" correctly
- ✅ Handles basic case assignment (NOM, ACC, DAT)
- ✅ Integrates with MorphologyEngine

**Known limitations:**
- Only works for Transfer frame
- Word order is hardcoded (not flexible)
- No pronoun form selection
- No articles in English
- Output may sound stiff

### Phase 2: Extended Generator (Week 16-18)

**Scope:**
- Add `Frame::Motion` and `Frame::Perception`
- Pronoun form selection (enclitic vs full)
- Article logic for English
- Pro-drop for Polish
- Better word order heuristics

**Rationale:**
- Motion and Perception are common frames
- Pronoun selection improves naturalness significantly
- Articles are required for grammatical English
- Pro-drop is natural for Polish
- This phase makes output more natural

**Success criteria:**
- ✅ Can generate Motion frames: "Tomek idzie do szkoły"
- ✅ Can generate Perception frames: "Tomek widzi Iza"
- ✅ Uses enclitic pronouns: "Tomek dał mi jabłko"
- ✅ Adds articles in English: "Tomek gave the book"
- ✅ Omits subject in Polish when appropriate: "Dał jabłko"

**Known limitations:**
- Still limited to 3 frame types
- Word order heuristics may be suboptimal
- No register/style awareness

### Phase 3: LanguageDescriptor-Driven Generator (Week 19-20)

**Scope:**
- Use `LanguageDescriptor` for all generation decisions
- Improved naturalness
- Register/style awareness (basic)
- Support for additional frames

**Rationale:**
- Move from hardcoded to data-driven generation
- Makes it easy to add new languages
- Improves maintainability
- This phase makes the generator truly flexible

**Success criteria:**
- ✅ All generation decisions come from LanguageDescriptor
- ✅ Can add new language by creating descriptor + lexicon
- ✅ Output sounds natural in both languages
- ✅ Handles register differences (formal vs informal)

**Known limitations:**
- Still limited vocabulary
- No complex syntax support
- Quality depends on descriptor quality

## Generator Limitations in v0.1

### What Works Well

✅ **Simple Transfer frames:** "Tomek dał jabłko Izie"  
✅ **Basic case assignment:** NOM, ACC, DAT handled correctly  
✅ **Morphological inflection:** Regular nouns and verbs  
✅ **Aspect pairs:** Perfective vs imperfective verbs  
✅ **Negative sentences:** ACC → GEN case change  
✅ **Temporal modifiers:** "wczoraj", "jutro" at sentence end  

### What Works Poorly

⚠️ **Pronoun form selection:** May use wrong form (full vs enclitic)  
⚠️ **Word order flexibility:** Polish allows more variation than generator produces  
⚠️ **Articles in English:** May choose wrong article (a vs the)  
⚠️ **Pro-drop in Polish:** May omit subject when it shouldn't  
⚠️ **Naturalness:** Output may sound stiff or robotic  

### What Doesn't Work

❌ **Complex sentences:** Subordinate clauses not supported  
❌ **Questions:** Question word order not implemented  
❌ **Passive voice:** Not supported  
❌ **Imperative mood:** Not supported  
❌ **Idiomatic expressions:** "dać komuś znać" → "let someone know"  
❌ **Collocations:** "mocna kawa" → "strong coffee"  
❌ **Register variation:** Formal vs informal not distinguished  
❌ **Discourse markers:** "no", "well", "you know" not generated  

### Realistic Expectations

**v0.1 output quality:**
- **Grammatical correctness:** ~90% (may have minor errors)
- **Naturalness:** ~60% (sounds somewhat stiff)
- **Coverage:** ~70% (works for simple sentences only)
- **Speed:** < 100ms per sentence

**What users should expect:**
- ✅ Simple sentences translate correctly
- ✅ Basic grammar is handled
- ⚠️ Output may need manual polishing
- ❌ Complex sentences will fail or produce incorrect output

**What users should NOT expect:**
- ❌ Human-like naturalness
- ❌ Idiomatic expressions
- ❌ Complex syntax
- ❌ Perfect grammar in all cases

### Workarounds for Limitations

**For stiff output:**
- Use post-processing with LLM to improve naturalness (v0.2+)
- Accept slightly unnatural output in v0.1
- Focus on correctness over naturalness

**For missing frames:**
- Use Transfer frame as approximation
- Break complex sentences into simple ones
- Add missing frames incrementally

**For pronoun errors:**
- Use full forms only (always correct, just less natural)
- Post-process to fix enclitic forms
- Add pronoun rules incrementally

**For article errors:**
- Default to "the" for definite, "a" for indefinite
- Post-process to fix article choice
- Add article rules incrementally

## Common Pitfalls and How to Avoid Them

### Pitfall 1: Generator Trying to Resolve Ambiguity

**Wrong:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    if il.frames[0].theme.case.is_none() {
        // Generator shouldn't do this!
        il.frames[0].theme.case = Some(Case::Accusative);
    }
    // ...
}
```

**Correct:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Trust that Deduction provided complete information
    let theme_case = il.frames[0].theme.features.case
        .expect("Deduction should have assigned case");
    // ...
}
```

**Why:** Generator should never make semantic decisions. That's Deduction's job.

### Pitfall 2: Hardcoding Word Forms

**Wrong:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Don't hardcode!
    let theme_form = if theme.concept == "apple" {
        "jabłko"
    } else {
        // ...
    };
}
```

**Correct:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Use lexicon and morphology
    let theme_lemma = lexicon.lookup(&language, &theme.concept)?;
    let theme_form = morphology.inflect_noun(&theme_lemma, case, number, gender)?;
}
```

**Why:** Hardcoding doesn't scale. Use data-driven approach.

### Pitfall 3: Ignoring LanguageDescriptor

**Wrong:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Don't hardcode language-specific rules!
    if language == LanguageId::PL {
        // Polish-specific logic
    } else if language == LanguageId::EN {
        // English-specific logic
    }
}
```

**Correct:**
```rust
fn generate(il: &InterlinguaNode, descriptor: &LanguageDescriptor) -> String {
    // Use descriptor for all language-specific decisions
    if descriptor.morphology.has_cases {
        // Assign cases
    }
    if descriptor.morphology.has_articles {
        // Add articles
    }
}
```

**Why:** LanguageDescriptor makes it easy to add new languages without changing generator code.

### Pitfall 4: Not Handling Missing Features

**Wrong:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Will panic if feature is missing!
    let case = entity.features.case.unwrap();
}
```

**Correct:**
```rust
fn generate(il: &InterlinguaNode) -> String {
    // Handle missing features gracefully
    let case = entity.features.case.unwrap_or(Case::Nominative);
    // Or return error
    let case = entity.features.case
        .ok_or(GenerateError::MissingFeature("case"))?;
}
```

**Why:** Deduction should provide complete information, but generator should handle edge cases gracefully.

### Pitfall 5: Over-Engineering Early

**Wrong:**
```rust
// Don't implement all features at once!
fn generate(il: &InterlinguaNode) -> String {
    // Handle all frames, all pronoun forms, all articles, etc.
    // This is too complex for v0.1
}
```

**Correct:**
```rust
// Start simple, expand incrementally
fn generate_transfer(il: &InterlinguaNode) -> String {
    // Only handle Transfer frame
    // Only handle basic cases
    // Only handle full pronoun forms
}
```

**Why:** Incremental development is easier to test and debug.

## References

- [DEDUCTION.md](./DEDUCTION.md) - For understanding input structure
- [MORPHOLOGY.md](./MORPHOLOGY.md) - For inflection rules
- [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md) - For language-specific rules
- [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) - For case assignment rules
