# Error Handling — Graceful Degradation

lexFlex is designed to handle errors and ambiguities gracefully, providing meaningful feedback and partial results when perfect processing isn't possible.

## Error Categories

```rust
pub enum LexFlexError {
    /// Parsing errors — cannot understand input
    ParseError(ParseError),
    
    /// Generation errors — cannot produce output
    GenerateError(GenerateError),
    
    /// Translation errors — capability mismatch
    TranslateError(TranslateError),
    
    /// Deduction errors — cannot resolve ambiguity
    DeductionError(DeductionError),
    
    /// Validation errors — semantic type mismatch
    ValidationError(ValidationError),
}
```

## Parse Errors

```rust
pub enum ParseError {
    /// Unknown word not in lexicon
    UnknownToken {
        token: String,
        position: usize,
        suggestions: Vec<String>,
    },
    
    /// Morphological ambiguity — multiple possible analyses
    MorphAmbiguous {
        token: String,
        candidates: Vec<MorphAnalysis>,
        context: ParseContext,
    },
    
    /// Syntactic error — invalid sentence structure
    SyntaxError {
        message: String,
        position: usize,
        expected: Vec<TokenType>,
        found: TokenType,
    },
    
    /// Semantic error — no matching frame
    NoFrameMatch {
        verb: String,
        available_frames: Vec<FrameType>,
    },
    
    /// Unresolved reference — pronoun without antecedent
    UnresolvedReference {
        pronoun: String,
        candidates: Vec<Entity>,
        context: DiscourseContext,
    },
    
    /// Role assignment failure — required role has no filler
    RoleUnassigned {
        frame: FrameType,
        missing_role: SemanticRole,
        available_entities: Vec<Entity>,
    },
}
```

## Generation Errors

```rust
pub enum GenerateError {
    /// Missing lexeme — no word for concept in target language
    MissingLexeme {
        concept: ConceptId,
        language: LanguageId,
        suggestions: Vec<ConceptId>,
    },
    
    /// Morphological failure — cannot inflect form
    MorphFailed {
        lemma: String,
        features: FeatureBundle,
        reason: String,
    },
    
    /// Agreement failure — adj-noun mismatch
    AgreementFailed {
        adjective: String,
        noun: String,
        expected_features: FeatureBundle,
        actual_features: FeatureBundle,
    },
    
    /// Unresolved reference in generation
    UnresolvedReference {
        entity: Entity,
        reason: String,
    },
}
```

## Translation Errors

```rust
pub enum TranslateError {
    /// Target language cannot express required capabilities
    InexpressibleInTarget {
        target: LanguageId,
        missing_capabilities: Vec<Capability>,
        suggestions: Vec<TranslationSuggestion>,
    },
    
    /// Unsupported language
    UnsupportedLanguage(LanguageId),
    
    /// Lossy translation — some meaning was lost
    LossyTranslation {
        lost_features: Vec<LostFeature>,
        approximate_output: String,
    },
}

pub struct TranslationSuggestion {
    /// What was inexpressible
    pub feature: Capability,
    
    /// How to work around it
    pub workaround: Workaround,
}

pub enum Workaround {
    /// Paraphrase in target language
    Paraphrase(String),
    
    /// Use loanword
    Loanword(String),
    
    /// Add explanation
    AddExplanation(String),
    
    /// Omit (with warning)
    Omit,
}
```

## Graceful Degradation Strategy

When errors occur, lexFlex attempts to provide partial results rather than failing completely.

```rust
pub enum DegradationLevel {
    /// Complete success — full understanding and generation
    Complete,
    
    /// Partial success — some elements unresolved but structure preserved
    Partial {
        resolved: InterlinguaNode,
        unresolved: Vec<UnresolvedElement>,
    },
    
    /// Minimal success — only main intent captured
    Minimal {
        intent: Intent,
        details_lost: Vec<String>,
    },
    
    /// Failed — cannot process at all
    Failed {
        error: LexFlexError,
    },
}

pub struct ErrorHandler {
    /// Configuration for degradation strategy
    pub strategy: DegradationStrategy,
}

pub enum DegradationStrategy {
    /// Ask user for clarification
    AskClarification,
    
    /// Use best guess with warnings
    BestGuess,
    
    /// Fail fast with detailed error
    FailFast,
    
    /// Return partial result with unresolved markers
    PartialResult,
}
```

## Error Recovery Patterns

### 1. Spelling Correction

```rust
pub fn handle_unknown_token(
    token: &str,
    lexicon: &SubLexicon,
) -> Result<Vec<String>, ParseError> {
    // Generate spelling suggestions using edit distance
    let suggestions = lexicon.find_similar(token, max_distance: 2);
    
    if suggestions.is_empty() {
        Err(ParseError::UnknownToken {
            token: token.to_string(),
            position: 0,
            suggestions: vec![],
        })
    } else {
        // Return suggestions for user or auto-correction
        Ok(suggestions)
    }
}
```

### 2. Morphological Disambiguation

```rust
pub fn resolve_morph_ambiguity(
    token: &str,
    candidates: Vec<MorphAnalysis>,
    context: &ParseContext,
) -> MorphAnalysis {
    // Use context to disambiguate
    
    // Strategy 1: Verb subcategorization
    if let Some(verb_subcat) = context.verb_subcategorization {
        let filtered = candidates.iter()
            .filter(|c| verb_subcat.accepts(c.case))
            .collect::<Vec<_>>();
        
        if filtered.len() == 1 {
            return filtered[0].clone();
        }
    }
    
    // Strategy 2: Agreement with nearby words
    if let Some(adj) = context.adjacent_adjective {
        let filtered = candidates.iter()
            .filter(|c| c.agrees_with(&adj))
            .collect::<Vec<_>>();
        
        if filtered.len() == 1 {
            return filtered[0].clone();
        }
    }
    
    // Strategy 3: Frequency-based (most common form)
    candidates.iter()
        .max_by_key(|c| c.frequency)
        .unwrap()
        .clone()
}
```

### 3. Reference Resolution Fallback

```rust
pub fn resolve_reference(
    pronoun: &str,
    discourse: &Discourse,
) -> Result<Entity, DeductionError> {
    // Try normal resolution
    match discourse.resolve_pronoun(pronoun) {
        Ok(entity) => Ok(entity),
        
        Err(DeductionError::Ambiguous(candidates)) => {
            // Multiple candidates — use salience
            let most_salient = candidates.iter()
                .max_by_key(|e| e.salience_score())
                .unwrap();
            
            Ok(most_salient.clone())
        }
        
        Err(DeductionError::NoCandidates) => {
            // No candidates — check if speaker/addressee
            if pronoun == "I" || pronoun == "ja" {
                return Ok(discourse.speaker.clone());
            }
            if pronoun == "you" || pronoun == "ty" {
                return Ok(discourse.addressee.clone());
            }
            
            // Truly unresolved — return error
            Err(DeductionError::UnresolvedReference {
                pronoun: pronoun.to_string(),
                candidates: vec![],
                context: discourse.clone(),
            })
        }
    }
}
```

### 4. Capability Mismatch Handling

```rust
pub fn handle_capability_mismatch(
    il: &InterlinguaNode,
    target: &dyn IMeaningRepresentation,
) -> Result<String, TranslateError> {
    let inexpressible = target.can_express(il);
    
    if inexpressible.is_empty() {
        // No mismatch — proceed normally
        return target.from_interlingua(il);
    }
    
    // Try workarounds for each inexpressible feature
    let mut modified_il = il.clone();
    let mut warnings = vec![];
    
    for feature in &inexpressible {
        match feature {
            Capability::TemporalReference => {
                // Target doesn't support temporal — omit or paraphrase
                modified_il = modified_il.remove_temporal();
                warnings.push("Temporal information omitted (not supported in target)");
            }
            
            Capability::EmotionExpression => {
                // Target doesn't support emotion — neutralize
                modified_il = modified_il.neutralize_emotion();
                warnings.push("Emotional tone neutralized (not supported in target)");
            }
            
            Capability::Quantification => {
                // Target doesn't support quantifiers — expand
                modified_il = modified_il.expand_quantification();
                warnings.push("Quantifier expanded to explicit enumeration");
            }
            
            _ => {
                // Cannot workaround — return error with suggestions
                return Err(TranslateError::InexpressibleInTarget {
                    target: target.language_id(),
                    missing_capabilities: vec![feature.clone()],
                    suggestions: vec![],
                });
            }
        }
    }
    
    // Generate with modifications
    let output = target.from_interlingua(&modified_il)?;
    
    // Return with warnings
    Ok(format!("{} [WARNINGS: {}]", output, warnings.join(", ")))
}
```

## Error Reporting

lexFlex provides structured error reports for debugging and user feedback.

```rust
pub struct ErrorReport {
    /// Error type and details
    pub error: LexFlexError,
    
    /// Where the error occurred
    pub location: ErrorLocation,
    
    /// What was being processed
    pub context: ProcessingContext,
    
    /// Suggested fixes
    pub suggestions: Vec<Suggestion>,
    
    /// Partial result (if available)
    pub partial_result: Option<InterlinguaNode>,
    
    /// Degradation level
    pub degradation: DegradationLevel,
}

pub struct ErrorLocation {
    pub input: String,
    pub position: usize,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

pub struct ProcessingContext {
    pub source_language: LanguageId,
    pub target_language: Option<LanguageId>,
    pub discourse_state: Option<Discourse>,  // None in MVP v0.1, Some in v0.2+
    pub pipeline_stage: PipelineStage,
}

pub enum PipelineStage {
    Tokenization,
    MorphologicalAnalysis,
    SyntacticParsing,
    SemanticParsing,
    Deduction,
    CapabilityCheck,
    Generation,
}

pub struct Suggestion {
    pub description: String,
    pub action: SuggestedAction,
}

pub enum SuggestedAction {
    /// Correct spelling
    CorrectSpelling { from: String, to: String },
    
    /// Add to lexicon
    AddToLexicon { word: String, concept: ConceptId },
    
    /// Clarify with user
    AskClarification { question: String },
    
    /// Use alternative phrasing
    Rephrase { original: String, alternative: String },
    
    /// Omit problematic element
    Omit { element: String, reason: String },
}
```

## Logging and Diagnostics

lexFlex logs all processing steps for post-mortem analysis.

```rust
pub struct ProcessingLog {
    pub timestamp: Timestamp,
    pub session_id: String,
    pub input: String,
    pub pipeline_stages: Vec<StageLog>,
    pub output: Option<String>,
    pub error: Option<ErrorReport>,
}

pub struct StageLog {
    pub stage: PipelineStage,
    pub duration_ms: u64,
    pub input: String,
    pub output: String,
    pub warnings: Vec<String>,
}

impl ProcessingLog {
    pub fn save(&self, log_dir: &Path) {
        let filename = format!(
            "{}_{}.json",
            self.timestamp.format("%Y-%m-%d_%H-%M-%S"),
            self.session_id
        );
        
        let path = log_dir.join(filename);
        let json = serde_json::to_string_pretty(self).unwrap();
        
        std::fs::write(path, json).unwrap();
    }
}
```

## Best Practices

### 1. Fail Gracefully

Always provide partial results when possible:

```rust
// BAD: Fail completely
fn parse(input: &str) -> Result<Utterance, ParseError> {
    let tokens = tokenize(input)?;
    let tree = parse_syntax(&tokens)?;  // If this fails, everything is lost
    // ...
}

// GOOD: Capture partial results
fn parse(input: &str) -> Result<Utterance, ParseError> {
    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => return Err(e.with_partial(tokens_so_far)),
    };
    
    let tree = match parse_syntax(&tokens) {
        Ok(t) => t,
        Err(e) => return Err(e.with_partial(partial_tree)),
    };
    // ...
}
```

### 2. Provide Context

Always include context in errors:

```rust
// BAD: Generic error
Err(ParseError::SyntaxError("Invalid syntax".to_string()))

// GOOD: Contextual error
Err(ParseError::SyntaxError {
    message: "Expected noun phrase after verb".to_string(),
    position: 15,
    expected: vec![TokenType::Noun, TokenType::Pronoun],
    found: TokenType::Verb,
})
```

### 3. Suggest Fixes

Always provide actionable suggestions:

```rust
// BAD: Just report error
Err(ParseError::UnknownToken("xyz".to_string()))

// GOOD: Suggest corrections
Err(ParseError::UnknownToken {
    token: "xyz".to_string(),
    position: 10,
    suggestions: vec!["xyz".to_string(), "xy".to_string()],
})
```

### 4. Log Everything

Always log processing steps for debugging:

```rust
fn parse(input: &str) -> Result<Utterance, ParseError> {
    log::info!("[PARSE] Input: {}", input);
    
    let tokens = tokenize(input)?;
    log::debug!("[PARSE] Tokens: {:?}", tokens);
    
    let tree = parse_syntax(&tokens)?;
    log::debug!("[PARSE] Syntax tree: {:?}", tree);
    
    // ...
}
```

## Summary

lexFlex handles errors through:

1. **Structured error types** — specific errors with full context
2. **Graceful degradation** — partial results over complete failure
3. **Recovery strategies** — spelling correction, disambiguation, fallbacks
4. **Capability checking** — validate before translating
5. **Comprehensive logging** — full pipeline traces for debugging
6. **Actionable suggestions** — help users fix problems

The goal is to **never lose information** — even when perfect processing isn't possible, lexFlex provides the best possible result with clear warnings about what couldn't be processed.
