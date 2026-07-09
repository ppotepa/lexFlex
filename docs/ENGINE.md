# Engine — Language Plugin Architecture

The Engine system is a **layered abstraction architecture** that supports multiple language types: natural languages (Polish, English), formal languages (mathematics, logic), and programming languages (Python, SQL). Each language is a **plugin** that implements the appropriate trait layer.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 0: Core Protocol                    │
│         IMeaningRepresentation (Universal Trait)            │
│   - to_interlingua(input) → Result<Interlingua>            │
│   - from_interlingua(il) → Result<output>                  │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Layer 1    │    │   Layer 2    │    │   Layer 3    │
│  INatural    │    │   IFormal    │    │ IProgramming │
│  Language    │    │   Language   │    │   Language   │
│              │    │              │    │              │
│ • parse()    │    │ • parse_expr │    │ • parse_code │
│ • generate() │    │ • render     │    │ • generate   │
│ • inflect()  │    │ • validate   │    │ • transpile  │
│ • descriptor │    │ • grammar    │    │ • AST        │
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    Polish    │    │  Mathematics │    │    Python    │
│    English   │    │    Logic     │    │      SQL     │
│    German    │    │              │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
```

## Why Layered Abstraction?

Different language types have fundamentally different structures:

- **Natural languages** have morphology, syntax, pragmatics, discourse
- **Formal languages** have grammar, symbols, proofs, but no morphology
- **Programming languages** have AST, semantics, execution, but no pragmatics

A single `ILanguageEngine` trait would force all languages into the same mold. Layered abstraction lets each language type expose its **native capabilities** while maintaining a **universal protocol** (Interlingua) for translation.

---

## Layer 0: IMeaningRepresentation (Universal Protocol)

The most abstract trait — every language plugin implements this. It defines the minimal contract for any language to participate in the Interlingua ecosystem.

```rust
pub trait IMeaningRepresentation {
    /// What type of input this language accepts
    type Input;
    
    /// What type of output this language produces
    type Output;
    
    /// Language identifier: "pl", "en", "math", "python", "sql"
    fn language_id(&self) -> &LanguageId;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// What kind of language this is
    fn language_kind(&self) -> LanguageKind;
    
    /// What this language can express
    fn capabilities(&self) -> &[Capability];
    
    /// What this language cannot express
    fn limitations(&self) -> &[Limitation];
    
    /// Convert input → Interlingua (universal meaning representation)
    fn to_interlingua(
        &self,
        input: &Self::Input,
    ) -> Result<Interlingua, ParseError>;
    
    /// Convert Interlingua → output in this language
    fn from_interlingua(
        &self,
        il: &Interlingua,
    ) -> Result<Self::Output, GenerateError>;
    
    /// Check if this language can express the given Interlingua
    fn can_express(&self, il: &Interlingua) -> Vec<InexpressibleFeature> {
        let required = il.required_capabilities();
        let available = self.capabilities();
        
        required.iter()
            .filter(|cap| !available.contains(cap))
            .map(|cap| InexpressibleFeature {
                capability: *cap,
                suggestion: self.suggest_workaround(cap),
            })
            .collect()
    }
}
```

### Language Kinds

```rust
pub enum LanguageKind {
    Natural,      // Polish, English, German, etc.
    Formal,       // Mathematics, Logic, Chemistry notation
    Programming,  // Python, SQL, Rust, etc.
    Domain,       // Custom domain-specific languages
}
```

### Capabilities and Limitations

```rust
pub enum Capability {
    // ─── Natural Language ───────────────────────
    TemporalReference,        // "wczoraj", "tomorrow"
    Deixis,                   // "this", "here", "now"
    EmotionExpression,        // emotions, sentiment
    Pragmatics,               // register, formality, politeness
    ProDrop,                  // subject omission
    FreeWordOrder,            // flexible syntax
    MorphologicalInflection,  // cases, conjugation
    
    // ─── Formal Language ────────────────────────
    Quantification,           // ∀, ∃
    FormalProof,              // theorem proving
    NumericPrecision,         // exact numbers
    SetTheory,                // sets, subsets, unions
    LogicalConnectives,       // ∧, ∨, ¬, →, ↔
    
    // ─── Programming Language ───────────────────
    Procedures,               // functions, methods
    ControlFlow,              // if/else, loops
    SideEffects,              // I/O, state mutation
    TypeSystem,               // types, generics
    ErrorHandling,            // try/catch, Result
    
    // ─── Universal ──────────────────────────────
    Negation,                 // negation in any form
    Coordination,             // and, or, but
    Conditionality,           // if...then
    Reference,                // referring to entities
    Ambiguity,                // intentional ambiguity (natural lang feature!)
}

pub enum Limitation {
    NoEmotionExpression,      // math, code
    NoDeixis,                 // math, code
    NoFormalProofs,           // natural languages
    NoQuantification,         // most programming languages
    NoAmbiguity,              // formal languages (by design)
    NoProcedures,             // natural languages, math
    NoProDrop,                // English, math, code
}
```

### Capability Matrix

```
                        PL   EN   Math   Python  SQL
TemporalReference       ✅   ✅   ❌     ❌      ❌
Deixis                  ✅   ✅   ❌     ❌      ❌
EmotionExpression       ✅   ✅   ❌     ❌      ❌
Pragmatics              ✅   ✅   ❌     ❌      ❌
ProDrop                 ✅   ❌   ❌     ❌      ❌
FreeWordOrder           ✅   ❌   ❌     ❌      ❌
MorphologicalInflection ✅   ⚠️   ❌     ❌      ❌
Quantification          ⚠️   ⚠️   ✅     ❌      ❌
FormalProof             ❌   ❌   ✅     ⚠️      ❌
NumericPrecision        ⚠️   ⚠️   ✅     ✅      ✅
SetTheory               ❌   ❌   ✅     ⚠️      ❌
LogicalConnectives      ⚠️   ⚠️   ✅     ⚠️      ⚠️
Procedures              ❌   ❌   ❌     ✅      ❌
ControlFlow             ❌   ❌   ❌     ✅      ⚠️
SideEffects             ❌   ❌   ❌     ✅      ❌
Negation                ✅   ✅   ✅     ✅      ✅
Coordination            ✅   ✅   ✅     ✅      ✅
Conditionality          ✅   ✅   ✅     ✅      ✅
Ambiguity               ✅   ✅   ❌     ❌      ❌
```

---

## Layer 1: INaturalLanguage

For natural languages with morphology, syntax, and pragmatics.

```rust
pub trait INaturalLanguage: IMeaningRepresentation<Input = str, Output = String> {
    // ─── Configuration ─────────────────────────
    fn descriptor(&self) -> &LanguageDescriptor;
    fn lexicon(&self) -> &SubLexicon;
    fn morphology(&self) -> &MorphologySystem;
    fn ontology(&self) -> &Ontology;
    
    // ─── Parse Pipeline ────────────────────────
    fn parse(
        &self,
        input: &str,
        discourse: Option<&Discourse>,  // None in MVP v0.1, Some in v0.2+
    ) -> Result<Utterance, ParseError>;

    /// Tokenize input into tokens
    fn tokenize(&self, input: &str) -> Vec<Token>;

    /// Morphological analysis
    fn analyze_morphology(&self, tokens: &[Token]) -> Vec<MorphAnalysis>;

    /// Syntactic parse
    fn parse_syntax(&self, analyzed: &[MorphAnalysis]) -> Result<SyntaxTree, ParseError>;

    /// Semantic frame assignment
    fn assign_frames(&self, tree: &SyntaxTree) -> Vec<Frame>;

    /// Deduction (resolve ambiguity, fill implicit info)
    fn deduce(
        &self,
        frames: &[Frame],
        discourse: Option<&Discourse>,  // None in MVP v0.1, Some in v0.2+
    ) -> Result<Utterance, DeductionError>;
    
    // ─── Generation Pipeline ───────────────────
    fn generate(
        &self,
        utterance: &Utterance,
    ) -> Result<String, GenerateError>;
    
    /// Select lexemes for concepts
    fn select_lexemes(&self, il: &Interlingua) -> Vec<Lexeme>;
    
    /// Assign grammatical cases/positions
    fn assign_cases(&self, lexemes: &[Lexeme], il: &Interlingua) -> Vec<CasedLexeme>;
    
    /// Inflect all forms
    fn inflect_all(&self, cased: &[CasedLexeme]) -> Vec<String>;
    
    /// Apply word order
    fn apply_word_order(&self, forms: &[String], il: &Interlingua) -> String;
    
    // ─── Morphology ────────────────────────────
    fn inflect(
        &self,
        lemma: &str,
        features: &FeatureBundle,
    ) -> Result<String, MorphError>;
    
    fn analyze(
        &self,
        form: &str,
    ) -> Result<Vec<AnalysisCandidate>, MorphError>;
    
    // ─── Lexicon Access ────────────────────────
    fn lookup_concept(&self, concept: &ConceptId) -> Option<&LexEntry>;
    fn lookup_form(&self, form: &str) -> Vec<&LexEntry>;
    
    // ─── Default Implementations ───────────────

    /// Bridge to IMeaningRepresentation
    /// 
    /// Pipeline: parse → deduction → InterlinguaNode
    /// See [DEDUCTION.md](./DEDUCTION.md) for ambiguity resolution details.
    fn to_interlingua(&self, input: &str) -> Result<Interlingua, ParseError> {
        // MVP v0.1: discourse is None
        // Feature v0.2+: discourse will be passed from LexFlexAPI
        let utterance = self.parse(input, None)?;
        // parse() internally calls deduction to resolve ambiguities
        Ok(Interlingua::Natural(utterance))
    }

    fn from_interlingua(&self, il: &Interlingua) -> Result<String, GenerateError> {
        match il {
            Interlingua::Natural(utterance) => self.generate(utterance),
            Interlingua::MathExpression(math_expr) => {
                // Convert mathematical expression → natural language description
                let utterance = self.math_to_natural(math_expr)?;
                self.generate(&utterance)
            }
            Interlingua::LogicalProposition(logical_expr) => {
                // Convert logical proposition → natural language description
                let utterance = self.logic_to_natural(logical_expr)?;
                self.generate(&utterance)
            }
            Interlingua::ProgramStatement(program_stmt) => {
                // Convert program statement → natural language description
                let utterance = self.program_to_natural(program_stmt)?;
                self.generate(&utterance)
            }
            _ => Err(GenerateError::UnsupportedInterlinguaType),
        }
    }
}
```

---

## Complete Parse Pipeline

The full parsing pipeline from raw input to Interlingua representation:

```
Input Text
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 1: TOKENIZATION                                       │
│                                                             │
│ Split input into tokens (words, punctuation, whitespace)    │
│                                                             │
│ "Tomek dał jabłko Izie"                                    │
│ → [Tomek, dał, jabłko, Izie]                               │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 2: MORPHOLOGICAL ANALYSIS                             │
│                                                             │
│ For each token, find all possible analyses (lemma + features)│
│                                                             │
│ "Tomek" → [Lemma("Tomek", NOUN, NOM, M, SG)]              │
│ "dał"   → [Lemma("dać", VERB, PAST, 3SG, M, PERF)]        │
│ "jabłko"→ [Lemma("jabłko", NOUN, NOM, N, SG),             │
│            Lemma("jabłko", NOUN, ACC, N, SG)]  ← ambiguous│
│ "Izie"  → [Lemma("Iza", NOUN, DAT, F, SG)]                │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 3: MORPHOLOGICAL DISAMBIGUATION                       │
│                                                             │
│ Use context to select correct analysis for ambiguous tokens │
│                                                             │
│ "jabłko" is ambiguous (NOM or ACC)                          │
│                                                             │
│ Strategy 1: Verb subcategorization                          │
│   "dać" requires [agent:NOM, theme:ACC, recipient:DAT]      │
│   → "jabłko" must be ACC (theme)                            │
│                                                             │
│ Strategy 2: Word order heuristics                           │
│   Pre-verbal position → prefer NOM (subject)                │
│   Post-verbal position → prefer ACC (object)                │
│                                                             │
│ Strategy 3: Agreement with adjacent words                   │
│   "duże jabłko" → "duże" is ACC → "jabłko" must be ACC    │
│                                                             │
│ Result: "jabłko" → ACC (disambiguated)                      │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 4: SYNTACTIC PARSING                                  │
│                                                             │
│ Build syntactic structure (constituency or dependency tree) │
│                                                             │
│ Dependency Tree:                                            │
│   dał (ROOT)                                                │
│   ├── Tomek (Subject)                                       │
│   ├── jabłko (Object)                                       │
│   └── Izie (IndirectObject)                                 │
│                                                             │
│ OR Constituency Tree:                                       │
│   S                                                         │
│   ├── NP (Tomek)                                            │
│   └── VP                                                    │
│       ├── V (dał)                                           │
│       ├── NP (jabłko)                                       │
│       └── NP (Izie)                                         │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 5: SEMANTIC ROLE LABELING                             │
│                                                             │
│ Map syntactic positions to semantic roles                   │
│                                                             │
│ Subject → Agent (for transitive verbs)                      │
│ Direct Object → Theme                                       │
│ Indirect Object → Recipient                                 │
│                                                             │
│ Result:                                                     │
│   Tomek → Agent                                             │
│   jabłko → Theme                                            │
│   Izie → Recipient                                          │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 6: FRAME ASSIGNMENT                                   │
│                                                             │
│ Select appropriate frame based on verb and roles            │
│                                                             │
│ Verb "dać" + [Agent, Theme, Recipient] → Frame::Transfer   │
│                                                             │
│ Frame::Transfer {                                           │
│   agent: Entity { concept: PERSON, name: "Tomek" },        │
│   recipient: Entity { concept: PERSON, name: "Iza" },      │
│   theme: Entity { concept: APPLE },                        │
│ }                                                           │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 7: DEDUCTION                                          │
│ (see DEDUCTION.md for full specification)                   │
│                                                             │
│ Resolve ambiguity, fill implicit information                │
│                                                             │
│ • Tense: PAST (from verb morphology)                        │
│ • Aspect: PERFECTIVE (from verb form)                       │
│ • Polarity: POSITIVE (no negation)                          │
│ • Modality: REALIS (factual)                                │
│ • Illocution: STATEMENT (declarative)                       │
│ • Reference resolution: resolve pronouns from discourse     │  ← Feature v0.2+
│ • Temporal resolution: resolve temporal adverbs             │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 8: INTERLINGUA CONSTRUCTION                           │
│                                                             │
│ Build complete Interlingua representation                   │
│                                                             │
│ Utterance {                                                 │
│   discourse: { speaker, addressee, topic, ... },           │  ← Feature v0.2+
│   sentences: [                                              │
│     Sentence {                                              │
│       frames: [Frame::Transfer { ... }],                   │
│       tense: Some(Past),                                    │
│       aspect: Some(Perfective),                             │
│       polarity: Positive,                                   │
│       illocution: Statement,                                │
│       topic: Some(EntityId(0)),                             │
│       focus: None,                                          │
│     }                                                       │
│   ]                                                         │
│ }                                                           │
│                                                             │
│ Note: In MVP v0.1, discourse is None and Utterance contains │
│ only sentences without discourse context.                   │
└─────────────────────────────────────────────────────────────┘
```

---

## Complete Generation Pipeline

The full generation pipeline from Interlingua to surface text:

```
Interlingua
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 1: GENERATION PLANNING                                │
│                                                             │
│ Decide HOW to express the Interlingua in target language    │
│                                                             │
│ • Word order (SVO, SOV, VSO, etc.)                          │
│ • Voice (active vs passive)                                 │
│ • Pro-drop (omit subject if language allows)                │
│ • Articles (add definite/indefinite articles if needed)     │
│ • Tense/aspect mapping (IL tense → target tense)            │
│ • Information structure (topic first, focus last)           │
│                                                             │
│ Example (EN):                                               │
│   Word order: SVO                                           │
│   Voice: Active                                             │
│   Pro-drop: No (EN requires subject)                        │
│   Articles: Yes (EN requires articles)                      │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 2: LEXEME SELECTION                                   │
│                                                             │
│ Select words for each concept in Interlingua                │
│                                                             │
│ PERSON (Tomek) → "Tomek"                                    │
│ GIVE + PAST + PERFECTIVE → "gave"                           │
│ APPLE → "apple"                                             │
│ PERSON (Iza) → "Iza"                                        │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 3: CASE/POSITION ASSIGNMENT                           │
│                                                             │
│ Determine grammatical case or syntactic position            │
│                                                             │
│ Agent → Subject position                                    │
│ Recipient → Indirect object                                 │
│ Theme → Direct object                                       │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 4: INFLECTION                                         │
│                                                             │
│ Apply morphological rules to produce correct word forms     │
│                                                             │
│ "give" + PAST → "gave"                                      │
│ "apple" + SG → "apple"                                      │
│ "Tomek" + NOM → "Tomek"                                     │
│ "Iza" + DAT → "Iza" (EN doesn't inflect for case)          │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 5: AGREEMENT                                          │
│                                                             │
│ Ensure agreement between related words                      │
│                                                             │
│ Adjective-Noun agreement (gender, number, case)             │
│ Subject-Verb agreement (person, number)                     │
│                                                             │
│ Example: "duży dom" → "duży" agrees with "dom" (M, SG, NOM)│
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 6: WORD ORDER                                         │
│                                                             │
│ Arrange words according to target language rules            │
│                                                             │
│ EN: SVO → [Subject] [Verb] [IndirectObject] [DirectObject] │
│                                                             │
│ [Tomek] [gave] [Iza] [an apple]                            │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 7: ARTICLE INSERTION                                  │
│                                                             │
│ Add articles if target language requires them               │
│                                                             │
│ EN: "apple" → "an apple" (indefinite, starts with vowel)   │
│ PL: no articles needed                                      │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 8: PHONOLOGICAL/ORTHOGRAPHIC RULES                    │
│                                                             │
│ Apply phonological and orthographic rules                   │
│                                                             │
│ • Capitalization (first word of sentence)                   │
│ • Punctuation (period, question mark, exclamation)          │
│ • Contraction (EN: "do not" → "don't")                      │
│ • Sandhi (sound changes at word boundaries, some languages) │
│                                                             │
│ Result: "Tomek gave Iza an apple."                          │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ STAGE 9: SURFACE REALIZATION                                │
│                                                             │
│ Final output string                                         │
│                                                             │
│ "Tomek gave Iza an apple."                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer 2: IFormalLanguage (Future)

For formal languages: mathematics, logic, chemistry notation.

```rust
pub trait IFormalLanguage: IMeaningRepresentation<Input = str, Output = String> {
    // ─── Configuration ─────────────────────────
    fn grammar(&self) -> &FormalGrammar;
    fn symbols(&self) -> &SymbolTable;
    
    // ─── Parse ─────────────────────────────────
    fn parse_expression(&self, expr: &str) -> Result<Interlingua, ParseError>;
    
    /// Validate expression syntax
    fn validate_syntax(&self, expr: &str) -> Result<bool, ValidationError>;
    
    /// Validate expression semantics (type-check)
    fn validate_semantics(&self, expr: &str) -> Result<bool, ValidationError>;
    
    // ─── Render ────────────────────────────────
    fn render_expression(&self, il: &Interlingua) -> Result<String, GenerateError>;
    
    /// Render to LaTeX
    fn to_latex(&self, il: &Interlingua) -> Result<String, GenerateError>;
    
    /// Render to Unicode math symbols
    fn to_unicode(&self, il: &Interlingua) -> Result<String, GenerateError>;
    
    // ─── Evaluation ────────────────────────────
    /// Evaluate a numeric expression
    fn evaluate(&self, expr: &str) -> Result<Value, EvalError>;
    
    /// Simplify an expression
    fn simplify(&self, expr: &str) -> Result<String, EvalError>;
    
    // ─── Default Implementations ───────────────
    fn to_interlingua(&self, input: &str) -> Result<Interlingua, ParseError> {
        self.parse_expression(input)
    }
    
    fn from_interlingua(&self, il: &Interlingua) -> Result<String, GenerateError> {
        self.render_expression(il)
    }
}
```

### Formal Grammar

```rust
pub struct FormalGrammar {
    /// Production rules
    pub productions: Vec<Production>,
    
    /// Terminal symbols
    pub terminals: HashSet<String>,
    
    /// Non-terminal symbols
    pub non_terminals: HashSet<String>,
    
    /// Start symbol
    pub start: String,
}

pub struct SymbolTable {
    /// Known symbols and their meanings
    pub symbols: HashMap<String, SymbolDefinition>,
    
    /// Operator precedence
    pub precedence: Vec<Vec<String>>,
    
    /// Operator associativity
    pub associativity: HashMap<String, Associativity>,
}

pub struct SymbolDefinition {
    pub name: String,
    pub category: SymbolCategory,
    pub arity: Arity,
    pub interlingua_mapping: ConceptId,
}

pub enum SymbolCategory {
    Operator,      // +, -, *, /
    Relation,      // =, <, >, ≤, ≥
    Function,      // sin, cos, log
    Constant,      // π, e
    Quantifier,    // ∀, ∃
    Connective,    // ∧, ∨, ¬, →, ↔
    SetOperator,   // ∈, ⊂, ∪, ∩
}
```

---

## Layer 3: IProgrammingLanguage (Future)

For programming languages: Python, SQL, Rust, etc.

```rust
pub trait IProgrammingLanguage: IMeaningRepresentation<Input = str, Output = String> {
    // ─── Configuration ─────────────────────────
    fn ast_parser(&self) -> &dyn AstParser;
    fn semantics(&self) -> &LanguageSemantics;
    
    // ─── Parse ─────────────────────────────────
    fn parse_code(&self, code: &str) -> Result<Interlingua, ParseError>;
    
    /// Parse to AST (language-specific)
    fn parse_to_ast(&self, code: &str) -> Result<Ast, ParseError>;
    
    /// Convert AST to Interlingua
    fn ast_to_interlingua(&self, ast: &Ast) -> Result<Interlingua, ParseError>;
    
    // ─── Generate ──────────────────────────────
    fn generate_code(&self, il: &Interlingua) -> Result<String, GenerateError>;
    
    /// Format generated code
    fn format_code(&self, code: &str) -> Result<String, FormatError>;
    
    // ─── Transpilation ─────────────────────────
    /// Translate to another programming language
    fn transpile_to(
        &self,
        il: &Interlingua,
        target: &dyn IProgrammingLanguage,
    ) -> Result<String, TranspileError> {
        // il is already language-neutral
        // target generates its own syntax
        target.generate_code(il)
    }
    
    // ─── Analysis ──────────────────────────────
    /// Extract function signatures
    fn extract_signatures(&self, code: &str) -> Vec<FunctionSignature>;
    
    /// Detect patterns (e.g., "this is a loop", "this is recursion")
    fn detect_patterns(&self, ast: &Ast) -> Vec<CodePattern>;
    
    // ─── Default Implementations ───────────────
    fn to_interlingua(&self, input: &str) -> Result<Interlingua, ParseError> {
        self.parse_code(input)
    }
    
    fn from_interlingua(&self, il: &Interlingua) -> Result<String, GenerateError> {
        self.generate_code(il)
    }
}
```

### AST Abstraction

```rust
/// Language-neutral AST representation
pub enum Ast {
    // Expressions
    Literal(Literal),
    Variable(String),
    BinaryOp { op: String, lhs: Box<Ast>, rhs: Box<Ast> },
    UnaryOp { op: String, operand: Box<Ast> },
    FunctionCall { name: String, args: Vec<Ast> },
    
    // Statements
    Assignment { target: String, value: Box<Ast> },
    If { condition: Box<Ast>, then_branch: Box<Ast>, else_branch: Option<Box<Ast>> },
    Loop { kind: LoopKind, body: Box<Ast> },
    Return(Option<Box<Ast>>),
    
    // Definitions
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        body: Box<Ast>,
        return_type: Option<String>,
    },
    
    // Blocks
    Block(Vec<Ast>),
}

pub enum LoopKind {
    For { variable: String, iterable: Box<Ast> },
    While { condition: Box<Ast> },
    Repeat { count: Box<Ast> },
}
```

---

## Universal Translator

The translator orchestrates all engines and handles cross-language translation.

```rust
pub struct UniversalTranslator {
    /// All registered language engines
    engines: HashMap<LanguageId, Box<dyn IMeaningRepresentation>>,
    
    /// Shared ontology
    ontology: Ontology,
    
    /// Middleware pipeline
    middleware: Vec<Box<dyn Middleware>>,
}

impl UniversalTranslator {
    /// Register a language engine
    pub fn register(&mut self, engine: Box<dyn IMeaningRepresentation>) {
        let id = engine.language_id().clone();
        self.engines.insert(id, engine);
    }
    
    /// Translate from any language to any language
    pub fn translate(
        &self,
        input: &str,
        from: &LanguageId,
        to: &LanguageId,
    ) -> Result<String, TranslateError> {
        let source = self.engines.get(from)
            .ok_or(TranslateError::UnsupportedLanguage(from.clone()))?;
        let target = self.engines.get(to)
            .ok_or(TranslateError::UnsupportedLanguage(to.clone()))?;
        
        // Apply pre-parse middleware
        let input = self.apply_before_parse(input);
        
        // 1. Source → Interlingua
        let il = source.to_interlingua(&input)?;
        
        // Apply post-parse middleware
        let il = self.apply_after_parse(il);
        
        // 2. Check if target can express this Interlingua
        let inexpressible = target.can_express(&il);
        if !inexpressible.is_empty() {
            return Err(TranslateError::InexpressibleInTarget {
                target: to.clone(),
                features: inexpressible,
            });
        }
        
        // 3. Interlingua → Target
        let output = target.from_interlingua(&il)?;
        
        // Apply post-generate middleware
        let output = self.apply_after_generate(&output);
        
        Ok(output)
    }
    
    /// Translate with graceful degradation
    pub fn translate_best_effort(
        &self,
        input: &str,
        from: &LanguageId,
        to: &LanguageId,
    ) -> TranslateResult {
        match self.translate(input, from, to) {
            Ok(output) => TranslateResult::Complete(output),
            Err(TranslateError::InexpressibleInTarget { features, .. }) => {
                // Try with lossy conversion
                let lossy_il = self.make_lossy(&il, to, &features);
                match target.from_interlingua(&lossy_il) {
                    Ok(output) => TranslateResult::Lossy {
                        output,
                        lost_features: features,
                    },
                    Err(e) => TranslateResult::Failed(e),
                }
            }
            Err(e) => TranslateResult::Failed(e),
        }
    }
}
```

---

## Polish Engine Implementation (Layer 1)

```rust
pub struct PolishEngine {
    descriptor: LanguageDescriptor,
    lexicon: SubLexicon,
    morphology: MorphologySystem,
    ontology: Ontology,
}

impl PolishEngine {
    pub fn load(data_dir: &Path) -> Result<Self, LoadError> {
        let descriptor = load_ron::<LanguageDescriptor>(
            data_dir.join("descriptors/pl.ron")
        )?;
        let lexicon = load_ron::<SubLexicon>(
            data_dir.join("lexicons/pl/lexicon.ron")
        )?;
        let morphology = MorphologySystem::load_pl(data_dir)?;
        let ontology = load_ron::<Ontology>(
            data_dir.join("ontology/ontology.ron")
        )?;
        
        Ok(Self { descriptor, lexicon, morphology, ontology })
    }
}

impl IMeaningRepresentation for PolishEngine {
    type Input = str;
    type Output = String;
    
    fn language_id(&self) -> &LanguageId { &LanguageId::PL }
    fn name(&self) -> &str { "Polish" }
    fn language_kind(&self) -> LanguageKind { LanguageKind::Natural }
    
    fn capabilities(&self) -> &[Capability] {
        &[
            Capability::TemporalReference,
            Capability::Deixis,
            Capability::EmotionExpression,
            Capability::Pragmatics,
            Capability::ProDrop,
            Capability::FreeWordOrder,
            Capability::MorphologicalInflection,
            Capability::Negation,
            Capability::Coordination,
            Capability::Conditionality,
            Capability::Reference,
            Capability::Ambiguity,
        ]
    }
    
    fn limitations(&self) -> &[Limitation] {
        &[
            Limitation::NoFormalProofs,
            Limitation::NoQuantification,  // partial — has some quantifiers
            Limitation::NoProcedures,
            Limitation::NoNumericPrecision,
        ]
    }
    
    fn to_interlingua(&self, input: &Self::Input) -> Result<Interlingua, ParseError> {
        // MVP v0.1: discourse is None
        // Feature v0.2+: discourse will be passed from LexFlexAPI
        let utterance = self.parse(input, None)?;
        Ok(Interlingua::Natural(utterance))
    }
    
    fn from_interlingua(&self, il: &Interlingua) -> Result<Self::Output, GenerateError> {
        match il {
            Interlingua::Natural(utterance) => self.generate(utterance),
            _ => Err(GenerateError::UnsupportedInterlinguaType),
        }
    }
}

impl INaturalLanguage for PolishEngine {
    fn descriptor(&self) -> &LanguageDescriptor { &self.descriptor }
    fn lexicon(&self) -> &SubLexicon { &self.lexicon }
    fn morphology(&self) -> &MorphologySystem { &self.morphology }
    fn ontology(&self) -> &Ontology { &self.ontology }

    fn parse(
        &self,
        input: &str,
        discourse: Option<&Discourse>,  // None in MVP v0.1, Some in v0.2+
    ) -> Result<Utterance, ParseError> {
        let tokens = self.tokenize(input);
        let analyzed = self.analyze_morphology(&tokens);
        let tree = self.parse_syntax(&analyzed)?;
        let frames = self.assign_frames(&tree);
        let utterance = self.deduce(&frames, discourse)?;
        Ok(utterance)
    }
    
    fn generate(
        &self,
        utterance: &Utterance,
    ) -> Result<String, GenerateError> {
        let il = Interlingua::Natural(utterance.clone());
        let lexemes = self.select_lexemes(&il);
        let cased = self.assign_cases(&lexemes, &il);
        let inflected = self.inflect_all(&cased);
        let surface = self.apply_word_order(&inflected, &il);
        Ok(surface)
    }
    
    fn inflect(
        &self,
        lemma: &str,
        features: &FeatureBundle,
    ) -> Result<String, MorphError> {
        self.morphology.inflect(lemma, features)
    }
    
    fn analyze(
        &self,
        form: &str,
    ) -> Result<Vec<AnalysisCandidate>, MorphError> {
        self.morphology.analyze(form)
    }
    
    fn lookup_concept(&self, concept: &ConceptId) -> Option<&LexEntry> {
        self.lexicon.get(concept)
    }
    
    fn lookup_form(&self, form: &str) -> Vec<&LexEntry> {
        self.lexicon.find_by_form(form)
    }
}
```

---

## Engine Construction and Registration

```rust
// src/main.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("data");
    
    let mut translator = UniversalTranslator::new();
    
    // Register natural language engines
    translator.register(Box::new(PolishEngine::load(data_dir)?));
    translator.register(Box::new(EnglishEngine::load(data_dir)?));
    
    // Future: register formal language engines
    // translator.register(Box::new(MathEngine::load(data_dir)?));
    // translator.register(Box::new(LogicEngine::load(data_dir)?));
    
    // Future: register programming language engines
    // translator.register(Box::new(PythonEngine::load(data_dir)?));
    // translator.register(Box::new(SqlEngine::load(data_dir)?));
    
    // Add middleware
    translator.add_middleware(Box::new(LoggingMiddleware));
    
    // Translate
    let result = translator.translate(
        "Tomek dał jabłko Izie",
        &LanguageId::PL,
        &LanguageId::EN,
    )?;
    
    println!("{}", result);
    // → "Tomek gave Iza an apple"
    
    Ok(())
}
```

---

## Middleware System

Middleware hooks into the translation pipeline for logging, metrics, filtering, and transformation.

```rust
pub trait Middleware {
    /// Called before parsing
    fn before_parse(&self, input: &str) -> String {
        input.to_string()
    }
    
    /// Called after parsing (Interlingua produced)
    fn after_parse(&self, il: Interlingua) -> Interlingua {
        il
    }
    
    /// Called before generation
    fn before_generate(&self, il: &Interlingua) -> Interlingua {
        il.clone()
    }
    
    /// Called after generation (surface string produced)
    fn after_generate(&self, output: &str) -> String {
        output.to_string()
    }
}

/// Logging middleware
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn before_parse(&self, input: &str) -> String {
        log::info!("[PARSE] Input: {}", input);
        input.to_string()
    }
    
    fn after_parse(&self, il: Interlingua) -> Interlingua {
        log::info!("[PARSE] Interlingua: {:?}", il);
        il
    }
    
    fn after_generate(&self, output: &str) -> String {
        log::info!("[GENERATE] Output: {}", output);
        output.to_string()
    }
}

/// Metrics middleware
pub struct MetricsMiddleware {
    parse_times: Mutex<Vec<Duration>>,
    generate_times: Mutex<Vec<Duration>>,
}

/// Profanity filter middleware
pub struct ProfanityFilter {
    blocked_words: HashSet<String>,
}

impl Middleware for ProfanityFilter {
    fn before_parse(&self, input: &str) -> String {
        // Filter input
        let mut filtered = input.to_string();
        for word in &self.blocked_words {
            filtered = filtered.replace(word, "***");
        }
        filtered
    }
}
```

---

## Cross-Language Translation Examples

### Natural → Natural (Current)

```
PL: "Tomek dał jabłko Izie"
→ Interlingua: Frame::Transfer { agent: Tomek, recipient: Iza, theme: Apple }
→ EN: "Tomek gave Iza an apple"
```

### Formal → Natural (Future)

```
Math: "∀x ∈ ℝ: x² ≥ 0"
→ Interlingua: QuantifiedStatement { ∀, x, ℝ, x² ≥ 0 }
→ PL: "Dla każdej liczby rzeczywistej x, kwadrat x jest większy lub równy zero"
→ EN: "For every real number x, the square of x is greater than or equal to zero"
```

### Programming → Natural (Future)

```
Python: "for i in range(10): print(i)"
→ Interlingua: Loop { For, i, range(10), [Print(i)] }
→ PL: "Powtórz 10 razy, wypisując licznik"
→ EN: "Repeat 10 times, printing the counter"
```

### Natural → Formal (Future)

```
EN: "The sum of two and three equals five"
→ Interlingua: Equation { lhs: Sum(2, 3), rhs: 5, relation: Equals }
→ Math: "2 + 3 = 5"
```

---

## Summary of All Documents

| Document | What It Defines |
|----------|----------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Big picture: 5 layers, data flow, file structure |
| [INTERLINGUA.md](./INTERLINGUA.md) | Core types: InterlinguaNode, Frame, Entity, deduction |
| [LEXICON.md](./LEXICON.md) | Master concepts + sub-lexicons, RON format |
| [MORPHOLOGY.md](./MORPHOLOGY.md) | Algorithmic paradigm rules, inflection, exceptions |
| [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) | Case system, role→case mapping, negation effects |
| [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md) | Declarative language description |
| [ENGINE.md](./ENGINE.md) | You are here — layered plugin architecture |
| [DISCOURSE.md](./DISCOURSE.md) | Context management, reference resolution, salience |
| [ONTOLOGY.md](./ONTOLOGY.md) | Concept hierarchy, IS_A relations, semantic validation |
| [PRONOUNS.md](./PRONOUNS.md) | Pronoun system, enclitic forms, reflexives |
| [TEMPORAL.md](./TEMPORAL.md) | Temporal reasoning and time references |
| [QUANTIFICATION.md](./QUANTIFICATION.md) | Quantifiers and logical operators |
| [ERROR_HANDLING.md](./ERROR_HANDLING.md) | Graceful degradation and error recovery |
| [SPEECH_ACTS.md](./SPEECH_ACTS.md) | Speech act recognition |
| [INTENTS.md](./INTENTS.md) | Intent extraction |
| [DIALOGUE.md](./DIALOGUE.md) | Dialogue management |
| [RESPONSE_PLANNING.md](./RESPONSE_PLANNING.md) | Response planning |
| [MEMORY.md](./MEMORY.md) | Long-term memory |
| [API.md](./API.md) | Public API |
