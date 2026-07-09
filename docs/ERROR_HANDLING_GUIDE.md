# Error Handling Guide — Practical Strategies

This document provides practical guidance on handling errors throughout the lexFlex system, from parsing to generation.

## Error Philosophy

lexFlex uses a **layered error handling strategy**:

1. **Strict mode** - Fail fast on any error (for debugging)
2. **Best-effort mode** - Continue with warnings, produce partial results
3. **Graceful degradation** - Produce simplified output when full processing fails

## Error Categories

### 1. Parse Errors

Errors that occur during surface parsing and morphological analysis.

```rust
pub enum ParseError {
    // Tokenization errors
    InvalidCharacter { char: char, position: usize },
    UnexpectedEOF,
    
    // Morphological errors
    UnknownWord { word: String, suggestions: Vec<String> },
    AmbiguousMorphology { word: String, analyses: Vec<MorphAnalysis> },
    
    // Syntactic errors
    MissingVerb,
    InvalidWordOrder { expected: Vec<WordOrder>, found: WordOrder },
    UnmatchedParenthesis { position: usize },
    
    // Structural errors
    EmptyInput,
    TooLong { length: usize, max: usize },
}
```

#### Handling Strategy

```rust
fn handle_parse_error(
    error: ParseError,
    mode: ErrorMode,
) -> Result<ParseResult, ParseError> {
    match mode {
        ErrorMode::Strict => {
            // Fail immediately
            Err(error)
        }
        
        ErrorMode::BestEffort => {
            match error {
                ParseError::UnknownWord { word, suggestions } => {
                    // Try to continue with best guess
                    log::warn!("Unknown word '{}', suggestions: {:?}", word, suggestions);
                    
                    // Use first suggestion or treat as proper noun
                    if let Some(suggestion) = suggestions.first() {
                        Ok(ParseResult::with_warning(
                            suggestion.clone(),
                            format!("Unknown word '{}' replaced with '{}'", word, suggestion),
                        ))
                    } else {
                        // Treat as proper noun
                        Ok(ParseResult::with_warning(
                            word.clone(),
                            format!("Unknown word '{}' treated as proper noun", word),
                        ))
                    }
                }
                
                ParseError::AmbiguousMorphology { word, analyses } => {
                    // Choose most likely analysis
                    log::warn!("Ambiguous morphology for '{}', choosing first", word);
                    let best = analyses.first().unwrap();
                    Ok(ParseResult::with_warning(
                        best.clone(),
                        format!("Ambiguous morphology for '{}', chose first analysis", word),
                    ))
                }
                
                ParseError::MissingVerb => {
                    // Cannot continue without verb
                    Err(error)
                }
                
                _ => Err(error),
            }
        }
        
        ErrorMode::GracefulDegradation => {
            match error {
                ParseError::UnknownWord { word, .. } => {
                    // Create placeholder entity
                    Ok(ParseResult::degraded(
                        Entity::unknown(word),
                        "Unknown word replaced with placeholder".to_string(),
                    ))
                }
                
                ParseError::MissingVerb => {
                    // Create dummy verb frame
                    Ok(ParseResult::degraded(
                        Frame::dummy(),
                        "Missing verb replaced with dummy frame".to_string(),
                    ))
                }
                
                _ => Err(error),
            }
        }
    }
}
```

### 2. Deduction Errors

Errors that occur during semantic deduction and frame resolution.

```rust
pub enum DeductionError {
    // Frame resolution errors
    NoVerbFound,
    UnknownVerb { lemma: String },
    UnknownFrameType { frame_type: String },
    MissingRequiredRole { role: SemanticRole, frame_type: String },
    
    // Case resolution errors
    UnresolvableCase { token: String, possible_cases: Vec<Case> },
    ConflictingCaseAssignments { 
        token: String, 
        assignments: Vec<(SemanticRole, Case)> 
    },
    
    // Semantic validation errors
    SemanticTypeViolation { 
        role: SemanticRole, 
        expected: ConceptId, 
        found: ConceptId 
    },
    
    // Pronoun resolution errors
    AmbiguousPronoun { pronoun: String, candidates: Vec<Entity> },
    NoAntecedentFound { pronoun: String },
    
    // Temporal errors
    InvalidTemporalReference { expression: String },
}
```

#### Handling Strategy

```rust
fn handle_deduction_error(
    error: DeductionError,
    mode: ErrorMode,
    partial_result: Option<Frame>,
) -> Result<DeductionResult, DeductionError> {
    match mode {
        ErrorMode::Strict => {
            Err(error)
        }
        
        ErrorMode::BestEffort => {
            match error {
                DeductionError::MissingRequiredRole { role, frame_type } => {
                    log::warn!(
                        "Missing required role {:?} for frame {}, using placeholder",
                        role,
                        frame_type
                    );
                    
                    // Create placeholder entity for missing role
                    let placeholder = Entity::placeholder(role);
                    Ok(DeductionResult::with_warning(
                        partial_result.unwrap().with_role(role, placeholder),
                        format!("Missing {:?} replaced with placeholder", role),
                    ))
                }
                
                DeductionError::SemanticTypeViolation { role, expected, found } => {
                    log::warn!(
                        "Semantic type violation: {:?} expected {:?}, found {:?}",
                        role,
                        expected,
                        found
                    );
                    
                    // Continue with warning
                    Ok(DeductionResult::with_warning(
                        partial_result.unwrap(),
                        format!(
                            "Semantic type violation: {:?} expected {:?}, found {:?}",
                            role, expected, found
                        ),
                    ))
                }
                
                DeductionError::AmbiguousPronoun { pronoun, candidates } => {
                    log::warn!("Ambiguous pronoun '{}', choosing first candidate", pronoun);
                    
                    // Choose first candidate
                    if let Some(candidate) = candidates.first() {
                        Ok(DeductionResult::with_warning(
                            partial_result.unwrap().resolve_pronoun(pronoun, candidate),
                            format!(
                                "Ambiguous pronoun '{}' resolved to first candidate",
                                pronoun
                            ),
                        ))
                    } else {
                        Err(error)
                    }
                }
                
                _ => Err(error),
            }
        }
        
        ErrorMode::GracefulDegradation => {
            match error {
                DeductionError::MissingRequiredRole { role, .. } => {
                    // Use generic placeholder
                    let placeholder = Entity::generic(role);
                    Ok(DeductionResult::degraded(
                        partial_result.unwrap().with_role(role, placeholder),
                        format!("Missing {:?} replaced with generic placeholder", role),
                    ))
                }
                
                DeductionError::SemanticTypeViolation { .. } => {
                    // Ignore violation and continue
                    Ok(DeductionResult::degraded(
                        partial_result.unwrap(),
                        "Semantic type violation ignored".to_string(),
                    ))
                }
                
                DeductionError::AmbiguousPronoun { pronoun, .. } => {
                    // Leave pronoun unresolved
                    Ok(DeductionResult::degraded(
                        partial_result.unwrap(),
                        format!("Pronoun '{}' left unresolved", pronoun),
                    ))
                }
                
                _ => Err(error),
            }
        }
    }
}
```

### 3. Generation Errors

Errors that occur during text generation.

```rust
pub enum GenerateError {
    // Lexical errors
    MissingLexicalEntry { concept: ConceptId, language: LanguageId },
    AmbiguousLexicalEntry { concept: ConceptId, entries: Vec<LexEntry> },
    
    // Morphological errors
    MorphologyError { lemma: String, features: FeatureBundle, reason: String },
    UnknownParadigm { paradigm: String },
    
    // Structural errors
    UnsupportedInterlinguaType,
    UnsupportedFrameType { frame_type: String, language: LanguageId },
    
    // Capability errors
    InexpressibleFeature { feature: String, language: LanguageId },
    MissingCapability { capability: Capability, language: LanguageId },
}
```

#### Handling Strategy

```rust
fn handle_generate_error(
    error: GenerateError,
    mode: ErrorMode,
    partial_output: Option<String>,
) -> Result<String, GenerateError> {
    match mode {
        ErrorMode::Strict => {
            Err(error)
        }
        
        ErrorMode::BestEffort => {
            match error {
                GenerateError::MissingLexicalEntry { concept, language } => {
                    log::warn!(
                        "Missing lexical entry for {:?} in {:?}, using concept name",
                        concept,
                        language
                    );
                    
                    // Use concept name as fallback
                    let fallback = concept.0.clone();
                    Ok(format!(
                        "{} [{}]",
                        partial_output.unwrap_or_default(),
                        fallback
                    ))
                }
                
                GenerateError::AmbiguousLexicalEntry { concept, entries } => {
                    log::warn!(
                        "Ambiguous lexical entry for {:?}, choosing first",
                        concept
                    );
                    
                    // Choose first entry
                    if let Some(entry) = entries.first() {
                        Ok(format!(
                            "{} {}",
                            partial_output.unwrap_or_default(),
                            entry.lemma
                        ))
                    } else {
                        Err(error)
                    }
                }
                
                GenerateError::MorphologyError { lemma, .. } => {
                    log::warn!("Morphology error for '{}', using lemma", lemma);
                    
                    // Fall back to lemma
                    Ok(format!(
                        "{} {}",
                        partial_output.unwrap_or_default(),
                        lemma
                    ))
                }
                
                GenerateError::InexpressibleFeature { feature, .. } => {
                    log::warn!("Inexpressible feature: {}", feature);
                    
                    // Continue without this feature
                    Ok(partial_output.unwrap_or_default())
                }
                
                _ => Err(error),
            }
        }
        
        ErrorMode::GracefulDegradation => {
            match error {
                GenerateError::MissingLexicalEntry { concept, .. } => {
                    // Use concept name in brackets
                    Ok(format!("[{}]", concept.0))
                }
                
                GenerateError::MorphologyError { lemma, .. } => {
                    // Use uninflected lemma
                    Ok(lemma)
                }
                
                GenerateError::InexpressibleFeature { .. } => {
                    // Skip this feature
                    Ok(partial_output.unwrap_or_default())
                }
                
                GenerateError::UnsupportedFrameType { .. } => {
                    // Generate simplified output
                    Ok("[unsupported frame]")
                }
                
                _ => Err(error),
            }
        }
    }
}
```

### 4. Translation Errors

Errors that occur during the translation pipeline.

```rust
pub enum TranslateError {
    // Pipeline errors
    ParseError(ParseError),
    DeductionError(DeductionError),
    GenerateError(GenerateError),
    
    // Capability errors
    UnsupportedLanguage { language: LanguageId },
    InexpressibleInTarget { 
        target: LanguageId, 
        features: Vec<InexpressibleFeature> 
    },
    
    // System errors
    InternalError { message: String },
}
```

#### Handling Strategy

```rust
fn handle_translate_error(
    error: TranslateError,
    mode: ErrorMode,
) -> Result<String, TranslateError> {
    match mode {
        ErrorMode::Strict => {
            Err(error)
        }
        
        ErrorMode::BestEffort => {
            match error {
                TranslateError::InexpressibleInTarget { target, features } => {
                    log::warn!(
                        "Inexpressible features in {:?}: {:?}",
                        target,
                        features
                    );
                    
                    // Try to translate without these features
                    // (This would require re-running generation with modified IL)
                    Err(error) // Cannot easily recover
                }
                
                TranslateError::ParseError(parse_err) => {
                    // Try to recover from parse error
                    match handle_parse_error(parse_err, mode) {
                        Ok(result) => {
                            // Continue with partial parse
                            // (Would need to continue pipeline)
                            Err(TranslateError::ParseError(parse_err))
                        }
                        Err(err) => Err(TranslateError::ParseError(err)),
                    }
                }
                
                _ => Err(error),
            }
        }
        
        ErrorMode::GracefulDegradation => {
            match error {
                TranslateError::InexpressibleInTarget { target, .. } => {
                    // Return message explaining limitation
                    Ok(format!(
                        "[Translation to {:?} not fully supported]",
                        target
                    ))
                }
                
                TranslateError::UnsupportedLanguage { language } => {
                    Ok(format!("[Language {:?} not supported]", language))
                }
                
                _ => Err(error),
            }
        }
    }
}
```

## Error Mode Configuration

Configure error handling at the API level:

```rust
pub struct TranslateOptions {
    // ... other options
    
    pub error_mode: ErrorMode,
    pub return_warnings: bool,
    pub log_errors: bool,
}

pub enum ErrorMode {
    Strict,              // Fail on any error
    BestEffort,          // Try to recover, return warnings
    GracefulDegradation, // Produce simplified output
}
```

### Usage Examples

```rust
// Strict mode - fail fast
let result = api.translate(
    "Tomek dał jabłko Izie",
    LanguageId::PL,
    LanguageId::EN,
    TranslateOptions {
        error_mode: ErrorMode::Strict,
        ..Default::default()
    },
)?;

// Best-effort mode - continue with warnings
let result = api.translate(
    "Tomek dał xyz Izie",
    LanguageId::PL,
    LanguageId::EN,
    TranslateOptions {
        error_mode: ErrorMode::BestEffort,
        return_warnings: true,
        ..Default::default()
    },
)?;

match result {
    Ok(TranslationResult { output, warnings, .. }) => {
        println!("Output: {}", output);
        for warning in warnings {
            println!("Warning: {}", warning);
        }
    }
    Err(err) => {
        println!("Error: {:?}", err);
    }
}

// Graceful degradation - produce something
let result = api.translate(
    "Completely invalid input @#$%",
    LanguageId::PL,
    LanguageId::EN,
    TranslateOptions {
        error_mode: ErrorMode::GracefulDegradation,
        return_warnings: true,
        ..Default::default()
    },
)?;

// Will return degraded output with warnings
```

## Error Recovery Strategies

### Strategy 1: Fallback to Lemma

When morphological inflection fails, use the uninflected lemma:

```rust
fn safe_inflect(
    lemma: &str,
    features: &FeatureBundle,
    morphology: &MorphologyEngine,
) -> String {
    match morphology.inflect(lemma, features) {
        Ok(inflected) => inflected,
        Err(err) => {
            log::warn!("Inflection failed for '{}': {:?}", lemma, err);
            lemma.to_string() // Fallback to lemma
        }
    }
}
```

### Strategy 2: Use Synonyms

When a lexical entry is missing, try synonyms:

```rust
fn find_lexical_entry(
    concept: &ConceptId,
    language: LanguageId,
    lexicon: &Lexicon,
    ontology: &Ontology,
) -> Option<LexEntry> {
    // Try direct lookup
    if let Some(entry) = lexicon.lookup(language, concept) {
        return Some(entry);
    }
    
    // Try synonyms
    let synonyms = ontology.get_synonyms(concept);
    for synonym in synonyms {
        if let Some(entry) = lexicon.lookup(language, &synonym) {
            log::info!("Using synonym {:?} for {:?}", synonym, concept);
            return Some(entry);
        }
    }
    
    // Try hypernyms (more general concepts)
    if let Some(hypernym) = ontology.get_hypernym(concept) {
        if let Some(entry) = lexicon.lookup(language, &hypernym) {
            log::info!("Using hypernym {:?} for {:?}", hypernym, concept);
            return Some(entry);
        }
    }
    
    None
}
```

### Strategy 3: Simplify Output

When generation fails, produce simplified output:

```rust
fn safe_generate(
    il: &InterlinguaNode,
    generator: &dyn Generator,
) -> Result<String, GenerateError> {
    match generator.generate(il) {
        Ok(output) => Ok(output),
        Err(err) => {
            log::warn!("Generation failed: {:?}", err);
            
            // Try simplified generation
            match simplify_interlingua(il) {
                Ok(simplified) => generator.generate(&simplified),
                Err(_) => Err(err),
            }
        }
    }
}

fn simplify_interlingua(il: &InterlinguaNode) -> Result<InterlinguaNode, ()> {
    match il {
        InterlinguaNode::Natural(utterance) => {
            // Remove temporal modifiers, simplify frames
            let simplified = utterance.clone();
            // ... simplification logic
            Ok(InterlinguaNode::Natural(simplified))
        }
        _ => Err(()),
    }
}
```

## Error Logging

Log all errors for debugging:

```rust
fn log_error(error: &dyn std::error::Error, context: &str) {
    log::error!("[{}] Error: {:?}", context, error);
    
    // Log full error chain
    let mut current = error.source();
    let mut depth = 0;
    while let Some(cause) = current {
        log::error!("[{}] Caused by ({}): {:?}", context, depth, cause);
        current = cause.source();
        depth += 1;
    }
}
```

## Error Messages

Provide helpful error messages:

```rust
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnknownWord { word, suggestions } => {
                write!(f, "Unknown word: '{}'", word)?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean: {}?", suggestions.join(", "))?;
                }
                Ok(())
            }
            
            ParseError::MissingVerb => {
                write!(f, "Sentence is missing a verb")
            }
            
            ParseError::InvalidWordOrder { expected, found } => {
                write!(
                    f,
                    "Invalid word order. Expected one of: {:?}, found: {:?}",
                    expected, found
                )
            }
            
            _ => write!(f, "{:?}", self),
        }
    }
}
```

## Testing Error Handling

```rust
#[test]
fn test_unknown_word_best_effort() {
    let api = create_test_api();
    
    let result = api.translate(
        "Tomek dał xyz Izie",
        LanguageId::PL,
        LanguageId::EN,
        TranslateOptions {
            error_mode: ErrorMode::BestEffort,
            return_warnings: true,
            ..Default::default()
        },
    );
    
    assert!(result.is_ok());
    let TranslationResult { output, warnings, .. } = result.unwrap();
    
    // Should produce output with warning
    assert!(!output.is_empty());
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("xyz"));
}

#[test]
fn test_missing_lexical_entry_graceful() {
    let api = create_test_api();
    
    let result = api.translate(
        "Tomek dał kwantyzator Izie", // "kwantyzator" not in lexicon
        LanguageId::PL,
        LanguageId::EN,
        TranslateOptions {
            error_mode: ErrorMode::GracefulDegradation,
            return_warnings: true,
            ..Default::default()
        },
    );
    
    assert!(result.is_ok());
    let TranslationResult { output, warnings, .. } = result.unwrap();
    
    // Should produce degraded output
    assert!(output.contains("[quantizer]")); // Concept name in brackets
    assert!(!warnings.is_empty());
}

#[test]
fn test_semantic_type_violation() {
    let api = create_test_api();
    
    // "Kamień dał jabłko Izie" - Stone gave apple to Iza (semantic violation)
    let result = api.translate(
        "Kamień dał jabłko Izie",
        LanguageId::PL,
        LanguageId::EN,
        TranslateOptions {
            error_mode: ErrorMode::BestEffort,
            return_warnings: true,
            ..Default::default()
        },
    );
    
    assert!(result.is_ok());
    let TranslationResult { warnings, .. } = result.unwrap();
    
    // Should have semantic type violation warning
    assert!(warnings.iter().any(|w| w.contains("semantic type violation")));
}
```

## Summary

### Key Principles

1. **Fail gracefully** - Never crash, always produce something useful
2. **Log everything** - Errors are valuable for debugging
3. **Provide helpful messages** - Tell users what went wrong and how to fix it
4. **Offer recovery strategies** - Try synonyms, fall back to lemmas, simplify output
5. **Make error mode configurable** - Let users choose strict vs best-effort

### Error Mode Recommendations

- **Development/Debugging:** Use `Strict` mode to catch all errors
- **Production (user-facing):** Use `BestEffort` mode with warnings
- **Demo/Showcase:** Use `GracefulDegradation` to always show something

### Common Pitfalls

1. **Swallowing errors silently** - Always log errors, even if you recover
2. **Returning empty results** - Prefer degraded output over empty output
3. **Cryptic error messages** - Provide actionable information
4. **Inconsistent error handling** - Use the same strategy throughout a component

## References

- [ERROR_HANDLING.md](./ERROR_HANDLING.md) - Error type definitions
- [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Implementation guidance
- [DEDUCTION.md](./DEDUCTION.md) - Deduction error handling
- [GENERATOR.md](./GENERATOR.md) - Generator error handling
