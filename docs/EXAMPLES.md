# Examples - End-to-End Translation Examples

This document shows complete examples of lexFlex in action, from simple translations to complex scenarios.

---

## Example 1: Simple SVO Translation

### Input (Polish)
```
Tomek dał jabłko Izie
```

### Parsing Pipeline

**Step 1: Tokenization**
```
["Tomek", "dał", "jabłko", "Izie"]
```

**Step 2: Morphological Analysis**
```
Tomek  → Lemma: "Tomek",  POS: Noun, Case: NOM, Gender: M, Number: SG
dał    → Lemma: "dać",    POS: Verb, Tense: PAST, Person: 3, Number: SG
jabłko → Lemma: "jabłko", POS: Noun, Case: NOM/ACC, Gender: N, Number: SG
Izie   → Lemma: "Iza",    POS: Noun, Case: DAT, Gender: F, Number: SG
```

**Step 3: Lexicon Lookup**
```
Tomek  → Concept: PERSON, Name: "Tomek"
dać    → Concept: GIVE, Frame: Transfer
jabłko → Concept: APPLE
Iza    → Concept: PERSON, Name: "Iza"
```

**Step 4: Frame Assignment**
```
Frame::Transfer {
    agent: Entity { concept: PERSON, name: "Tomek", gender: M, number: SG },
    recipient: Entity { concept: PERSON, name: "Iza", gender: F, number: SG },
    theme: Entity { concept: APPLE, gender: N, number: SG },
}
```

**Step 5: Deduction**
```
- Verb "dać" requires [agent:NOM, recipient:DAT, theme:ACC]
- "jabłko" is ambiguous (NOM or ACC)
- Subcategorization resolves: "jabłko" = ACC (theme), not NOM (agent)
- Tense: PAST, Aspect: PERFECTIVE
- Polarity: POSITIVE
- Illocution: STATEMENT
```

**Interlingua Output**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity {
                        concept: "PERSON",
                        name: Some("Tomek"),
                        features: FeatureBundle {
                            gender: Some(Masculine),
                            number: Some(Singular),
                            animacy: Some(Animate),
                            ..Default::default()
                        },
                    },
                    recipient: Entity {
                        concept: "PERSON",
                        name: Some("Iza"),
                        features: FeatureBundle {
                            gender: Some(Feminine),
                            number: Some(Singular),
                            animacy: Some(Animate),
                            ..Default::default()
                        },
                    },
                    theme: Entity {
                        concept: "APPLE",
                        features: FeatureBundle {
                            gender: Some(Neuter),
                            number: Some(Singular),
                            countability: Some(Count),
                            ..Default::default()
                        },
                    },
                }
            ],
            tense: Tense::Past,
            aspect: Aspect::Perfective,
            polarity: Polarity::Positive,
            illocution: Illocution::Statement,
        }
    ],
}
```

### Generation Pipeline (English)

**Step 1: Lexeme Selection**
```
PERSON (Tomek) → "Tomek"
GIVE + PAST + PERFECTIVE → "gave"
APPLE → "apple"
PERSON (Iza) → "Iza"
```

**Step 2: Case/Position Assignment**
```
agent → Subject position
recipient → Indirect object
theme → Direct object
```

**Step 3: Inflection**
```
"gave" (past tense of "give")
"apple" (singular)
```

**Step 4: Word Order**
```
SVO: Subject + Verb + IndirectObject + DirectObject
```

**Step 5: Articles**
```
EN requires articles for countable nouns
"apple" → "an apple" (indefinite, starts with vowel)
```

**Step 6: Surface Realization**
```
"Tomek gave Iza an apple"
```

### Output (English)
```
Tomek gave Iza an apple
```

---

## Example 2: Cat Drinks Milk

### Input (Polish)
```
Kot pije mleko
```

### Parsing

**Morphological Analysis**
```
Kot   → Lemma: "kot",   POS: Noun, Case: NOM, Gender: M, Number: SG
pije  → Lemma: "pić",   POS: Verb, Tense: PRES, Person: 3, Number: SG
mleko → Lemma: "mleko", POS: Noun, Case: ACC, Gender: N, Number: SG
```

**Frame Assignment**
```
Frame::Consumption {
    agent: Entity { concept: CAT, gender: M, number: SG },
    patient: Entity { concept: MILK, gender: N, number: SG },
}
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Consumption {
                    agent: Entity {
                        concept: "CAT",
                        features: FeatureBundle {
                            gender: Some(Masculine),
                            number: Some(Singular),
                            animacy: Some(Animate),
                            ..Default::default()
                        },
                    },
                    patient: Entity {
                        concept: "MILK",
                        features: FeatureBundle {
                            gender: Some(Neuter),
                            number: Some(Singular),
                            countability: Some(Mass),
                            ..Default::default()
                        },
                    },
                }
            ],
            tense: Tense::Present,
            aspect: Aspect::Imperfective,
            polarity: Polarity::Positive,
            illocution: Illocution::Statement,
        }
    ],
}
```

### Generation (English)

**Lexeme Selection**
```
CAT → "cat"
DRINK + PRESENT + IMPERFECTIVE → "drinks" (3sg present)
MILK → "milk"
```

**Articles**
```
"cat" → "The cat" (definite, specific cat)
"milk" → "milk" (mass noun, no article needed)
```

**Output**
```
The cat drinks milk
```

---

## Example 3: Negation

### Input (Polish)
```
Tomek nie dał jabłka Izie
```

### Parsing

**Morphological Analysis**
```
Tomek   → Lemma: "Tomek",   POS: Noun, Case: NOM
nie     → Lemma: "nie",     POS: Particle (negation)
dał     → Lemma: "dać",     POS: Verb, Tense: PAST
jabłka  → Lemma: "jabłko",  POS: Noun, Case: GEN (negated ACC → GEN!)
Izie    → Lemma: "Iza",     POS: Noun, Case: DAT
```

**Key Rule: Negation Case Shift**
```
In Polish, negation of transitive verb changes ACC → GEN
"jabłko" (ACC) → "jabłka" (GEN)
```

**Frame Assignment**
```
Frame::Transfer {
    agent: Entity { concept: PERSON, name: "Tomek" },
    recipient: Entity { concept: PERSON, name: "Iza" },
    theme: Entity { concept: APPLE },
}
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { concept: "PERSON", name: Some("Tomek"), .. },
                    recipient: Entity { concept: "PERSON", name: Some("Iza"), .. },
                    theme: Entity { concept: "APPLE", .. },
                }
            ],
            tense: Tense::Past,
            aspect: Aspect::Perfective,
            polarity: Polarity::Negative,  // ← NEGATION
            illocution: Illocution::Statement,
        }
    ],
}
```

### Generation (English)

**Negation Handling**
```
EN: Add auxiliary "did not" + base verb
"dał" (past) → "did not give"
```

**Output**
```
Tomek did not give Iza an apple
```

Or more naturally:
```
Tomek didn't give Iza an apple
```

---

## Example 4: Question

### Input (Polish)
```
Czy Tomek dał jabłko Izie?
```

### Parsing

**Morphological Analysis**
```
Czy    → Question particle
Tomek  → Noun, NOM
dał    → Verb, PAST
jabłko → Noun, ACC
Izie   → Noun, DAT
?      → Punctuation
```

**Illocution Detection**
```
"Czy" at sentence start → YES/NO QUESTION
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { concept: "PERSON", name: Some("Tomek"), .. },
                    recipient: Entity { concept: "PERSON", name: Some("Iza"), .. },
                    theme: Entity { concept: "APPLE", .. },
                }
            ],
            tense: Tense::Past,
            aspect: Aspect::Perfective,
            polarity: Polarity::Positive,
            illocution: Illocution::Question,  // ← QUESTION
        }
    ],
}
```

### Generation (English)

**Question Formation**
```
EN: Use do-support for past tense questions
"dał" → "Did Tomek give"
```

**Output**
```
Did Tomek give Iza an apple?
```

---

## Example 5: Temporal Reference

### Input (Polish)
```
Wczoraj Tomek kupił książkę
```

### Parsing

**Morphological Analysis**
```
Wczoraj  → Temporal adverb: "yesterday"
Tomek    → Noun, NOM
kupił    → Verb, PAST, PERFECTIVE
książkę  → Noun, ACC (feminine)
```

**Temporal Resolution**
```
"wczoraj" → TemporalReference::Deictic {
    word: "wczoraj",
    resolved: Some(Timestamp { year: 2024, month: 1, day: 14 })
}
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { concept: "PERSON", name: Some("Tomek"), .. },
                    theme: Entity { concept: "BOOK", .. },
                    // implicit: recipient = store/seller
                }
            ],
            tense: Tense::Past,
            aspect: Aspect::Perfective,
            polarity: Polarity::Positive,
            illocution: Illocution::Statement,
            temporal: Some(TemporalReference::Deictic {
                word: "wczoraj",
                resolved: Some(Timestamp { year: 2024, month: 1, day: 14 }),
            }),
        }
    ],
}
```

### Generation (English)

**Temporal Expression**
```
"wczoraj" → "Yesterday"
```

**Output**
```
Yesterday Tomek bought a book
```

Or more naturally:
```
Tomek bought a book yesterday
```

---

## Example 6: Quantification

### Input (Polish)
```
Wszyscy studenci zdali egzamin
```

### Parsing

**Morphological Analysis**
```
Wszyscy   → Quantifier: Universal (∀)
studenci  → Noun, NOM, PL
zdali     → Verb, PAST, PL
egzamin   → Noun, ACC, SG
```

**Quantifier Detection**
```
"wszyscy" → Quantifier::Universal
```

**Interlingua**
```rust
InterlinguaNode::QuantifiedStatement {
    quantifier: Quantifier::Universal,
    variable: Variable { name: "x" },
    domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
    body: Box::new(InterlinguaNode::Application {
        function: Box::new(InterlinguaNode::Variable("PASS")),
        args: vec![
            InterlinguaNode::Variable("x"),
            InterlinguaNode::Variable("exam"),
        ],
    }),
}
```

**Logical Form**
```
∀x (Student(x) → Pass(x, exam))
```

### Generation (English)

**Quantifier Translation**
```
"wszyscy studenci" → "All students"
"zdali egzamin" → "passed the exam"
```

**Output**
```
All students passed the exam
```

---

## Example 7: Pronoun Resolution (Multi-Sentence)

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

This example demonstrates pronoun resolution across multiple sentences using discourse context. This feature is planned for v0.2+.

### Input (Polish)
```
Tomek dał jabłko Izie. Ona się ucieszyła.
```

### Parsing

**Sentence 1**
```
Tomek dał jabłko Izie
→ Frame::Transfer { agent: Tomek, recipient: Iza, theme: apple }
→ Discourse: participants = [Tomek, Iza]
```

**Sentence 2**
```
Ona się ucieszyła
→ "Ona" = PRON(NOM, F, SG)
→ Reference resolution: search discourse for feminine entity
→ Found: Iza (gender: F, number: SG)
→ "Ona" resolves to "Iza"
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [Frame::Transfer { agent: Tomek, recipient: Iza, theme: apple }],
            resolved_refs: [],
        },
        Sentence {
            frames: [Frame::Emotion {
                experiencer: Entity {
                    concept: "PERSON",
                    name: Some("Iza"),  // resolved from "Ona"
                    ..
                },
                stimulus: Entity { concept: "JOY", .. },
            }],
            resolved_refs: [("Ona", EntityRef("Iza"))],
        },
    ],
}
```

### Generation (English)

**Pronoun Translation**
```
"Ona" → "She" (resolved to Iza)
"się ucieszyła" → "was happy" / "became happy"
```

**Output**
```
Tomek gave Iza an apple. She was happy.
```

---

## Example 8: Coordination

### Input (Polish)
```
Tomek dał jabłko Izie a czekoladę Tomkowi
```

### Parsing

**Sentence 1**
```
Tomek dał jabłko Izie
→ Frame::Transfer { agent: Tomek, recipient: Iza, theme: apple }
→ Coordination: First
```

**Sentence 2**
```
[czekoladę Tomkowi]
→ Implied agent: Iza (from context)
→ Frame::Transfer { agent: Iza, recipient: Tomek, theme: chocolate }
→ Coordination: Last, type: Contrast
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [Frame::Transfer { agent: Tomek, recipient: Iza, theme: apple }],
            coordination: Some(Coordination {
                conjunct_type: ConjunctionType::Contrast,
                position: CoordPosition::First,
            }),
        },
        Sentence {
            frames: [Frame::Transfer { agent: Iza, recipient: Tomek, theme: chocolate }],
            coordination: Some(Coordination {
                conjunct_type: ConjunctionType::Contrast,
                position: CoordPosition::Last,
            }),
        },
    ],
}
```

### Generation (English)

**Coordination Translation**
```
"a" → "and" / "while" / "whereas"
```

**Output**
```
Tomek gave Iza an apple, and Iza gave Tomek a chocolate
```

Or:
```
Tomek gave Iza an apple, while Iza gave Tomek a chocolate
```

---

## Example 9: Complex Sentence with Multiple Features

### Input (Polish)
```
Wczoraj wszyscy studenci nie zdali egzaminu
```

### Parsing

**Components**
```
Wczoraj     → Temporal: yesterday
wszyscy     → Quantifier: Universal (∀)
studenci    → Noun: students
nie         → Negation
zdali       → Verb: passed (past, plural)
egzaminu    → Noun: exam (GEN - negated ACC)
```

**Interlingua**
```rust
Utterance {
    sentences: [
        Sentence {
            frames: [
                Frame::Cognition {
                    cognizer: Entity { concept: "STUDENT", number: Plural },
                    content: Entity { concept: "EXAM" },
                }
            ],
            quantifier: Some(QuantifiedExpression {
                quantifier: Quantifier::Universal,
                variable: Variable { name: "x" },
                domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
                body: Box::new(InterlinguaNode::Application {
                    function: Box::new(InterlinguaNode::Variable("PASS")),
                    args: vec![
                        InterlinguaNode::Variable("x"),
                        InterlinguaNode::Variable("exam"),
                    ],
                }),
            }),
            tense: Tense::Past,
            polarity: Polarity::Negative,
            temporal: Some(TemporalReference::Deictic {
                word: "wczoraj",
                resolved: Some(Timestamp { year: 2024, month: 1, day: 14 }),
            }),
        }
    ],
}
```

**Logical Form**
```
∀x (Student(x) → ¬Pass(x, exam))
```

### Generation (English)

**Output**
```
Yesterday all students did not pass the exam
```

Or more naturally:
```
Yesterday, none of the students passed the exam
```

---

## Example 10: Error Handling

### Input (Polish)
```
Tomek dał xyz Izie
```

### Parsing

**Error Detection**
```
"xyz" → UnknownToken {
    token: "xyz",
    position: 2,
    suggestions: []  // no similar words in lexicon
}
```

**Graceful Degradation**
```rust
DegradationStrategy {
    level: DegradationLevel::Partial,
    partial_result: Some(Utterance {
        // Partial parse without "xyz"
        sentences: [Sentence {
            frames: [Frame::Transfer {
                agent: Tomek,
                recipient: Iza,
                theme: Entity { concept: "UNKNOWN", .. },
            }],
        }],
    }),
    clarification_needed: vec![
        ClarificationRequest::UnknownWord { word: "xyz" }
    ],
}
```

**Output**
```
Error: Unknown word "xyz" at position 2

Partial parse:
  Agent: Tomek
  Recipient: Iza
  Theme: [unknown]

Suggestions:
  - Did you mean "jabłko"?
  - Did you mean "książka"?
```

---

## Summary

These examples demonstrate:

1. **Simple SVO** - basic subject-verb-object sentences
2. **Morphology** - case inflection, verb conjugation
3. **Negation** - polarity changes, case shifts (ACC → GEN)
4. **Questions** - illocution detection, question formation
5. **Temporal** - time references, deictic resolution
6. **Quantification** - universal/existential quantifiers
7. **Pronouns** - reference resolution, coreference
8. **Coordination** - multiple clauses, conjunctions
9. **Complex sentences** - multiple features combined
10. **Error handling** - graceful degradation, suggestions

Each example shows the complete pipeline: input → parsing → Interlingua → generation → output.
