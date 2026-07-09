# API — Public Interface

The lexFlex API provides a clean, layered interface for clients: translation applications, conversational bots, CLI tools, and future integrations.

---

## Core API: LexFlexAPI

The main entry point for all lexFlex operations.

```rust
pub struct LexFlexAPI {
    /// Universal translator with all registered engines
    translator: UniversalTranslator,

    /// Processing log for debugging
    log: ProcessingLog,
}
```

**Note:** Conversation features (discourse context, long-term memory) are planned for v0.2+. See [Future: Conversation API](#future-conversation-api-v02) section below.

---

## Translation API

### Simple Translation

```rust
impl LexFlexAPI {
    /// Translate text from one language to another
    pub fn translate(
        &self,
        input: &str,
        from: &LanguageId,
        to: &LanguageId,
    ) -> Result<TranslationResult, TranslateError> {
        let result = self.translator.translate(input, from, to)?;

        Ok(TranslationResult {
            output: result,
            interlingua: None,  // not exposed by default
            warnings: vec![],
        })
    }
}

pub struct TranslationResult {
    /// The translated text
    pub output: String,
    
    /// Optional: the Interlingua representation (for debugging)
    pub interlingua: Option<InterlinguaNode>,
    
    /// Warnings about lossy translation
    pub warnings: Vec<String>,
}
```

### Translation with Options

```rust
pub struct TranslateOptions {
    /// Return Interlingua representation
    pub return_interlingua: bool,
    
    /// Use best-effort translation (allow lossy)
    pub best_effort: bool,
    
    /// Target register (formal, informal, etc.)
    pub target_register: Option<Register>,
    
    /// Preserve formatting (markdown, etc.)
    pub preserve_formatting: bool,
}

impl LexFlexAPI {
    pub fn translate_with_options(
        &self,
        input: &str,
        from: &LanguageId,
        to: &LanguageId,
        options: &TranslateOptions,
    ) -> Result<TranslationResult, TranslateError> {
        // ...
    }
}
```

### Batch Translation

```rust
impl LexFlexAPI {
    /// Translate multiple texts
    pub fn translate_batch(
        &self,
        inputs: &[(&str, LanguageId, LanguageId)],
    ) -> Vec<Result<TranslationResult, TranslateError>> {
        inputs.iter()
            .map(|(input, from, to)| self.translate(input, from, to))
            .collect()
    }
}
```

---

## Parsing API

### Parse to Interlingua

```rust
impl LexFlexAPI {
    /// Parse text into Interlingua representation
    pub fn parse(
        &self,
        input: &str,
        language: &LanguageId,
    ) -> Result<ParseResult, ParseError> {
        let engine = self.translator.get_engine(language)?;
        let utterance = engine.to_interlingua(input)?;

        Ok(ParseResult {
            interlingua: utterance,
            warnings: vec![],
        })
    }
}

pub struct ParseResult {
    /// The Interlingua representation
    pub interlingua: InterlinguaNode,
    
    /// Warnings about unresolved elements
    pub warnings: Vec<String>,
}
```

### Analyze Text

```rust
impl LexFlexAPI {
    /// Analyze text structure (tokens, morphology, syntax)
    pub fn analyze(
        &self,
        input: &str,
        language: &LanguageId,
    ) -> Result<AnalysisResult, ParseError> {
        let engine = self.translator.get_engine(language)?;
        
        // Only for natural languages
        let natural_engine = engine.as_natural_language()?;
        
        let tokens = natural_engine.tokenize(input);
        let morphology = natural_engine.analyze_morphology(&tokens);
        let syntax = natural_engine.parse_syntax(&morphology)?;
        
        Ok(AnalysisResult {
            tokens,
            morphology,
            syntax_tree: syntax,
        })
    }
}

pub struct AnalysisResult {
    pub tokens: Vec<Token>,
    pub morphology: Vec<MorphAnalysis>,
    pub syntax_tree: SyntaxTree,
}
```

---

## Generation API

### Generate from Interlingua

```rust
impl LexFlexAPI {
    /// Generate text from Interlingua representation
    pub fn generate(
        &self,
        interlingua: &InterlinguaNode,
        language: &LanguageId,
    ) -> Result<String, GenerateError> {
        let engine = self.translator.get_engine(language)?;
        engine.from_interlingua(interlingua)
    }
}
```

### Generate with Style

```rust
pub struct GenerateOptions {
    /// Target register (formal, informal, etc.)
    pub register: Option<Register>,
    
    /// Target tone
    pub tone: Option<Tone>,
    
    /// Maximum length (in words)
    pub max_length: Option<usize>,
    
    /// Prefer pronouns over nouns (when appropriate)
    pub prefer_pronouns: bool,
}

impl LexFlexAPI {
    pub fn generate_with_options(
        &self,
        interlingua: &InterlinguaNode,
        language: &LanguageId,
        options: &GenerateOptions,
    ) -> Result<String, GenerateError> {
        // Apply options to generation
        let modified_il = self.apply_style(interlingua, options);
        self.generate(&modified_il, language)
    }
}
```

---

## Future: Conversation API (v0.2+)

**⚠️ NOT AVAILABLE IN MVP v0.1**

The following features are planned for v0.2+ and require:
- `Discourse` context management
- `LongTermMemory` persistence
- `SpeechActRecognizer`, `IntentExtractor`, `EmotionRecognizer`
- `ResponsePlanner`

For detailed specifications, see:
- [DIALOGUE.md](./DIALOGUE.md)
- [SPEECH_ACTS.md](./SPEECH_ACTS.md)
- [INTENTS.md](./INTENTS.md)
- [RESPONSE_PLANNING.md](./RESPONSE_PLANNING.md)
- [MEMORY.md](./MEMORY.md)

### Planned Features

```rust
// Future API (v0.2+)
impl LexFlexAPI {
    /// Process user utterance: parse + interpret + update context
    pub fn process_utterance(
        &mut self,
        input: &str,
        language: &LanguageId,
    ) -> Result<ProcessedUtterance, ProcessError> {
        // 1. Parse
        let utterance = self.parse(input, language)?;

        // 2. Interpret (speech acts, intents, emotions)
        let speech_act = SpeechActRecognizer::recognize(&utterance.interlingua);
        let intent = IntentExtractor::extract(&utterance.interlingua, speech_act);
        let emotion = EmotionRecognizer::recognize(&utterance.interlingua);

        // 3. Update discourse
        self.discourse.add_utterance(&utterance.interlingua);

        // 4. Update long-term memory
        self.memory.update_from_utterance(&utterance.interlingua);

        Ok(ProcessedUtterance {
            utterance: utterance.interlingua,
            speech_act,
            intent,
            emotion,
        })
    }

    /// Plan and generate response to user utterance
    pub fn respond(
        &mut self,
        processed: &ProcessedUtterance,
        language: &LanguageId,
    ) -> Result<String, RespondError> {
        // 1. Plan response
        let plan = ResponsePlanner::plan(
            &processed.intent,
            &self.discourse,
            &self.memory,
        );

        // 2. Generate response Interlingua
        let response_il = plan.to_interlingua();

        // 3. Generate surface form
        let response = self.generate(&response_il, language)?;

        // 4. Update discourse with bot's utterance
        self.discourse.add_utterance(&response_il);

        Ok(response)
    }

    /// Full conversation turn: input → response
    pub fn converse(
        &mut self,
        input: &str,
        language: &LanguageId,
    ) -> Result<String, ConverseError> {
        let processed = self.process_utterance(input, language)?;
        self.respond(&processed, language)
    }
}

pub struct ProcessedUtterance {
    pub utterance: InterlinguaNode,
    pub speech_act: SpeechAct,
    pub intent: Intent,
    pub emotion: Emotion,
}
```

---

## Engine Management API

### Register Language Engine

```rust
impl LexFlexAPI {
    /// Register a new language engine
    pub fn register_engine(
        &mut self,
        engine: Box<dyn IMeaningRepresentation>,
    ) {
        self.translator.register(engine);
    }
    
    /// List available languages
    pub fn available_languages(&self) -> Vec<LanguageInfo> {
        self.translator.list_engines()
    }
    
    /// Get engine capabilities
    pub fn engine_capabilities(
        &self,
        language: &LanguageId,
    ) -> Option<&[Capability]> {
        self.translator.get_engine(language)
            .map(|e| e.capabilities())
    }
}

pub struct LanguageInfo {
    pub id: LanguageId,
    pub name: String,
    pub kind: LanguageKind,
    pub capabilities: Vec<Capability>,
    pub limitations: Vec<Limitation>,
}
```

### Check Translation Feasibility

```rust
impl LexFlexAPI {
    /// Check if translation between two languages is possible
    pub fn can_translate(
        &self,
        from: &LanguageId,
        to: &LanguageId,
    ) -> TranslationFeasibility {
        let source = self.translator.get_engine(from);
        let target = self.translator.get_engine(to);
        
        match (source, target) {
            (Some(_), None) => TranslationFeasibility::UnsupportedTarget(to.clone()),
            (None, _) => TranslationFeasibility::UnsupportedSource(from.clone()),
            (Some(src), Some(tgt)) => {
                // Check capability compatibility
                let src_caps = src.capabilities();
                let tgt_caps = tgt.capabilities();
                
                let missing: Vec<_> = src_caps.iter()
                    .filter(|c| !tgt_caps.contains(c))
                    .cloned()
                    .collect();
                
                if missing.is_empty() {
                    TranslationFeasibility::FullyCompatible
                } else {
                    TranslationFeasibility::PartiallyCompatible {
                        missing_capabilities: missing,
                    }
                }
            }
        }
    }
}

pub enum TranslationFeasibility {
    FullyCompatible,
    PartiallyCompatible {
        missing_capabilities: Vec<Capability>,
    },
    UnsupportedSource(LanguageId),
    UnsupportedTarget(LanguageId),
}
```

---

## Middleware API

### Add Middleware

```rust
impl LexFlexAPI {
    /// Add middleware to the processing pipeline
    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.translator.add_middleware(middleware);
    }
}
```

### Built-in Middleware

See [ENGINE.md](./ENGINE.md#middleware-system) for complete definitions of middleware types:
- `LoggingMiddleware` - logs all processing steps
- `MetricsMiddleware` - tracks performance metrics
- `ProfanityFilter` - filters inappropriate content
- `RateLimiter` - limits API calls

### Example: Using Middleware

```rust
impl LexFlexAPI {
    /// Add middleware to the processing pipeline
    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.translator.add_middleware(middleware);
    }
}

// Usage:
api.add_middleware(Box::new(LoggingMiddleware::new()));
api.add_middleware(Box::new(ProfanityFilter::new()));
```

---

## Future: Memory API (v0.2+)

**⚠️ NOT AVAILABLE IN MVP v0.1**

Memory features require `LongTermMemory` and `Discourse` components (see [Future: Conversation API](#future-conversation-api-v02)).

### Planned Features

#### User Profile

```rust
// Future API (v0.2+)
impl LexFlexAPI {
    /// Get user profile
    pub fn user_profile(&self) -> &UserProfile {
        self.memory.user_profile()
    }

    /// Update user profile
    pub fn update_profile(&mut self, update: ProfileUpdate) {
        self.memory.update_profile(update);
    }

    /// Set user preference
    pub fn set_preference(&mut self, key: PreferenceKey, value: PreferenceValue) {
        self.memory.set_preference(key, value);
    }
}

pub struct ProfileUpdate {
    pub name: Option<String>,
    pub preferred_language: Option<LanguageId>,
    pub preferred_register: Option<Register>,
    pub facts: Vec<Fact>,
}
```

#### Conversation History

```rust
// Future API (v0.2+)
impl LexFlexAPI {
    /// Get conversation history
    pub fn conversation_history(&self) -> &[UtteranceSummary] {
        self.discourse.history()
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.discourse.clear();
    }

    /// Get current discourse topic
    pub fn current_topic(&self) -> Option<&Entity> {
        self.discourse.topic()
    }
}
```

---

## CLI Interface

lexFlex provides a command-line interface for direct usage.

```bash
# Translate
lexflex translate "Tomek dał jabłko Izie" --from pl --to en
# → "Tomek gave Iza an apple"

# Parse
lexflex parse "Tomek dał jabłko Izie" --lang pl --format json
# → { "interlingua": { ... } }

# Analyze
lexflex analyze "Tomek dał jabłko Izie" --lang pl
# → tokens, morphology, syntax tree

# List languages
lexflex languages
# → pl (Polish, Natural), en (English, Natural)

# Check capabilities
lexflex capabilities pl
# → [TemporalReference, Deixis, EmotionExpression, ...]
```

**Note:** Interactive conversation mode (`lexflex converse`) is planned for v0.2+.

---

## Library Usage

### Basic Example

```rust
use lexflex::{LexFlexAPI, LanguageId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize API
    let api = LexFlexAPI::builder()
        .with_polish_engine("data/")?
        .with_english_engine("data/")?
        .with_logging("logs/")
        .build();

    // Simple translation
    let result = api.translate(
        "Tomek dał jabłko Izie",
        &LanguageId::PL,
        &LanguageId::EN,
    )?;

    println!("{}", result.output);
    // → "Tomek gave Iza an apple"

    Ok(())
}
```

### Future: Conversation Example (v0.2+)

**⚠️ NOT AVAILABLE IN MVP v0.1**

```rust
// Future API (v0.2+)
use lexflex::{LexFlexAPI, LanguageId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = LexFlexAPI::builder()
        .with_polish_engine("data/")?
        .build();

    // Multi-turn conversation
    let response1 = api.converse("Cześć!", &LanguageId::PL)?;
    println!("Bot: {}", response1);

    let response2 = api.converse("Jak się masz?", &LanguageId::PL)?;
    println!("Bot: {}", response2);

    let response3 = api.converse("Opowiedz mi o sobie.", &LanguageId::PL)?;
    println!("Bot: {}", response3);

    Ok(())
}
```

### Advanced Example

```rust
use lexflex::{LexFlexAPI, LanguageId, TranslateOptions, Register};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = LexFlexAPI::builder()
        .with_polish_engine("data/")?
        .with_english_engine("data/")?
        .build();

    // Translation with options
    let options = TranslateOptions {
        return_interlingua: true,
        best_effort: true,
        target_register: Some(Register::Formal),
        preserve_formatting: true,
    };

    let result = api.translate_with_options(
        "Cześć, jak się masz?",
        &LanguageId::PL,
        &LanguageId::EN,
        &options,
    )?;

    println!("Translation: {}", result.output);
    // → "Hello, how are you?"

    if let Some(il) = result.interlingua {
        println!("Interlingua: {:?}", il);
    }

    for warning in &result.warnings {
        println!("Warning: {}", warning);
    }

    Ok(())
}
```

---

## Error Handling

All API methods return `Result` types with detailed error information.

```rust
// Parse error with suggestions
match api.parse("Tomek dał xyz Izie", &LanguageId::PL) {
    Ok(result) => println!("Parsed: {:?}", result.interlingua),
    Err(ParseError::UnknownToken { token, suggestions, .. }) => {
        println!("Unknown word: {}", token);
        println!("Did you mean: {:?}", suggestions);
    }
    Err(e) => println!("Error: {}", e),
}

// Translation error with capability mismatch
match api.translate("Tomek się cieszy", &LanguageId::PL, &LanguageId::MATH) {
    Ok(result) => println!("Translated: {}", result.output),
    Err(TranslateError::InexpressibleInTarget { missing_capabilities, .. }) => {
        println!("Cannot translate: target language doesn't support:");
        for cap in &missing_capabilities {
            println!("  - {:?}", cap);
        }
    }
    Err(e) => println!("Error: {}", e),
}
```

---

## Builder Pattern

LexFlexAPI uses the builder pattern for flexible initialization.

```rust
pub struct LexFlexBuilder {
    engines: Vec<Box<dyn IMeaningRepresentation>>,
    middleware: Vec<Box<dyn Middleware>>,
    log_dir: Option<PathBuf>,
}

impl LexFlexBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn with_polish_engine(mut self, data_dir: &str) -> Result<Self> {
        let engine = PolishEngine::load(Path::new(data_dir))?;
        self.engines.push(Box::new(engine));
        Ok(self)
    }

    pub fn with_english_engine(mut self, data_dir: &str) -> Result<Self> {
        let engine = EnglishEngine::load(Path::new(data_dir))?;
        self.engines.push(Box::new(engine));
        Ok(self)
    }

    pub fn with_math_engine(mut self, data_dir: &str) -> Result<Self> {
        let engine = MathEngine::load(Path::new(data_dir))?;
        self.engines.push(Box::new(engine));
        Ok(self)
    }

    pub fn with_logging(mut self, log_dir: &str) -> Self {
        self.log_dir = Some(PathBuf::from(log_dir));
        self
    }

    pub fn with_middleware(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn build(self) -> LexFlexAPI {
        let mut translator = UniversalTranslator::new();

        for engine in self.engines {
            translator.register(engine);
        }

        for mw in self.middleware {
            translator.add_middleware(mw);
        }

        LexFlexAPI {
            translator,
            log: ProcessingLog::new(self.log_dir),
        }
    }
}
```

**Note:** Future v0.2+ will add `with_memory_dir()` and `with_discourse()` builder methods.

---

## Summary

The lexFlex API provides:

1. **Translation** — simple and advanced translation with options
2. **Parsing** — parse text to Interlingua representation
3. **Generation** — generate text from Interlingua
4. **Engine management** — register and query language engines
5. **Middleware** — extensible processing pipeline
6. **CLI** — command-line interface
7. **Builder pattern** — flexible initialization
8. **Detailed errors** — structured error types with suggestions

**Future (v0.2+):**
- **Conversation** — multi-turn dialogue support
- **Memory** — user profile and conversation history
