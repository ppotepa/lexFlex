# Logging Framework — Contextual Per-Request Logging

lexFlex uses **contextual, per-request logging** to track all operations with full traceability. Every request gets a unique ID, and all logs are associated with that ID for easy debugging and analysis.

---

## Core Principles

1. **Per-Request Context**: Every request gets a unique `RequestId` that flows through the entire pipeline
2. **Structured Logging**: All logs are structured (JSON) for easy parsing and analysis
3. **Verbosity Levels**: Different verbosity levels for different use cases
4. **Full Traceability**: Every operation is logged with timing, inputs, outputs, and errors
5. **Context Propagation**: Request context (language, discourse state, etc.) is propagated through all stages

---

## Request ID

Every request gets a unique identifier:

```rust
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        RequestId(Uuid::new_v4().to_string())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Format**: `req_<uuid>` (e.g., `req_550e8400-e29b-41d4-a716-446655440000`)

---

## Request Context

```rust
pub struct RequestContext {
    pub request_id: RequestId,
    pub timestamp: Timestamp,
    pub source_language: LanguageId,
    pub target_language: Option<LanguageId>,
    pub operation: OperationType,
    pub input: String,
    
    // Feature v0.2+ (None in MVP v0.1):
    pub discourse_state: Option<DiscourseState>,
    pub session_id: Option<SessionId>,
    pub user_id: Option<UserId>,
}

pub enum OperationType {
    // MVP v0.1:
    Parse,
    Generate,
    Translate,
    Analyze,
    
    // Feature v0.2+:
    Converse,
}
```

---

## Verbosity Levels

### Level 0: Silent
No logging (production mode).

### Level 1: Errors Only
Log only errors and critical failures.

```rust
log::error!(
    request_id = %ctx.request_id,
    operation = %ctx.operation,
    error = %err,
    "Parse failed"
);
```

### Level 2: Warnings + Errors
Log warnings and errors.

```rust
log::warn!(
    request_id = %ctx.request_id,
    operation = %ctx.operation,
    stage = "morphological_analysis",
    ambiguous_tokens = 2,
    "Morphological ambiguity detected"
);
```

### Level 3: Info + Warnings + Errors
Log high-level pipeline stages (start, end, duration).

```rust
log::info!(
    request_id = %ctx.request_id,
    operation = %ctx.operation,
    stage = "parse",
    duration_ms = 45,
    "Parse completed"
);
```

### Level 4: Debug + Info + Warnings + Errors
Log detailed pipeline data (intermediate results).

```rust
log::debug!(
    request_id = %ctx.request_id,
    stage = "tokenization",
    tokens = ?tokens,
    "Tokenization result"
);
```

### Level 5: Trace + Debug + Info + Warnings + Errors
Log everything (raw data, full pipeline trace).

```rust
log::trace!(
    request_id = %ctx.request_id,
    stage = "morphological_analysis",
    token = "jabłko",
    analyses = ?analyses,
    "Morphological analysis for token"
);
```

---

## Log Structure

All logs are structured JSON:

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "request_id": "req_550e8400-e29b-41d4-a716-446655440000",
  "operation": "translate",
  "stage": "parse",
  "duration_ms": 45,
  "source_language": "pl",
  "target_language": "en",
  "message": "Parse completed",
  "context": {
    "input": "Tomek dał jabłko Izie",
    "discourse_state": {
      "topic": "Tomek",
      "entities": 3
    }
  }
}
```

---

## Pipeline Logging

### Parse Pipeline Logging

```rust
pub fn parse_with_logging(
    input: &str,
    ctx: &RequestContext,
) -> Result<Utterance, ParseError> {
    let logger = Logger::new(ctx);
    
    // Stage 1: Tokenization
    logger.stage_start("tokenization");
    let tokens = tokenize(input);
    logger.stage_complete("tokenization", &tokens);
    
    // Stage 2: Morphological Analysis
    logger.stage_start("morphological_analysis");
    let analyses = analyze_morphology(&tokens);
    logger.stage_complete("morphological_analysis", &analyses);
    
    // Stage 3: Morphological Disambiguation
    logger.stage_start("morphological_disambiguation");
    let disambiguated = disambiguate_morphology(&analyses, ctx.discourse_state.as_ref());
    logger.stage_complete("morphological_disambiguation", &disambiguated);
    
    // Stage 4: Syntactic Parsing
    logger.stage_start("syntactic_parsing");
    let tree = parse_syntax(&disambiguated)?;
    logger.stage_complete("syntactic_parsing", &tree);
    
    // Stage 5: Semantic Role Labeling
    logger.stage_start("semantic_role_labeling");
    let roles = label_semantic_roles(&tree);
    logger.stage_complete("semantic_role_labeling", &roles);
    
    // Stage 6: Frame Assignment
    logger.stage_start("frame_assignment");
    let frames = assign_frames(&roles);
    logger.stage_complete("frame_assignment", &frames);
    
    // Stage 7: Deduction
    logger.stage_start("deduction");
    let utterance = deduce(&frames, ctx.discourse_state.as_ref())?;
    logger.stage_complete("deduction", &utterance);
    
    // Stage 8: Interlingua Construction
    logger.stage_start("interlingua_construction");
    let il = construct_interlingua(&utterance);
    logger.stage_complete("interlingua_construction", &il);
    
    logger.request_complete();
    
    Ok(utterance)
}
```

### Generation Pipeline Logging

```rust
pub fn generate_with_logging(
    il: &Interlingua,
    ctx: &RequestContext,
) -> Result<String, GenerateError> {
    let logger = Logger::new(ctx);
    
    // Stage 1: Generation Planning
    logger.stage_start("generation_planning");
    let plan = plan_generation(il, &ctx.target_language);
    logger.stage_complete("generation_planning", &plan);
    
    // Stage 2: Lexeme Selection
    logger.stage_start("lexeme_selection");
    let lexemes = select_lexemes(il);
    logger.stage_complete("lexeme_selection", &lexemes);
    
    // Stage 3: Case/Position Assignment
    logger.stage_start("case_assignment");
    let cased = assign_cases(&lexemes, il);
    logger.stage_complete("case_assignment", &cased);
    
    // Stage 4: Inflection
    logger.stage_start("inflection");
    let inflected = inflect_all(&cased);
    logger.stage_complete("inflection", &inflected);
    
    // Stage 5: Agreement
    logger.stage_start("agreement");
    let agreed = apply_agreement(&inflected, il);
    logger.stage_complete("agreement", &agreed);
    
    // Stage 6: Word Order
    logger.stage_start("word_order");
    let ordered = apply_word_order(&agreed, il);
    logger.stage_complete("word_order", &ordered);
    
    // Stage 7: Article Insertion
    logger.stage_start("article_insertion");
    let with_articles = insert_articles(&ordered, il);
    logger.stage_complete("article_insertion", &with_articles);
    
    // Stage 8: Phonological/Orthographic Rules
    logger.stage_start("phonological_rules");
    let final_form = apply_phonological_rules(&with_articles);
    logger.stage_complete("phonological_rules", &final_form);
    
    logger.request_complete();
    
    Ok(final_form)
}
```

---

## Logger Implementation

```rust
pub struct Logger {
    ctx: RequestContext,
    start_time: Instant,
    stage_times: HashMap<String, Instant>,
}

impl Logger {
    pub fn new(ctx: &RequestContext) -> Self {
        Logger {
            ctx: ctx.clone(),
            start_time: Instant::now(),
            stage_times: HashMap::new(),
        }
    }
    
    pub fn stage_start(&self, stage: &str) {
        self.stage_times.insert(stage.to_string(), Instant::now());
        
        log::trace!(
            request_id = %self.ctx.request_id,
            stage = stage,
            "Stage started"
        );
    }
    
    pub fn stage_complete<T: Serialize>(&self, stage: &str, result: &T) {
        let duration = self.stage_times
            .get(stage)
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        
        log::debug!(
            request_id = %self.ctx.request_id,
            stage = stage,
            duration_ms = duration,
            result = %serde_json::to_string(result).unwrap(),
            "Stage completed"
        );
    }
    
    pub fn stage_error(&self, stage: &str, error: &dyn std::error::Error) {
        let duration = self.stage_times
            .get(stage)
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        
        log::error!(
            request_id = %self.ctx.request_id,
            stage = stage,
            duration_ms = duration,
            error = %error,
            "Stage failed"
        );
    }
    
    pub fn request_complete(&self) {
        let total_duration = self.start_time.elapsed().as_millis();
        
        log::info!(
            request_id = %self.ctx.request_id,
            operation = %self.ctx.operation,
            total_duration_ms = total_duration,
            "Request completed"
        );
    }
}
```

---

## Log Output Formats

### Console Output (Development)

```
[2024-01-15T10:30:45.123Z] INFO  req_550e8400-e29b-41d4-a716-446655440000 | translate | parse | 45ms | Parse completed
[2024-01-15T10:30:45.078Z] DEBUG req_550e8400-e29b-41d4-a716-446655440000 | tokenization | 2ms | tokens: ["Tomek", "dał", "jabłko", "Izie"]
[2024-01-15T10:30:45.080Z] DEBUG req_550e8400-e29b-41d4-a716-446655440000 | morphological_analysis | 5ms | analyses: [...]
[2024-01-15T10:30:45.085Z] DEBUG req_550e8400-e29b-41d4-a716-446655440000 | morphological_disambiguation | 3ms | disambiguated: [...]
```

### JSON Output (Production)

```json
{"timestamp":"2024-01-15T10:30:45.123Z","level":"INFO","request_id":"req_550e8400-e29b-41d4-a716-446655440000","operation":"translate","stage":"parse","duration_ms":45,"message":"Parse completed"}
{"timestamp":"2024-01-15T10:30:45.078Z","level":"DEBUG","request_id":"req_550e8400-e29b-41d4-a716-446655440000","stage":"tokenization","duration_ms":2,"tokens":["Tomek","dał","jabłko","Izie"]}
```

### File Output (Audit)

Logs are written to files with rotation:

```
logs/
├── 2024-01-15/
│   ├── req_550e8400-e29b-41d4-a716-446655440000.log
│   ├── req_660e8400-e29b-41d4-a716-446655440001.log
│   └── ...
└── ...
```

Each request gets its own log file for easy debugging.

---

## Error Logging

Errors are logged with full context:

```rust
log::error!(
    request_id = %ctx.request_id,
    operation = %ctx.operation,
    stage = "syntactic_parsing",
    error = %err,
    error_chain = ?err.chain().collect::<Vec<_>>(),
    input = %ctx.input,
    discourse_state = ?ctx.discourse_state,
    "Syntactic parsing failed"
);
```

---

## Performance Metrics

Performance metrics are logged for each stage:

```rust
pub struct StageMetrics {
    pub stage: String,
    pub duration_ms: u64,
    pub memory_bytes: u64,
    pub allocations: u64,
}

pub struct RequestMetrics {
    pub request_id: RequestId,
    pub total_duration_ms: u64,
    pub stages: Vec<StageMetrics>,
    pub peak_memory_bytes: u64,
}
```

---

## Configuration

```rust
pub struct LoggingConfig {
    pub verbosity: VerbosityLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_rotation: Option<FileRotationConfig>,
}

pub enum VerbosityLevel {
    Silent,
    ErrorsOnly,
    Warnings,
    Info,
    Debug,
    Trace,
}

pub enum LogFormat {
    Text,
    Json,
}

pub enum LogOutput {
    Console,
    File(PathBuf),
    Both,
}

pub struct FileRotationConfig {
    pub max_size_mb: u64,
    pub max_files: usize,
    pub compress: bool,
}
```

---

## Usage Example

```rust
use lexflex::logging::{Logger, RequestContext, RequestId, OperationType};

fn main() {
    // Initialize logging
    lexflex::logging::init(LoggingConfig {
        verbosity: VerbosityLevel::Debug,
        format: LogFormat::Json,
        output: LogOutput::Both,
        file_rotation: Some(FileRotationConfig {
            max_size_mb: 100,
            max_files: 10,
            compress: true,
        }),
    });
    
    // Create request context
    let ctx = RequestContext {
        request_id: RequestId::new(),
        timestamp: Timestamp::now(),
        source_language: LanguageId::PL,
        target_language: Some(LanguageId::EN),
        operation: OperationType::Translate,
        input: "Tomek dał jabłko Izie".to_string(),
        discourse_state: None,
        session_id: None,
        user_id: None,
    };
    
    // Parse with logging
    let result = parse_with_logging(&ctx.input, &ctx);
    
    match result {
        Ok(utterance) => {
            log::info!(
                request_id = %ctx.request_id,
                "Translation successful"
            );
        }
        Err(err) => {
            log::error!(
                request_id = %ctx.request_id,
                error = %err,
                "Translation failed"
            );
        }
    }
}
```

---

## Summary

lexFlex logging provides:

1. **Per-Request Context**: Unique `RequestId` for every request
2. **Structured Logging**: JSON format for easy parsing
3. **Verbosity Levels**: 6 levels from Silent to Trace
4. **Pipeline Stages**: Every stage is logged with timing
5. **Error Context**: Full error chain and context
6. **Performance Metrics**: Duration, memory, allocations
7. **Multiple Outputs**: Console, file, or both
8. **File Rotation**: Automatic rotation and compression
