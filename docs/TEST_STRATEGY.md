# Test Strategy - How to Test lexFlex

This document describes the testing approach for lexFlex, covering unit tests, integration tests, property-based tests, and golden tests.

---

## Testing Philosophy

1. **Test at multiple levels** - unit, integration, e2e
2. **Use snapshot testing** - catch regressions in output
3. **Property-based testing** - find edge cases automatically
4. **Golden tests** - ensure known inputs produce known outputs
5. **Test data separately** - validate RON files before loading

---

## Unit Tests

Test individual components in isolation.

### Morphology Tests

```rust
// src/engines/polish/morphology.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noun_declension_jabłko() {
        let morphology = PolishMorphology::new(/* ... */);
        
        // Nominative singular
        assert_eq!(
            morphology.inflect_noun("jabłko", Case::Nominative, Number::Singular, Gender::Neuter),
            "jabłko"
        );
        
        // Genitive singular
        assert_eq!(
            morphology.inflect_noun("jabłko", Case::Genitive, Number::Singular, Gender::Neuter),
            "jabłka"
        );
        
        // Dative singular
        assert_eq!(
            morphology.inflect_noun("jabłko", Case::Dative, Number::Singular, Gender::Neuter),
            "jabłku"
        );
        
        // Accusative singular (neuter: ACC = NOM)
        assert_eq!(
            morphology.inflect_noun("jabłko", Case::Accusative, Number::Singular, Gender::Neuter),
            "jabłko"
        );
    }

    #[test]
    fn test_noun_declension_książka() {
        let morphology = PolishMorphology::new(/* ... */);
        
        assert_eq!(
            morphology.inflect_noun("książka", Case::Nominative, Number::Singular, Gender::Feminine),
            "książka"
        );
        
        assert_eq!(
            morphology.inflect_noun("książka", Case::Accusative, Number::Singular, Gender::Feminine),
            "książkę"
        );
        
        assert_eq!(
            morphology.inflect_noun("książka", Case::Genitive, Number::Singular, Gender::Feminine),
            "książki"
        );
    }

    #[test]
    fn test_verb_conjugation_czytać() {
        let morphology = PolishMorphology::new(/* ... */);
        
        assert_eq!(
            morphology.inflect_verb("czytać", Tense::Present, Person::First, Number::Singular),
            "czytam"
        );
        
        assert_eq!(
            morphology.inflect_verb("czytać", Tense::Present, Person::Second, Number::Singular),
            "czytasz"
        );
        
        assert_eq!(
            morphology.inflect_verb("czytać", Tense::Present, Person::Third, Number::Singular),
            "czyta"
        );
    }
}
```

### Parser Tests

```rust
// src/engines/polish/parser.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svo() {
        let lexicon = Lexicon::new(/* ... */);
        let parser = PolishParser::new(lexicon);
        
        let result = parser.parse("Tomek dał jabłko Izie");
        
        assert!(result.is_ok());
        let utterance = result.unwrap();
        
        assert_eq!(utterance.sentences.len(), 1);
        let sentence = &utterance.sentences[0];
        
        assert_eq!(sentence.frames.len(), 1);
        match &sentence.frames[0] {
            Frame::Transfer { agent, recipient, theme } => {
                assert_eq!(agent.concept, "PERSON");
                assert_eq!(agent.name, Some("Tomek".to_string()));
                assert_eq!(recipient.concept, "PERSON");
                assert_eq!(recipient.name, Some("Iza".to_string()));
                assert_eq!(theme.concept, "APPLE");
            }
            _ => panic!("Expected Transfer frame"),
        }
    }

    #[test]
    fn test_parse_unknown_word() {
        let lexicon = Lexicon::new(/* ... */);
        let parser = PolishParser::new(lexicon);
        
        let result = parser.parse("Tomek dał xyz Izie");
        
        assert!(result.is_err());
    }
}
```

### Generator Tests

```rust
// src/engines/polish/generator.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_transfer() {
        let morphology = PolishMorphology::new(/* ... */);
        let generator = PolishGenerator::new(morphology);
        
        let utterance = Utterance {
            sentences: vec![Sentence {
                frames: vec![Frame::Transfer {
                    agent: Entity::new("PERSON", FeatureBundle {
                        gender: Some(Gender::Masculine),
                        number: Some(Number::Singular),
                        ..Default::default()
                    }).with_name("Tomek"),
                    recipient: Entity::new("PERSON", FeatureBundle {
                        gender: Some(Gender::Feminine),
                        number: Some(Number::Singular),
                        ..Default::default()
                    }).with_name("Iza"),
                    theme: Entity::new("APPLE", FeatureBundle {
                        gender: Some(Gender::Neuter),
                        number: Some(Number::Singular),
                        ..Default::default()
                    }),
                }],
                tense: Tense::Past,
                aspect: Aspect::Perfective,
                polarity: Polarity::Positive,
                illocution: Illocution::Statement,
            }],
        };
        
        let result = generator.generate(&utterance);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        
        // Should produce something like "Tomek dał jabłko Izie"
        assert!(output.contains("Tomek"));
        assert!(output.contains("dał"));
        assert!(output.contains("jabłko"));
        assert!(output.contains("Izie"));
    }
}
```

---

## Integration Tests

Test the full pipeline from input to output.

```rust
// tests/integration_test.rs

use lexflex::LexFlex;

#[test]
fn test_translate_pl_to_en_simple() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.translate_pl_to_en("Tomek dał jabłko Izie");
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    // Should contain key elements
    assert!(output.contains("Tomek") || output.contains("Tom"));
    assert!(output.contains("gave") || output.contains("dał"));
    assert!(output.contains("apple") || output.contains("jabłko"));
}

#[test]
fn test_translate_pl_to_en_cat_drinks_milk() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.translate_pl_to_en("Kot pije mleko");
    
    assert!(result.is_ok());
}

#[test]
fn test_translate_unknown_word() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.translate_pl_to_en("Tomek dał xyz Izie");
    
    // Should fail gracefully
    assert!(result.is_err());
}
```

---

## Golden Tests (Snapshot Testing)

Use `insta` crate for snapshot testing.

```toml
# Cargo.toml

[dev-dependencies]
insta = "1.34"
```

```rust
// tests/golden_tests.rs

use insta::assert_snapshot;
use lexflex::LexFlex;

#[test]
fn test_golden_simple_translation() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.translate_pl_to_en("Tomek dał jabłko Izie").unwrap();
    
    assert_snapshot!(result, @"Tomek gave Iza an apple");
}

#[test]
fn test_golden_cat_drinks_milk() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.translate_pl_to_en("Kot pije mleko").unwrap();
    
    assert_snapshot!(result, @"The cat drinks milk");
}

#[test]
fn test_golden_interlingua_representation() {
    let lexflex = LexFlex::new("data").unwrap();
    
    let result = lexflex.parse_to_interlingua("Tomek dał jabłko Izie", "pl").unwrap();
    
    assert_snapshot!(format!("{:?}", result), @r###"
    Utterance {
        sentences: [
            Sentence {
                frames: [
                    Transfer {
                        agent: Entity {
                            concept: "PERSON",
                            name: Some("Tomek"),
                            features: FeatureBundle {
                                gender: Some(Masculine),
                                number: Some(Singular),
                                ...
                            },
                        },
                        ...
                    },
                ],
                ...
            },
        ],
    }
    "###);
}
```

Run golden tests:

```bash
# Run tests and accept new snapshots
cargo test -- --nocapture
cargo insta review

# Or accept all snapshots
cargo insta accept
```

---

## Property-Based Tests

Use `quickcheck` or `proptest` to find edge cases.

```toml
# Cargo.toml

[dev-dependencies]
proptest = "1.4"
```

```rust
// tests/property_tests.rs

use proptest::prelude::*;
use lexflex::LexFlex;

proptest! {
    #[test]
    fn test_morphology_never_panics(
        lemma in "[a-ząćęłńóśźż]{3,10}",
        case in 0..7u8,
        number in 0..2u8,
        gender in 0..3u8,
    ) {
        let morphology = PolishMorphology::new(/* ... */);
        
        let case = match case {
            0 => Case::Nominative,
            1 => Case::Genitive,
            2 => Case::Dative,
            3 => Case::Accusative,
            4 => Case::Instrumental,
            5 => Case::Locative,
            _ => Case::Vocative,
        };
        
        let number = if number == 0 { Number::Singular } else { Number::Plural };
        
        let gender = match gender {
            0 => Gender::Masculine,
            1 => Gender::Feminine,
            _ => Gender::Neuter,
        };
        
        // Should never panic, even with invalid input
        let _ = morphology.inflect_noun(&lemma, case, number, gender);
    }

    #[test]
    fn test_parser_handles_any_input(input in "[a-ząćęłńóśźż ]{1,50}") {
        let lexflex = LexFlex::new("data").unwrap();
        
        // Should not panic, may return error
        let _ = lexflex.translate_pl_to_en(&input);
    }
}
```

---

## Data Validation Tests

Validate RON files before loading.

```rust
// tests/data_validation_test.rs

use lexflex::data::loader::DataLoader;

#[test]
fn test_concepts_valid() {
    let loader = DataLoader::new("data");
    let result = loader.load_concepts();
    
    assert!(result.is_ok(), "Failed to load concepts: {:?}", result.err());
    
    let concepts = result.unwrap();
    
    // Should have at least 50 concepts
    assert!(concepts.len() >= 50, "Expected at least 50 concepts, got {}", concepts.len());
    
    // Each concept should have an ID
    for concept in &concepts {
        assert!(!concept.id.is_empty(), "Concept ID is empty");
    }
}

#[test]
fn test_lexicon_valid() {
    let loader = DataLoader::new("data");
    
    // Polish lexicon
    let result = loader.load_lexicon("pl");
    assert!(result.is_ok(), "Failed to load Polish lexicon: {:?}", result.err());
    
    let lexicon = result.unwrap();
    assert!(lexicon.entries.len() >= 50, "Expected at least 50 Polish words");
    
    // English lexicon
    let result = loader.load_lexicon("en");
    assert!(result.is_ok(), "Failed to load English lexicon: {:?}", result.err());
    
    let lexicon = result.unwrap();
    assert!(lexicon.entries.len() >= 30, "Expected at least 30 English words");
}

#[test]
fn test_morphology_valid() {
    let loader = DataLoader::new("data");
    
    // Polish noun paradigms
    let result = loader.load_morphology("pl", "noun");
    assert!(result.is_ok(), "Failed to load Polish noun paradigms: {:?}", result.err());
    
    let paradigms = result.unwrap();
    assert!(paradigms.len() >= 3, "Expected at least 3 noun paradigms");
    
    // Polish verb paradigms
    let result = loader.load_morphology("pl", "verb");
    assert!(result.is_ok(), "Failed to load Polish verb paradigms: {:?}", result.err());
    
    let paradigms = result.unwrap();
    assert!(paradigms.len() >= 2, "Expected at least 2 verb paradigms");
}
```

---

## Benchmark Tests

Measure performance.

```rust
// benches/benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lexflex::LexFlex;

fn benchmark_translation(c: &mut Criterion) {
    let lexflex = LexFlex::new("data").unwrap();
    
    c.bench_function("translate_pl_to_en", |b| {
        b.iter(|| {
            lexflex.translate_pl_to_en(black_box("Tomek dał jabłko Izie"))
        })
    });
}

fn benchmark_parsing(c: &mut Criterion) {
    let lexflex = LexFlex::new("data").unwrap();
    
    c.bench_function("parse_polish", |b| {
        b.iter(|| {
            lexflex.parse_to_interlingua(black_box("Tomek dał jabłko Izie"), "pl")
        })
    });
}

criterion_group!(benches, benchmark_translation, benchmark_parsing);
criterion_main!(benches);
```

```toml
# Cargo.toml

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmark"
harness = false
```

Run benchmarks:

```bash
cargo bench
```

---

## Test Coverage

Measure test coverage.

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html

# Open target/tarpaulin/coverage.html
```

Target: 80%+ coverage for core modules.

---

## Continuous Integration

Add tests to CI pipeline.

```yaml
# .github/workflows/test.yml

name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test
      
      - name: Run golden tests
        run: cargo test --test golden_tests
      
      - name: Run benchmarks
        run: cargo bench
      
      - name: Check coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
```

---

## Test Organization

```
tests/
├── integration_test.rs      # Full pipeline tests
├── golden_tests.rs          # Snapshot tests
├── property_tests.rs        # Property-based tests
├── data_validation_test.rs  # RON file validation
└── benchmark.rs             # Performance benchmarks

benches/
└── benchmark.rs             # Criterion benchmarks
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_noun_declension_jabłko

# Run with output
cargo test -- --nocapture

# Run golden tests and review snapshots
cargo test --test golden_tests
cargo insta review

# Run benchmarks
cargo bench

# Check coverage
cargo tarpaulin --out Html
```

---

## Summary

Testing strategy:

1. **Unit tests** - test morphology, parser, generator in isolation
2. **Integration tests** - test full pipeline (PL → IL → EN)
3. **Golden tests** - snapshot testing for regression detection
4. **Property-based tests** - find edge cases automatically
5. **Data validation** - ensure RON files are valid
6. **Benchmarks** - measure performance
7. **Coverage** - target 80%+ for core modules
