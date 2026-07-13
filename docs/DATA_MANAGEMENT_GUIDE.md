# Data Management Guide — Working with RON Files

This document provides guidance on creating, maintaining, and validating RON data files in lexFlex. Since lexFlex is heavily **data-driven**, the quality of data files directly impacts system quality.

## Overview

lexFlex stores linguistic knowledge in RON (Rusty Object Notation) files:
- `data/concepts/concepts.ron` - Master concept definitions
- `data/lexicons/<lang>/lexicon.ron` - Per-language word entries
- `data/morphology/<lang>/*.ron` - Morphological paradigm rules
- `data/ontology/ontology.ron` - Concept hierarchies and type constraints
- `data/descriptors/<lang>.ron` - Language descriptors

**Key principle:** Data files are the **source of truth** for linguistic knowledge. Code should be minimal and generic; data should be rich and specific.

**New (on-the-fly):** The integrated `lexflex-learner` (with optional local LLM first) automatically appends proposals for unknown words directly to `data/concepts/concepts.ron` (generic Interlingua) and `data/lexicons/{pl,en}/lexicon.ron` during normal parsing/translation (via `resolve_concept_for_unknown` in the pipeline). This builds the DB recursively for base lemmas too. Review generated entries before committing; use `LEXFLEX_LEARN_PROPOSALS_DIR` or the benchmark script for curated bulk runs.

## Recommended Order of Data Creation

### Phase 1: Core Concepts (Week 1-2)

**Start with:** `data/concepts/concepts.ron`

**Why first:**
- Concepts are language-independent
- All other data references concepts
- Defines the semantic foundation

**What to include:**
- 50-80 core concepts for MVP
- Focus on most common verbs and nouns
- Define frame types for verbs

**Example:**
```ron
[
    Concept(
        id: "PERSON",
        category: Entity,
        inherent_features: FeatureBundle(
            animacy: Some(Animate),
        ),
        playable_roles: [Agent, Recipient, Experiencer],
    ),
    Concept(
        id: "GIVE",
        category: Action,
        frame_type: Some("Transfer"),
        roles: [Agent, Recipient, Theme],
    ),
    // ... more concepts
]
```

**Validation:**
- All concepts have unique IDs
- Frame types are consistent
- Roles are valid semantic roles

### Phase 2: Lexicon (Week 2-3)

**Next:** `data/lexicons/pl/lexicon.ron` and `data/lexicons/en/lexicon.ron`

**Why second:**
- Lexicon maps words to concepts
- Requires concepts to exist
- Tests concept definitions

**What to include:**
- 80-100 words per language for MVP
- Focus on words needed for test sentences
- Include morphological features

**Example:**
```ron
Lexicon(
    entries: {
        "Tomek": LexEntry(
            lemma: "Tomek",
            pos: Noun,
            concept: "PERSON",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                case: Some(Nominative),
            ),
        ),
        "dać": LexEntry(
            lemma: "dać",
            pos: Verb,
            concept: "GIVE",
            features: FeatureBundle(
                aspect: Some(Perfective),
            ),
        ),
        // ... more entries
    },
)
```

**Validation:**
- All concepts referenced in lexicon exist in concepts.ron
- All words have valid POS tags
- Morphological features are consistent

### Phase 3: Morphology (Week 3-4)

**Next:** `data/morphology/pl/*.ron` and `data/morphology/en/*.ron`

**Why third:**
- Morphology rules are used by generator
- Requires lexicon to test inflection
- Complex to implement correctly

**What to include:**
- 4 noun paradigms for Polish
- 2 verb paradigms for Polish
- Basic paradigms for English

**Example:**
```ron
MorphParadigm(
    name: "neuter_o",
    applies_to: Noun,
    rules: [
        MorphRule(
            conditions: [CaseIs(Nominative), NumberIs(Singular)],
            operations: [],  // jabłko → jabłko
        ),
        MorphRule(
            conditions: [CaseIs(Genitive), NumberIs(Singular)],
            operations: [ReplaceSuffix { from: "o", to: "a" }],  // jabłko → jabłka
        ),
        // ... more rules
    ],
)
```

**Validation:**
- All paradigms handle required cases/tenses
- Rules don't conflict
- Test inflection with sample words

### Phase 4: Ontology (Week 4-5)

**Next:** `data/ontology/ontology.ron`

**Why fourth:**
- Ontology validates semantic types
- Requires concepts and lexicon
- Used by deduction engine

**What to include:**
- IS_A hierarchy for concepts
- Type constraints for frames
- Feature inheritance rules

**Example:**
```ron
Ontology(
    is_a: {
        "APPLE": ["FRUIT"],
        "FRUIT": ["FOOD"],
        "FOOD": ["PHYSICAL_OBJECT"],
    },
    type_constraints: [
        TypeConstraint(
            frame: "Transfer",
            role: Agent,
            required_type: "ANIMATE",
        ),
        // ... more constraints
    ],
)
```

**Validation:**
- No cycles in IS_A hierarchy
- All referenced concepts exist
- Type constraints are consistent

### Phase 5: Language Descriptors (Week 5-6)

**Finally:** `data/descriptors/pl.ron` and `data/descriptors/en.ron`

**Why last:**
- Descriptors drive generation
- Requires all other data to exist
- Complex to get right

**What to include:**
- Word order rules
- Case system (for Polish)
- Article system (for English)
- Pronoun strategies

**Example:**
```ron
LanguageDescriptor(
    language: "pl",
    name: "Polish",
    morphology: MorphologyDescriptor(
        morph_type: Fusional,
        has_cases: true,
        has_articles: false,
    ),
    syntax: SyntaxDescriptor(
        word_order: SVO,
        pro_drop: true,
    ),
)
```

**Validation:**
- Descriptor matches language properties
- All referenced paradigms exist
- Generation produces correct output

## Maintaining Consistency

### Cross-File References

**Problem:** Data files reference each other. Changes in one file may break others.

**Solution:** Use validation tests to check consistency.

```rust
#[test]
fn test_concept_consistency() {
    let concepts = load_concepts("data/concepts/concepts.ron");
    let pl_lexicon = load_lexicon("data/lexicons/pl/lexicon.ron");
    
    // Check that all concepts in lexicon exist in concepts.ron
    for entry in pl_lexicon.entries.values() {
        assert!(
            concepts.iter().any(|c| c.id == entry.concept),
            "Concept '{}' not found in concepts.ron",
            entry.concept
        );
    }
}

#[test]
fn test_paradigm_consistency() {
    let pl_lexicon = load_lexicon("data/lexicons/pl/lexicon.ron");
    let pl_morphology = load_morphology("data/morphology/pl/");
    
    // Check that all paradigms referenced in lexicon exist
    for entry in pl_lexicon.entries.values() {
        if let Some(paradigm) = &entry.paradigm {
            assert!(
                pl_morphology.paradigms.iter().any(|p| p.name == *paradigm),
                "Paradigm '{}' not found in morphology",
                paradigm
            );
        }
    }
}
```

### Versioning Data

**Problem:** Data evolves over time. Need to track changes.

**Solution:** Use git to version data files. Tag releases.

```bash
# Tag a release
git tag -a v0.1.0 -m "MVP data release"
git push origin v0.1.0

# View changes
git log --oneline data/concepts/concepts.ron
```

**Best practices:**
- Commit data changes separately from code changes
- Write meaningful commit messages
- Tag major releases

### Adding New Frames

**Problem:** Adding new frame types requires updating multiple files.

**Solution:** Follow this checklist:

1. **Add frame to concepts.ron:**
   ```ron
   Concept(
       id: "CREATE",
       category: Action,
       frame_type: Some("Creation"),
       roles: [Agent, Theme, Instrument],
   ),
   ```

2. **Add verbs to lexicon:**
   ```ron
   "tworzyć": LexEntry(
       lemma: "tworzyć",
       pos: Verb,
       concept: "CREATE",
       features: FeatureBundle(
           aspect: Some(Imperfective),
       ),
   ),
   ```

3. **Add type constraints to ontology:**
   ```ron
   TypeConstraint(
       frame: "Creation",
       role: Agent,
       required_type: "ANIMATE",
   ),
   ```

4. **Implement generator logic:**
   ```rust
   fn generate_creation(frame: &Frame::Creation, ...) -> Result<Vec<String>, GenerateError> {
       // Implement generation for Creation frame
   }
   ```

5. **Add tests:**
   ```rust
   #[test]
   fn test_creation_frame() {
       let il = create_creation_interlingua("Tomek", "rzeźba", "dłuto");
       let result = generator.generate(&il);
       assert!(result.contains("tworzy"));
   }
   ```

## Common Data Errors

### Error 1: Missing Concept Reference

**Symptom:**
```
Error: Concept 'UNKNOWN_CONCEPT' not found in concepts.ron
```

**Cause:** Lexicon references a concept that doesn't exist.

**Fix:** Add the missing concept to `concepts.ron` or fix the reference in lexicon.

### Error 2: Invalid Paradigm Reference

**Symptom:**
```
Error: Paradigm 'unknown_paradigm' not found in morphology
```

**Cause:** Lexicon references a paradigm that doesn't exist.

**Fix:** Add the missing paradigm to morphology or fix the reference in lexicon.

### Error 3: Conflicting Morphological Rules

**Symptom:**
```
Error: Multiple rules match for case=Accusative, number=Singular
```

**Cause:** Two or more rules have overlapping conditions.

**Fix:** Make conditions more specific or remove conflicting rules.

### Error 4: Cycle in IS_A Hierarchy

**Symptom:**
```
Error: Cycle detected in IS_A hierarchy: A → B → C → A
```

**Cause:** Ontology has circular references.

**Fix:** Remove the cycle by restructuring the hierarchy.

### Error 5: Inconsistent Features

**Symptom:**
```
Error: Entity has gender=Masculine but concept=APPLE (which is Neuter)
```

**Cause:** Lexicon entry has features that conflict with concept's inherent features.

**Fix:** Fix the features in lexicon or update concept's inherent features.

## Data Validation Tests

### Comprehensive Validation

```rust
#[test]
fn test_all_data_consistency() {
    // Load all data
    let concepts = load_concepts("data/concepts/concepts.ron");
    let pl_lexicon = load_lexicon("data/lexicons/pl/lexicon.ron");
    let en_lexicon = load_lexicon("data/lexicons/en/lexicon.ron");
    let pl_morphology = load_morphology("data/morphology/pl/");
    let en_morphology = load_morphology("data/morphology/en/");
    let ontology = load_ontology("data/ontology/ontology.ron");
    
    // Validate concepts
    validate_concepts(&concepts);
    
    // Validate lexicons
    validate_lexicon(&pl_lexicon, &concepts);
    validate_lexicon(&en_lexicon, &concepts);
    
    // Validate morphology
    validate_morphology(&pl_morphology);
    validate_morphology(&en_morphology);
    
    // Validate ontology
    validate_ontology(&ontology, &concepts);
    
    // Cross-validate
    validate_cross_references(&concepts, &pl_lexicon, &pl_morphology);
    validate_cross_references(&concepts, &en_lexicon, &en_morphology);
}
```

### Data Quality Metrics

```rust
fn measure_data_quality() -> DataQualityReport {
    let concepts = load_concepts("data/concepts/concepts.ron");
    let pl_lexicon = load_lexicon("data/lexicons/pl/lexicon.ron");
    
    DataQualityReport {
        concept_count: concepts.len(),
        pl_word_count: pl_lexicon.entries.len(),
        en_word_count: load_lexicon("data/lexicons/en/lexicon.ron").entries.len(),
        paradigm_count: load_morphology("data/morphology/pl/").paradigms.len(),
        consistency_score: calculate_consistency_score(),
        coverage_score: calculate_coverage_score(),
    }
}
```

## Best Practices

### 1. Start Small, Expand Gradually

**Don't:**
```ron
// Don't try to add 500 concepts at once!
[
    Concept { id: "PERSON", ... },
    Concept { id: "APPLE", ... },
    // ... 498 more concepts
]
```

**Do:**
```ron
// Start with 50-80 core concepts
[
    Concept { id: "PERSON", ... },
    Concept { id: "GIVE", ... },
    Concept { id: "APPLE", ... },
    // ... 47-77 more concepts
]
```

**Why:** Easier to validate, test, and maintain.

### 2. Use Meaningful IDs

**Don't:**
```ron
Concept { id: "C001", ... }  // What is C001?
```

**Do:**
```ron
Concept { id: "PERSON", ... }  // Clear and meaningful
```

**Why:** Easier to understand and debug.

### 3. Document Complex Rules

**Don't:**
```ron
MorphRule(
    conditions: [CaseIs(Accusative), NumberIs(Singular), GenderIs(Masculine), AnimacyIs(Animate)],
    operations: [ReplaceSuffix { from: "y", to: "ego" }],
)
```

**Do:**
```ron
// Polish: masculine animate nouns in accusative singular
// Example: "dobry kot" → "dobrego kota"
MorphRule(
    conditions: [CaseIs(Accusative), NumberIs(Singular), GenderIs(Masculine), AnimacyIs(Animate)],
    operations: [ReplaceSuffix { from: "y", to: "ego" }],
)
```

**Why:** Helps others understand the rule.

### 4. Test Data Independently

**Don't:**
```rust
// Don't only test data through full pipeline
#[test]
fn test_translation() {
    let result = translate("Tomek dał jabłko", PL, EN);
    assert_eq!(result, "Tomek gave an apple");
}
```

**Do:**
```rust
// Test data independently
#[test]
fn test_concept_loading() {
    let concepts = load_concepts("data/concepts/concepts.ron");
    assert!(concepts.iter().any(|c| c.id == "PERSON"));
}

#[test]
fn test_morphology_inflection() {
    let morphology = load_morphology("data/morphology/pl/");
    let result = morphology.inflect("jabłko", Accusative, Singular, Neuter);
    assert_eq!(result, "jabłko");
}
```

**Why:** Easier to isolate data errors.

### 5. Keep Data and Code Separate

**Don't:**
```rust
// Don't hardcode linguistic knowledge in code!
fn generate(il: &InterlinguaNode) -> String {
    if il.frames[0].theme.concept == "apple" {
        "jabłko"  // Hardcoded!
    } else {
        // ...
    }
}
```

**Do:**
```rust
// Use data files for linguistic knowledge
fn generate(il: &InterlinguaNode, lexicon: &Lexicon) -> String {
    let lemma = lexicon.lookup(&il.frames[0].theme.concept)?;
    // ...
}
```

**Why:** Data-driven approach is more maintainable.

## Tools and Utilities

### Data Loader

```rust
pub struct DataLoader {
    base_dir: PathBuf,
}

impl DataLoader {
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }
    
    pub fn load_concepts(&self) -> Result<Vec<Concept>, DataError> {
        let path = self.base_dir.join("concepts/concepts.ron");
        let content = fs::read_to_string(path)?;
        let concepts: Vec<Concept> = ron::from_str(&content)?;
        Ok(concepts)
    }
    
    pub fn load_lexicon(&self, language: &str) -> Result<Lexicon, DataError> {
        let path = self.base_dir.join(format!("lexicons/{}/lexicon.ron", language));
        let content = fs::read_to_string(path)?;
        let lexicon: Lexicon = ron::from_str(&content)?;
        Ok(lexicon)
    }
    
    // ... more loaders
}
```

### Data Validator

```rust
pub struct DataValidator {
    concepts: Vec<Concept>,
    lexicons: HashMap<String, Lexicon>,
    morphology: HashMap<String, MorphologyEngine>,
    ontology: Ontology,
}

impl DataValidator {
    pub fn validate_all(&self) -> Result<(), Vec<DataError>> {
        let mut errors = Vec::new();
        
        // Validate concepts
        if let Err(e) = self.validate_concepts() {
            errors.push(e);
        }
        
        // Validate lexicons
        for (lang, lexicon) in &self.lexicons {
            if let Err(e) = self.validate_lexicon(lang, lexicon) {
                errors.push(e);
            }
        }
        
        // Validate morphology
        for (lang, morph) in &self.morphology {
            if let Err(e) = self.validate_morphology(lang, morph) {
                errors.push(e);
            }
        }
        
        // Validate ontology
        if let Err(e) = self.validate_ontology() {
            errors.push(e);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Summary

Data management in lexFlex:

1. **Follow the recommended order:** Concepts → Lexicon → Morphology → Ontology → Descriptors
2. **Maintain consistency:** Use validation tests to check cross-references
3. **Version data:** Use git to track changes
4. **Start small:** Add 50-80 concepts, not 500
5. **Test independently:** Validate data without full pipeline
6. **Keep data and code separate:** Use data-driven approach

**Key principle:** Data is the source of truth. Code should be minimal and generic; data should be rich and specific.

## References

- [DATA_SAMPLES.md](./DATA_SAMPLES.md) - Example RON files
- [LEXICON.md](./LEXICON.md) - Lexicon structure
- [MORPHOLOGY.md](./MORPHOLOGY.md) - Morphology rules
- [ONTOLOGY.md](./ONTOLOGY.md) - Ontology structure
