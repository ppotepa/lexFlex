# End-to-End Example: Complete Pipeline

This document shows a complete example of how a sentence flows through the entire lexFlex system, from input text to output text in another language.

## Example Sentence

**Input (Polish):** "Tomek dał jabłko Izie wczoraj"

**Expected Output (English):** "Tomek gave an apple to Iza yesterday"

---

## Linguistic Graph Navigation (Adam accompaniment example)

After parsing `"Tomek mieszka z żoną i córką."` the system builds a multi-layer graph on `Sentence.graph`:

1. **Surface:** `WordNode` chain with `Next`/`Prev` — `word.navig_next(graph)` walks forward.
2. **Semantic:** `EntityNode` (PERSON, WIFE, DAUGHTER) + `FrameNode` (Existence/LIVE) linked by `Realizes` and `HasRole`.
3. **Construction:** `PathBuilder::starting_with_verb().then_preposition(&["z"]).then_noun_phrase()` finds accompaniment paths; `find_construction(verb_id, "Accompaniment")` returns participant word ids.
4. **Generation:** Pipeline reads instrumental features → prep `with`; `TraceStep` records `involved_nodes` / `involved_edges`.
5. **Output:** `Tomek lives with a wife and a daughter.`

Benchmark traces under `results/runs/*-trace/detailed_traces.txt` include `LINGUISTIC GRAPH` RON with `words`, `entities`, `frames`, `edges` sections.

---

## Phase 1: Tokenization

The input text is split into tokens.

**Input:**
```
Tomek dał jabłko Izie wczoraj
```

**Output:**
```rust
vec![
    Token { text: "Tomek", position: 0 },
    Token { text: "dał", position: 1 },
    Token { text: "jabłko", position: 2 },
    Token { text: "Izie", position: 3 },
    Token { text: "wczoraj", position: 4 },
]
```

---

## Phase 2: Morphological Analysis

Each token is analyzed to determine its grammatical features.

**Analysis:**

| Token | Lemma | POS | Features |
|-------|-------|-----|----------|
| Tomek | Tomek | Noun | Gender: Masculine, Number: Singular, Case: Nominative, Animacy: Animate |
| dał | dać | Verb | Tense: Past, Aspect: Perfective, Person: 3rd, Number: Singular, Gender: Masculine |
| jabłko | jabłko | Noun | Gender: Neuter, Number: Singular, Case: **Ambiguous (NOM/ACC)** |
| Izie | Iza | Noun | Gender: Feminine, Number: Singular, Case: Dative |
| wczoraj | wczoraj | Adverb | Temporal: Yesterday |

**Note:** "jabłko" is ambiguous - it could be Nominative or Accusative. This will be resolved by Deduction.

**Output:**
```rust
vec![
    MorphAnalysis {
        lemma: "Tomek",
        pos: PartOfSpeech::Noun,
        features: FeatureBundle {
            gender: Some(Gender::Masculine),
            number: Some(Number::Singular),
            case: Some(Case::Nominative),
            animacy: Some(Animacy::Animate),
            ..Default::default()
        },
    },
    MorphAnalysis {
        lemma: "dać",
        pos: PartOfSpeech::Verb,
        features: FeatureBundle {
            tense: Some(Tense::Past),
            aspect: Some(Aspect::Perfective),
            person: Some(Person::Third),
            number: Some(Number::Singular),
            gender: Some(Gender::Masculine),
            ..Default::default()
        },
    },
    MorphAnalysis {
        lemma: "jabłko",
        pos: PartOfSpeech::Noun,
        features: FeatureBundle {
            gender: Some(Gender::Neuter),
            number: Some(Number::Singular),
            case: None, // Ambiguous - will be resolved
            ..Default::default()
        },
    },
    MorphAnalysis {
        lemma: "Iza",
        pos: PartOfSpeech::Noun,
        features: FeatureBundle {
            gender: Some(Gender::Feminine),
            number: Some(Number::Singular),
            case: Some(Case::Dative),
            animacy: Some(Animacy::Animate),
            ..Default::default()
        },
    },
    MorphAnalysis {
        lemma: "wczoraj",
        pos: PartOfSpeech::Adverb,
        features: FeatureBundle {
            temporal: Some(TemporalReference::Relative {
                offset: Duration::days(1),
                anchor: TemporalAnchor::Now,
            }),
            ..Default::default()
        },
    },
]
```

---

## Phase 3: Partial Structure Building

Build a partial syntactic structure before deduction.

**Output:**
```rust
Utterance {
    sentences: vec![Sentence {
        frames: vec![], // Empty - will be filled by Deduction
        temporal: Some(TemporalReference::Relative {
            offset: Duration::days(1),
            anchor: TemporalAnchor::Now,
        }),
        tense: Some(Tense::Past),
        aspect: Some(Aspect::Perfective),
        polarity: Polarity::Positive,
        // ... other fields
    }],
}
```

---

## Phase 4: Deduction Engine

This is where the magic happens. The Deduction Engine resolves ambiguities and fills in missing information.

### Step 4.1: Verb Frame Matching

**Input:** Verb "dać" (give)

**Lookup in Ontology:**
```rust
Frame::Transfer {
    required_roles: vec![
        SemanticRole::Agent,
        SemanticRole::Recipient,
        SemanticRole::Theme,
    ],
}
```

**Output:** We know we need to find Agent, Recipient, and Theme.

### Step 4.2: Case and Role Resolution

**Problem:** We have 4 nouns/adverbs and need to assign 3 roles.

**Available entities:**
1. Tomek (Nominative, Masculine, Animate)
2. dał (Verb - not an entity)
3. jabłko (Case ambiguous: NOM/ACC, Neuter, Inanimate)
4. Izie (Dative, Feminine, Animate)
5. wczoraj (Adverb - temporal modifier)

**Resolution Strategy:**

1. **Explicit case wins:**
   - Izie → Dative → Recipient ✓

2. **Verb frame + position:**
   - "dać" requires: Agent (NOM), Recipient (DAT), Theme (ACC)
   - Tomek is Nominative → Agent ✓
   - jabłko must be Accusative → Theme ✓

3. **Ontology constraints:**
   - Agent must be Animate → Tomek is Animate ✓
   - Recipient must be Animate → Izie is Animate ✓
   - Theme can be anything → jabłko is Inanimate ✓

**Output:**
```rust
Frame::Transfer {
    agent: Entity {
        concept: ConceptId("person"),
        name: Some("Tomek"),
        features: FeatureBundle {
            gender: Some(Gender::Masculine),
            number: Some(Number::Singular),
            case: Some(Case::Nominative),
            animacy: Some(Animacy::Animate),
            ..Default::default()
        },
    },
    recipient: Entity {
        concept: ConceptId("person"),
        name: Some("Iza"),
        features: FeatureBundle {
            gender: Some(Gender::Feminine),
            number: Some(Number::Singular),
            case: Some(Case::Dative),
            animacy: Some(Animacy::Animate),
            ..Default::default()
        },
    },
    theme: Entity {
        concept: ConceptId("apple"),
        name: None,
        features: FeatureBundle {
            gender: Some(Gender::Neuter),
            number: Some(Number::Singular),
            case: Some(Case::Accusative), // Resolved!
            ..Default::default()
        },
    },
}
```

### Step 4.3: Pronoun Resolution

**No pronouns in this sentence** - skip this step.

### Step 4.4: Temporal Anchoring

**Input:** "wczoraj" (yesterday)

**Resolution:**
```rust
TemporalReference::Relative {
    offset: Duration::days(1),
    anchor: TemporalAnchor::Now,
}
```

**Output:** Sentence temporal field is set.

### Step 4.5: Ontology Validation

**Check:** Does this frame make sense?

- Transfer frame with Animate Agent ✓
- Animate Recipient ✓
- Inanimate Theme ✓

**Result:** Valid!

### Step 4.6: Feature Normalization

**Normalize aspect and tense:**
- Aspect: Perfective ✓
- Tense: Past ✓
- Polarity: Positive ✓

**Final Deduction Output:**
```rust
Utterance {
    sentences: vec![Sentence {
        frames: vec![Frame::Transfer {
            agent: Entity {
                concept: ConceptId("person"),
                name: Some("Tomek"),
                features: FeatureBundle {
                    gender: Some(Gender::Masculine),
                    number: Some(Number::Singular),
                    case: Some(Case::Nominative),
                    animacy: Some(Animacy::Animate),
                    ..Default::default()
                },
            },
            recipient: Entity {
                concept: ConceptId("person"),
                name: Some("Iza"),
                features: FeatureBundle {
                    gender: Some(Gender::Feminine),
                    number: Some(Number::Singular),
                    case: Some(Case::Dative),
                    animacy: Some(Animacy::Animate),
                    ..Default::default()
                },
            },
            theme: Entity {
                concept: ConceptId("apple"),
                name: None,
                features: FeatureBundle {
                    gender: Some(Gender::Neuter),
                    number: Some(Number::Singular),
                    case: Some(Case::Accusative),
                    ..Default::default()
                },
            },
        }],
        temporal: Some(TemporalReference::Relative {
            offset: Duration::days(1),
            anchor: TemporalAnchor::Now,
        }),
        tense: Some(Tense::Past),
        aspect: Some(Aspect::Perfective),
        polarity: Polarity::Positive,
        illocution: Illocution::Statement,
    }],
}
```

---

## Phase 5: InterlinguaNode

The result of Deduction is a complete `InterlinguaNode`.

```rust
InterlinguaNode::Natural(Utterance {
    sentences: vec![Sentence {
        frames: vec![Frame::Transfer { /* ... */ }],
        temporal: Some(TemporalReference::Relative {
            offset: Duration::days(1),
            anchor: TemporalAnchor::Now,
        }),
        tense: Some(Tense::Past),
        aspect: Some(Aspect::Perfective),
        polarity: Polarity::Positive,
        illocution: Illocution::Statement,
    }],
})
```

This is the **language-neutral representation** that can be used to generate text in any language.

---

## Phase 6: Generation (Polish → English)

Now we generate English text from the InterlinguaNode.

### Step 6.1: Frame Analysis

**Extract roles:**
- Agent: Tomek (person, masculine, singular)
- Recipient: Iza (person, feminine, singular)
- Theme: apple (neuter, singular)

### Step 6.2: Lexical Selection

**Choose English words:**

| Concept | English Lemma |
|---------|---------------|
| person (Tomek) | "Tomek" (proper name) |
| person (Iza) | "Iza" (proper name) |
| apple | "apple" |
| give | "give" |
| yesterday | "yesterday" |

### Step 6.3: Morphological Inflection

**Apply English morphology:**

**Verb "give":**
- Tense: Past → "gave"

**Noun "apple":**
- Number: Singular → "apple"
- Definiteness: Indefinite (first mention) → "an apple"

**Proper names:**
- "Tomek" → "Tomek"
- "Iza" → "Iza"

### Step 6.4: Word Order Determination

**English word order for Transfer frame:**

**Option 1:** Subject + Verb + Indirect Object + Direct Object
- "Tomek gave Iza an apple"

**Option 2:** Subject + Verb + Direct Object + Prepositional Phrase
- "Tomek gave an apple to Iza"

**Decision:** Use Option 2 (more explicit for non-native names)

**Temporal modifier:**
- "yesterday" goes at the end

**Final order:**
- "Tomek gave an apple to Iza yesterday"

### Step 6.5: Surface Realization

**Assemble final sentence:**

```rust
"Tomek gave an apple to Iza yesterday"
```

---

## Final Output

**Input (Polish):** "Tomek dał jabłko Izie wczoraj"

**InterlinguaNode:** (shown in Phase 5)

**Output (English):** "Tomek gave an apple to Iza yesterday"

---

## Summary of Key Decisions

### Polish Parser Decisions

1. **Case ambiguity resolution:** "jabłko" resolved to Accusative (not Nominative) based on verb frame requirements
2. **Role assignment:** Used explicit case + verb frame + ontology constraints
3. **Temporal anchoring:** "wczoraj" converted to relative temporal reference

### English Generator Decisions

1. **Article selection:** "an apple" (indefinite, first mention)
2. **Word order:** Subject + Verb + Direct Object + Prepositional Phrase
3. **Temporal modifier placement:** At the end of the sentence
4. **Verb inflection:** "give" → "gave" (past tense)

---

## What Could Go Wrong?

### Potential Failure Points

1. **Morphological analysis error:**
   - If "jabłko" was incorrectly tagged as only Nominative, deduction would fail
   - **Recovery:** Fallback to verb frame requirements

2. **Deduction error:**
   - If ontology says Theme must be Animate, "jabłko" would fail
   - **Recovery:** Check ontology rules, fix if incorrect

3. **Generation error:**
   - If English lexicon doesn't have "apple", generation fails
   - **Recovery:** Add missing lexical entry

4. **Word order error:**
   - If generator chooses wrong order, sentence is ungrammatical
   - **Recovery:** Use LanguageDescriptor rules

---

## Variations

### Variation 1: Negative Sentence

**Input:** "Tomek nie dał jabłka Izie"

**Key difference:**
- Polarity: Negative
- Theme case: Genitive (not Accusative) - Polish requires Genitive in negative sentences

**Output:** "Tomek didn't give an apple to Iza"

### Variation 2: Question

**Input:** "Czy Tomek dał jabłko Izie?"

**Key difference:**
- Illocution: Question

**Output:** "Did Tomek give an apple to Iza?"

### Variation 3: Different Frame

**Input:** "Tomek widzi Iza" (Tomek sees Iza)

**Frame:** Perception (not Transfer)
- Experiencer: Tomek
- Stimulus: Iza

**Output:** "Tomek sees Iza"

---

## Testing This Example

```rust
#[test]
fn test_end_to_end_tomek_dal_jablko() {
    let input = "Tomek dał jabłko Izie wczoraj";
    let expected_output = "Tomek gave an apple to Iza yesterday";
    
    let result = translate(input, LanguageId::PL, LanguageId::EN);
    
    assert_eq!(result, expected_output);
}
```

---

## References

- [GENERATOR.md](./GENERATOR.md) - For generation details
- [DEDUCTION.md](./DEDUCTION.md) - For deduction details
- [INTERLINGUA.md](./INTERLINGUA.md) - For InterlinguaNode structure
- [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) - For case assignment rules
