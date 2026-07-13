# Interlingua — Specification

Interlingua is the **language-neutral semantic core** of lexFlex. It is not a language — it is a **protocol** that represents meaning in a way that is richer than any single natural language. Every language plugin maps to and from this representation.

## Core Principle

> **Interlingua after deduction is complete.** Every pronoun is resolved, every implicit meaning is explicit, every ambiguity is eliminated. An Interlingua representation contains everything needed to generate a sentence in any target language.

## Universality Principle

> **Interlingua is a superset of all natural languages.** It contains ALL possible grammatical, semantic, and pragmatic features found across all languages. Each language uses only the subset relevant to it.

This means:
- **FeatureBundle** contains ALL possible features (gender, case, tense, aspect, evidentiality, honorifics, classifiers, etc.)
- **Case enum** contains ALL cases from all languages (Polish 7, Finnish 15, Hungarian 18, etc.)
- **Sentence** contains optional tense, aspect, evidentiality, topic-comment structure
- Each language's **LanguageDescriptor** specifies which features it uses and which it ignores

See [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md) for detailed examples across languages (Polish, English, Chinese, Japanese, Finnish, Turkish, Arabic, etc.).

---

## Type Hierarchy

```
Utterance                    ← top-level: a complete communicative act
├── Discourse                ← context: who speaks, to whom, why
├── Sentence[]               ← one utterance may contain multiple sentences
│   ├── Frame[]              ← predicate-argument structures
│   ├── Tense, Aspect, Modality
```

**2026-07-10 Audit note (synced with ERRORS.MD full PL+EN section):** 
Entity currently uses flat features + optional `coordination: Option<Coordination>`. No dedicated head + `modifiers: Vec<Entity>` structure yet. FeatureBundle is rich (includes Degree, Gender variants incl. MasculinePersonal for virile potential, Animacy, etc.) but unification/propagation and NP agreement are still ad-hoc (see ERRORS.MD "NP Structure", "Agreement Engine", "Linguistic Theory Gaps"). Future extensions for clitics, government slots, and richer modifier lists must be added here + reflected in ERRORS.MD + UNIFIED + PLAN. Docs kept in sync.
│   ├── Polarity             ← positive / negative
│   ├── Illocution           ← statement, question, command, exclamation
│   ├── Topic                ← what the sentence is about
│   ├── Focus                ← what carries emphasis
│   └── ResolvedRefs         ← pronoun → entity mappings
```

---

## Entity

An Entity is anything that can participate in a semantic role.

```rust
struct Entity {
    concept: Concept,                  // APPLE, PERSON, CITY, ...
    name: Option<String>,              // "Tomek", "Warszawa"
    features: FeatureBundle,           // [m, sg, 3rd], [def], [animate]
    reference: Reference,              // how this entity was introduced
}

enum Reference {
    Direct,                            // explicitly named: "Tomek"
    Anaphoric(String),                 // refers back: "jej" → resolved to entity id
    Cataphoric(String),                // refers forward (rare)
    Deictic,                           // "this", "that" — context-dependent
    Generic,                           // "people in general"
    Unresolved,                        // deduction couldn't resolve (error state)
}
```

### FeatureBundle

A set of grammatical/semantic features that describe an entity or predicate. Feature bundles are **supersets** — they contain ALL possible features across ALL languages. A specific language's engine reads only the features it cares about.

```rust
struct FeatureBundle {
    // ── Morphological features (language-specific subsets) ──
    gender: Option<Gender>,            // PL/DE/RU/FR: m/f/n; EN/ZH: None
    number: Option<Number>,            // sg, pl, du (all languages)
    case: Option<Case>,                // PL: 7 cases; FI: 15 cases; EN: None
    animacy: Option<Animacy>,          // PL/RU: animate/inanimate; EN/ZH: None
    
    // ── Verbal features ──
    tense: Option<Tense>,              // EN/FR/DE: past/present/future; ZH: None
    aspect: Option<Aspect>,            // PL/RU/ZH: perfective/imperfective
    mood: Option<Mood>,                // indicative/subjunctive/imperative
    voice: Option<Voice>,              // active/passive/middle
    evidentiality: Option<Evidentiality>, // TR/JA/KO: direct/reported/inferred; PL/EN: None
    
    // ── Pragmatic features ──
    honorific_level: Option<HonorificLevel>, // JA/KO/JV: plain/polite/honorific/humble; PL/EN: None
    
    // ── Syntactic features ──
    person: Option<Person>,            // 1st, 2nd, 3rd
    definiteness: Option<Definiteness>,// EN: the/a; PL: None
    countability: Option<Countability>,// EN: count/mass; PL: None
    
    // ── Classifier features (Asian languages) ──
    classifier: Option<Classifier>,    // ZH/JA/TH/VN: 个/人/匹/etc; PL/EN: None
    
    // ── Semantic features ──
    concreteness: Option<Concreteness>,// concrete, abstract
}

// ── Morphological enums ──
enum Gender { Masculine, Feminine, Neuter }
enum Number { Singular, Plural, Dual }
enum Person { First, Second, Third }
enum Animacy { Animate, Inanimate }
enum Definiteness { Definite, Indefinite }
enum Countability { Count, Mass }
enum Concreteness { Concrete, Abstract }

// ── Case enum (universal superset) ──
enum Case {
    // Core IE cases
    Nominative,        // PL, DE, RU, LA, GR
    Genitive,          // PL, DE, RU, LA, GR
    Dative,            // PL, DE, RU, LA
    Accusative,        // PL, DE, RU, LA, GR
    
    // Extended Slavic/Baltic
    Instrumental,      // PL, RU, LT
    Locative,          // PL, RU
    Vocative,          // PL, CS, HR, LT, LA, GR
    Prepositional,     // RU
    
    // Finno-Ugric (Finnish, Hungarian)
    Partitive,         // FI: osittainen (partial)
    Inessive,          // FI: -ssa (in)
    Elative,           // FI: -sta (from)
    Illative,          // FI: -Vn (into)
    Adessive,          // FI: -lla (on)
    Ablative,          // FI: -lta (from)
    Allative,          // FI: -lle (to)
    Essive,            // FI: -na (as)
    Translative,       // FI: -ksi (becoming)
    Comitative,        // FI: -ne (with)
    Abessive,          // FI: -tta (without)
    
    // Ergative-absolutive languages
    Ergative,          // BASQUE, GEORGIAN
    Absolutive,        // BASQUE, GEORGIAN
    
    // Austronesian
    Oblique,           // TAGALOG
}

// ── Verbal enums ──
enum Tense { Past, Present, Future }
enum Aspect { Perfective, Imperfective, Progressive, Habitual }
enum Mood { Indicative, Subjunctive, Imperative, Conditional }
enum Voice { Active, Passive, Middle }

enum Evidentiality {
    Direct,         // speaker witnessed the event (TR, JA, KO)
    Reported,       // speaker heard it from someone else
    Inferred,       // speaker inferred from evidence
    Assumed,        // speaker assumes/believes
}

// ── Pragmatic enums ──
enum HonorificLevel {
    Plain,          // informal (PL: ty, EN: you)
    Polite,         // formal (PL: Pan/Pani, EN: sir/madam)
    Honorific,      // respectful (JA: お...になる)
    Humble,         // self-deprecating (JA: お...する)
}

// ── Classifier enum (Asian languages) ──
enum Classifier {
    General,        // ZH: 个 (ge), JA: つ (tsu)
    Person,         // ZH: 人 (rén), JA: 人 (nin)
    Animal,         // JA: 匹 (hiki) - small animals
    Flat,           // JA: 枚 (mai) - flat objects
    Long,           // JA: 本 (hon) - long objects
    Book,           // JA: 冊 (satsu) - books
    Custom(String), // language-specific classifiers
}
```

**Cross-language examples:**

```
"Tomek" (PL) → Entity {
    features: {
        gender: Some(Masculine),
        number: Some(Singular),
        case: Some(Nominative),
        animacy: Some(Animate),
        tense: None, aspect: None,
        evidentiality: None,
        honorific_level: None,
        classifier: None,
    }
}

"苹果" (píngguǒ, ZH) → Entity {
    features: {
        gender: None,           // ZH has no grammatical gender
        number: Some(Singular),
        case: None,             // ZH uses word order, not cases
        classifier: Some(General), // ZH uses classifiers: 个苹果
    }
}

"りんご" (ringo, JA) → Entity {
    features: {
        gender: None,
        number: Some(Singular),
        case: None,             // JA uses particles (が, を, に)
        honorific_level: Some(Plain),
    }
}

"kirja" (FI) → Entity {
    features: {
        gender: None,           // FI has no grammatical gender
        number: Some(Singular),
        case: Some(Inessive),   // FI: "kirjassa" = "in the book"
    }
}
```

---

## Syntax Tree

Syntactic parsing produces a tree structure before semantic analysis. lexFlex supports both constituency and dependency parsing.

### Constituency Tree

```rust
pub struct SyntaxTree {
    pub root: SyntaxNode,
    pub language: LanguageId,
}

pub struct SyntaxNode {
    pub node_type: NodeType,
    pub span: (usize, usize),  // start, end positions in input
    pub children: Vec<SyntaxNode>,
    pub features: FeatureBundle,
}

pub enum NodeType {
    // Phrasal categories
    S,      // Sentence
    NP,     // Noun Phrase
    VP,     // Verb Phrase
    PP,     // Prepositional Phrase
    AP,     // Adjective Phrase
    AdvP,   // Adverbial Phrase
    
    // Functional categories
    CP,     // Complementizer Phrase (for embedded clauses)
    TP,     // Tense Phrase
    DP,     // Determiner Phrase
    NegP,   // Negation Phrase
    
    // Lexical categories
    N,      // Noun
    V,      // Verb
    Adj,    // Adjective
    Adv,    // Adverb
    P,      // Preposition
    Det,    // Determiner
    Conj,   // Conjunction
    Comp,   // Complementizer (that, if, whether)
    
    // Special
    EmptyCategory(EmptyCategory),
}

pub enum EmptyCategory {
    PRO,    // Subject of infinitival clause: "Tom tried [PRO to give]"
    Pro,    // Dropped subject in pro-drop: "[pro] dał jabłko"
    Trace,  // Left by movement: "What did Tom give __?"
    NullOperator,  // In relative clauses: "the apple [that __ was given]"
}
```

### Dependency Tree

```rust
pub struct DependencyTree {
    pub nodes: Vec<DependencyNode>,
    pub root: usize,  // index of root node (usually main verb)
}

pub struct DependencyNode {
    pub id: usize,
    pub word: String,
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub features: FeatureBundle,
    pub head: Option<usize>,  // index of head node
    pub relation: DependencyRelation,
}

pub enum DependencyRelation {
    // Core arguments
    Subject,
    Object,
    IndirectObject,
    
    // Modifiers
    AdjectivalModifier,
    AdverbialModifier,
    Determiner,
    Numeral,
    
    // Phrasal
    PrepositionalModifier,
    ClausalModifier,
    RelativeClause,
    
    // Coordination
    Coordination,
    Conjunct,
    
    // Functional
    Auxiliary,
    Negation,
    CaseMarker,
    
    // Special
    Root,
    Punctuation,
}
```

### Syntactic Movement

Movement operations create dependencies between surface and underlying positions:

```rust
pub struct Movement {
    pub from: usize,      // original position
    pub to: usize,        // surface position
    pub movement_type: MovementType,
    pub trace: usize,     // position of trace
}

pub enum MovementType {
    WhMovement,           // "What did Tom give __?"
    Topicalization,       // "The apple, Tom gave __ to Iza"
    Passive,              // "The apple was given __ by Tom"
    Raising,              // "Tom seems __ to have given the apple"
    Scrambling,           // "Jabłko Tomek dał Izie" (Polish free word order)
}
```

---

## Complex Sentences

### Relative Clauses

```rust
pub struct RelativeClause {
    pub antecedent: EntityId,
    pub relativizer: Option<String>,  // "that", "which", "who", "który"
    pub clause: Box<Sentence>,
    pub gap_position: usize,  // position of the gap in the clause
}

// Example: "The apple [that Tom gave __ to Iza] was red"
// antecedent: APPLE
// relativizer: Some("that")
// clause: Sentence { frames: [Transfer { agent: Tom, recipient: Iza, theme: GAP }] }
// gap_position: 2 (theme position)
```

### Adverbial Clauses

```rust
pub struct AdverbialClause {
    pub clause_type: AdverbialClauseType,
    pub subordinating_conjunction: String,  // "because", "when", "if", "although"
    pub clause: Box<Sentence>,
}

pub enum AdverbialClauseType {
    Temporal,       // "when Tom gave the apple"
    Causal,         // "because Tom gave the apple"
    Conditional,    // "if Tom gives the apple"
    Concessive,     // "although Tom gave the apple"
    Purpose,        // "so that Iza would eat the apple"
    Result,         // "so that Iza ate the apple"
}
```

### Complement Clauses

```rust
pub struct ComplementClause {
    pub complementizer: Option<String>,  // "that", "if", "whether"
    pub clause: Box<Sentence>,
    pub matrix_verb: String,  // verb that takes the complement
}

// Example: "Tom said [that he gave the apple]"
// complementizer: Some("that")
// clause: Sentence { frames: [Transfer { agent: he, theme: apple }] }
// matrix_verb: "said"
```

### Coordination

```rust
pub struct Coordination {
    pub conjuncts: Vec<Sentence>,
    pub coordinator: Option<String>,  // "and", "but", "or"
    pub coordination_type: CoordinationType,
}

pub enum CoordinationType {
    ClausalCoordination,    // "Tom gave an apple and Iza ate it"
    VPCoordination,         // "Tom gave an apple and ate a book"
    NPCoordination,         // "Tom and Iza gave apples"
    Gapping,                // "Tom gave an apple to Iza, and Iza __ a book to Tom"
    VPellipsis,             // "Tom gave an apple, and Iza did too"
}
```

### Ellipsis

```rust
pub struct Ellipsis {
    pub ellipsis_type: EllipsisType,
    pub antecedent: EntityId,  // what is being elided
    pub position: usize,       // where the ellipsis occurs
}

pub enum EllipsisType {
    Gapping,          // "Tom gave __ to Iza, and Iza __ a book to Tom"
    VPellipsis,       // "Tom gave an apple, and Iza did __ too"
    Sluicing,         // "Someone gave an apple, but I don't know who __"
    NPellipsis,       // "Tom gave me an apple, and I gave him __"
    Predicateellipsis, // "Tom gave an apple, and Iza did __ too"
}
```

### Cleft Sentences

```rust
pub struct CleftSentence {
    pub cleft_type: CleftType,
    pub focused_element: EntityId,
    pub presupposition: Box<Sentence>,
}

pub enum CleftType {
    ItCleft,          // "It was Tom who gave the apple"
    WhCleft,          // "What Tom gave was an apple"
    PseudoCleft,      // "The one who gave the apple was Tom"
}
```

### Control and Raising

```rust
pub struct ControlStructure {
    pub control_type: ControlType,
    pub matrix_verb: String,
    pub embedded_clause: Box<Sentence>,
    pub controller: EntityId,  // who controls the embedded subject
}

pub enum ControlType {
    SubjectControl,   // "Tom tried [PRO to give the apple]" (Tom controls PRO)
    ObjectControl,    // "Tom asked Iza [PRO to give the apple]" (Iza controls PRO)
}

pub struct RaisingStructure {
    pub raising_verb: String,  // "seem", "appear", "likely"
    pub raised_subject: EntityId,
    pub embedded_clause: Box<Sentence>,
}

// Example: "Tom seems [__ to have given the apple]"
// raising_verb: "seems"
// raised_subject: Tom
// embedded_clause: Sentence { frames: [Transfer { agent: Tom, theme: apple }] }
```

---

## Voice and Valency

### Voice

```rust
pub enum Voice {
    Active,      // "Tom gave the apple"
    Passive,     // "The apple was given by Tom"
    Middle,      // "The apple sells well" (some languages)
    Antipassive, // "Tom gave" (agent-focused, patient omitted)
    Applicative, // "Tom gave Iza the apple" (benefactive applied)
    Causative,   // "Tom made Iza give the apple"
    Reflexive,   // "Tom washes himself"
    Reciprocal,  // "Tom and Iza love each other"
}

pub struct VoiceTransformation {
    pub original_voice: Voice,
    pub transformed_voice: Voice,
    pub transformation_type: VoiceTransformationType,
}

pub enum VoiceTransformationType {
    PassiveTransformation,   // Active → Passive
    AntipassiveTransformation,
    ApplicativeTransformation,
    CausativeTransformation,
}
```

### Valency

```rust
pub struct ValencyFrame {
    pub verb: String,
    pub required_arguments: Vec<Argument>,
    pub optional_arguments: Vec<Argument>,
    pub valency_changes: Vec<ValencyChange>,
}

pub struct Argument {
    pub role: SemanticRole,
    pub case: Option<Case>,
    pub preposition: Option<String>,
}

pub enum ValencyChange {
    Transitivization,     // Intransitive → Transitive
    Ditransitivization,   // Transitive → Ditransitive
    Detransitivization,   // Transitive → Intransitive
    Reflexivization,      // Transitive → Reflexive
}
```

---

## Reported Speech

```rust
pub struct ReportedSpeech {
    pub reporting_verb: String,  // "said", "asked", "told"
    pub speaker: EntityId,
    pub addressee: Option<EntityId>,
    pub reported_content: Box<Sentence>,
    pub speech_type: SpeechType,
    pub tense_shift: Option<TenseShift>,  // backshift in reported speech
}

pub enum SpeechType {
    DirectSpeech,       // Tom said: "I gave the apple"
    IndirectSpeech,     // Tom said that he gave the apple
    FreeIndirectSpeech, // Tom wondered. Had he given the apple?
}

pub struct TenseShift {
    pub original_tense: Tense,
    pub shifted_tense: Tense,  // Present → Past, Past → Past Perfect
}
```

---

## Conditional and Subjunctive

```rust
pub struct ConditionalSentence {
    pub condition: Box<Sentence>,
    pub consequence: Box<Sentence>,
    pub conditional_type: ConditionalType,
    pub mood: Mood,
}

pub enum ConditionalType {
    Zero,       // "If you heat water, it boils" (general truth)
    First,      // "If you heat the water, it will boil" (real future)
    Second,     // "If you heated the water, it would boil" (hypothetical present)
    Third,      // "If you had heated the water, it would have boiled" (hypothetical past)
    Mixed,      // "If you had heated the water, it would be boiling now"
}

pub enum Mood {
    Indicative,     // factual statements
    Subjunctive,    // hypothetical, wishes, suggestions
    Imperative,     // commands
    Conditional,    // conditional mood (some languages)
    Optative,       // wishes (some languages)
    Jussive,        // commands for 3rd person (some languages)
}
```

---

## Wh-Movement and Questions

```rust
pub struct WhQuestion {
    pub wh_word: String,  // "who", "what", "where", "when", "why", "how"
    pub wh_role: SemanticRole,  // what role the wh-word fills
    pub moved_from: usize,  // original position
    pub moved_to: usize,    // surface position (usually sentence-initial)
    pub trace: usize,       // position of trace
    pub question_type: QuestionType,
}

pub enum QuestionType {
    YesNo,              // "Did Tom give the apple?"
    WhSubject,          // "Who gave the apple?" (no movement)
    WhObject,           // "What did Tom give?" (movement)
    WhAdjunct,          // "Where did Tom give the apple?" (movement)
    WhPrepositional,    // "To whom did Tom give the apple?" (movement)
    MultipleWh,         // "Who gave what to whom?" (multiple wh-words)
    EchoQuestion,       // "Tom gave the apple to WHO?" (no movement, intonation)
}
```

---

## Binding Theory

Binding theory constrains coreference relations. See [DISCOURSE.md](./DISCOURSE.md#binding-theory-extended) for full definitions of:
- `BindingDomain`, `BindingDomainType`, `ClauseBoundary`, `ClauseType`
- `BindingPrinciple` (PrincipleA, PrincipleB, PrincipleC)
- `AnaphorType` (Anaphor, Pronoun, RExpression)

### Binding Principles Summary

**Principle A**: Anaphors (reflexives like "himself", "siebie") must be bound in their local domain.

**Principle B**: Pronouns ("he", "on") must be free in their local domain.

**Principle C**: R-expressions (proper names like "Tomek", "Iza") must be free everywhere.

---

## Information Structure (Extended)

```rust
pub struct InformationStructure {
    pub topic: TopicStatus,
    pub focus: FocusStatus,
    pub given: Vec<EntityId>,
    pub new: Vec<EntityId>,
    pub presupposition: Option<Box<Sentence>>,
}

pub enum FocusStatus {
    Broad,              // entire sentence is new
    Narrow(EntityId),   // specific element is focused
    Contrastive {       // contrasted with alternative
        focused: EntityId,
        alternative: EntityId,
    },
    Verum,              // focus on truth value ("Tom DID give the apple")
}

pub struct Topicalization {
    pub topicalized_element: EntityId,
    pub original_position: usize,
    pub surface_position: usize,
    pub trace: usize,
}

pub struct Scrambling {
    pub scrambled_elements: Vec<(EntityId, usize, usize)>,  // (element, from, to)
    pub motivation: ScramblingMotivation,
}

pub enum ScramblingMotivation {
    Topicalization,     // moving topic to front
    FocusFronting,      // moving focus to front
    Defocalization,     // moving given information to middle
    Weight,             // heavy elements moved to end
}
```

---

## Frame

A Frame describes an event, state, or relationship as a **predicate with semantic roles**.

```rust
enum Frame {
    // Physical actions
    Transfer {                   // give, send, hand
        agent: Entity,
        recipient: Entity,
        theme: Entity,
    },
    Motion {                     // go, come, run
        mover: Entity,
        source: Option<Entity>,
        goal: Option<Entity>,
        path: Option<Entity>,
    },
    Creation {                   // make, build, cook
        creator: Entity,
        created: Entity,
        material: Option<Entity>,
    },
    Destruction {                // break, destroy, eat
        agent: Entity,
        patient: Entity,
        instrument: Option<Entity>,
    },
    
    // Perception & cognition
    Perception {                 // see, hear, notice
        experiencer: Entity,
        stimulus: Entity,
    },
    Cognition {                  // think, know, believe
        cognizer: Entity,
        content: Entity,         // what is thought/known
    },
    Emotion {                    // love, fear, enjoy
        experiencer: Entity,
        stimulus: Entity,
    },
    
    // Communication
    Communication {              // say, tell, ask
        speaker: Entity,
        addressee: Option<Entity>,
        message: Entity,
    },
    Statement {                  // simple assertion about a subject
        subject: Entity,
        property: Entity,        // predicate adjective or noun
    },
    
    // Existence & possession
    Existence {                  // exist, be
        entity: Entity,
        location: Option<Entity>,
    },
    Possession {                 // have, own
        possessor: Entity,
        possessed: Entity,
    },
    
    // Custom frame for concepts not yet categorized
    Custom {
        name: String,
        roles: Vec<(SemanticRole, Entity)>,
    },
}
```

### Semantic Roles

Roles that entities can play within a frame:

```rust
enum SemanticRole {
    Agent,         // volitional doer: "Tomek dał"
    Patient,       // entity undergoing change: "zbił szybę"
    Theme,         // entity moved/located: "dał jabłko"
    Recipient,     // endpoint of transfer: "dał Izie"
    Experiencer,   // entity perceiving/feeling: "widzi", "boi się"
    Stimulus,      // what is perceived: "widzi dom"
    Source,        // origin of motion: "z Warszawy"
    Goal,          // destination of motion: "do Krakowa"
    Location,      // where: "w domu"
    Instrument,    // means: "nożem"
    Beneficiary,   // for whom: "dla niej"
    Topic,         // what communication is about
}
```

### Role → Case Mapping

Each language defines how semantic roles map to grammatical cases (or prepositions). This is defined **per language**, but the canonical defaults for Polish are:

```
Role            PL Case          EN Realization
──────────      ──────────       ──────────────
Agent           Nominative       Subject position
Patient         Accusative       Direct object
Theme           Accusative       Direct object
Recipient       Dative           Indirect object / "to X"
Experiencer     Nominative       Subject
Stimulus        Accusative/Gen   Direct object / "of X"
Source          Ablative(=Gen)   "from X"
Goal            Accusative/Loc   "to X"
Location        Locative         "in/at X"
Instrument      Instrumental     "with X"
Beneficiary     Dative           "for X"
```

> See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) for full case system.

---

## Sentence

```rust
struct Sentence {
    frames: Vec<Frame>,                  // usually 1, coordinated clauses may have more

    // Temporal (optional — not all languages have tense)
    tense: Option<Tense>,                // Some(Past) for PL/EN, None for ZH
    aspect: Option<Aspect>,              // Some(Perfective) for PL/RU/ZH
    
    // Modal
    modality: Option<Modality>,
    polarity: Polarity,
    
    // Evidentiality (TR, JA, KO: important; PL, EN: None)
    evidentiality: Option<Evidentiality>,

    // Pragmatic
    illocution: Illocution,
    
    // Information structure
    topic: Option<EntityRef>,            // what the sentence is about
    topic_marker: Option<TopicMarker>,   // JA: は (wa), KO: 은/는; PL: implicit
    focus: Option<EntityRef>,            // what carries emphasis

    // Resolved references from this sentence
    resolved_refs: Vec<(String, EntityRef)>,  // pronoun_text → entity

    // Coordination
    coordination: Option<Coordination>,  // "a", "i", "ale"
}

// ── Temporal enums ──
enum Tense {
    Past,
    Present,
    Future,
}

enum Aspect {
    Perfective,          // completed, bounded: "dał", "zrobił"
    Imperfective,        // ongoing, unbounded: "dawał", "robił"
    Progressive,         // EN: "is giving"
    Habitual,            // "used to give", "would give"
}

// ── Topic marker (for topic-prominent languages) ──
enum TopicMarker {
    Explicit(String),    // JA: は (wa), KO: 은/는 (eun/neun)
    Implicit,            // PL: word order (topic first)
}

enum Modality {
    Realis,              // factual: "dał" (it happened)
    Irrealis,            // non-factual: "dałby" (would give), "chce dać" (wants to give)
    Epistemic,           // possibility: "mógł dać" (could have given)
    Deontic,             // obligation: "musi dać" (must give)
}

enum Polarity {
    Positive,
    Negative,            // "nie dał" → in PL: NEG + GEN for direct object
}

enum Illocution {
    Statement,
    YesNoQuestion,
    WhQuestion { wh_role: SemanticRole },
    Command,
    Exclamation,
}

struct Coordination {
    conjunct_type: ConjunctionType,      // And, Or, But, Contrast
    position: CoordPosition,             // First, Middle, Last
}

enum ConjunctionType {
    And,     // "i", "and"
    Or,      // "lub", "or"
    But,     // "ale", "but"
    Contrast,// "a", "whereas"
}
```

---

## Utterance

The top-level unit — a complete communicative act.

```rust
struct Utterance {
    // MVP v0.1:
    sentences: Vec<Sentence>,
    
    // Feature v0.2+:
    discourse: Discourse,                    // conversation context
    discourse_relations: Vec<DiscourseRelation>,  // relationships between sentences
}
```

**Note:** `discourse` and `discourse_relations` are planned for v0.2+. See [DISCOURSE.md](./DISCOURSE.md).

---

## Discourse

**Partial implementation:** `Utterance.discourse` tracks `entities_in_focus`, `recent_mentions`, `coref_edges`, and `current_topic` (continuing salient subject). `Sentence.construction_concepts` holds discourse construction concept IDs (`ZERO_ANAPHORA`, `TOPIC_CONTINUATION`, `CONTINUING_AGENT`). See [DISCOURSE.md](./DISCOURSE.md) for zero-anaphora resolution.

Context that spans the entire utterance (and potentially multiple utterances in conversation).

```rust
struct Discourse {
    // Participants
    speaker: Entity,
    addressee: Entity,
    participants: Vec<Entity>,           // all entities in discourse scope
    
    // Context
    topic: Option<EntityRef>,            // current topic of conversation
    shared_knowledge: Vec<EntityRef>,    // what both sides know
    purpose: DiscoursePurpose,
    register: Register,
    
    // History (for multi-utterance conversation)
    history: Vec<UtteranceSummary>,      // condensed previous utterances
}

enum DiscoursePurpose {
    Inform,
    Request,
    Command,
    Question,
    Promise,
    Social,              // greetings, farewells
}

enum Register {
    Informal,            // "daj mi" / "give me"
    Neutral,             // "proszę daj" / "please give"
    Formal,              // "czy mógłby Pan podać" / "could you please pass"
    Technical,           // domain-specific terminology
    Poetic,              // figurative, artistic
}
```

### Discourse and Generation

The discourse context directly influences how sentences are generated:

| Discourse Feature | PL Effect | EN Effect |
|---|---|---|
| `register: Formal` | "Pan/Pani" + 3sg verb | "sir/madam", modal verbs |
| `register: Informal` | pro-drop: "Dałem" | subject required: "I gave" |
| `speaker == agent` | 1st person verb | 1st person verb |
| `addressee == recipient` | 2nd person + DAT | 2nd person + "you" |
| `topic` established | pro-drop subject OK | pronoun substitution |

---

## Deduction

Deduction is the process of going from surface parse to complete Interlingua. It resolves all ambiguity and fills in implicit information.

**See [DEDUCTION.md](./DEDUCTION.md) for complete specification of the Deduction Engine.**

### What Deduction Must Resolve

#### 1. Case Disambiguation

```
"Tomek dał jabłko Izie"

Surface: "jabłko" is morphologically ambiguous — could be NOM or ACC.
Deduction: verb "dać" subcategorizes [agent:NOM, theme:ACC, recipient:DAT]
           → "jabłko" resolves to ACC (theme), not NOM (agent)
           → "Izie" is DAT → matches recipient role
```

#### 2. Reference Resolution

**⚠️ FEATURE - v0.2+** (requires discourse context)

```
"Tomek dał jej jabłko. Ona się ucieszyła."

Sentence 1: "jej" = PRON(DAT, F, SG)
  → Deduction: search discourse.participants for feminine entity
  → If found: resolved_refs["jej"] = Entity{id: "iza_1"}
  → If not found: reference = Unresolved (needs more context)

Sentence 2: "Ona" = PRON(NOM, F, SG)
  → Deduction: coreference with "jej" from sentence 1
  → resolved_refs["Ona"] = Entity{id: "iza_1"}
```

#### 3. Implicit Aspect/Modality

```
"dał"     → tense: PAST, aspect: PERFECTIVE, modality: REALIS
           → deduction: the action was completed successfully

"dawał"   → tense: PAST, aspect: IMPERFECTIVE, modality: REALIS
           → deduction: the action was ongoing, completion unknown

"chciał dać" → tense: PAST, aspect: PERFECTIVE, modality: IRREALIS
           → deduction: intention existed, actual giving unknown
```

#### 4. Negation Scope

```
"Tomek nie dał jabłka Izie"

Surface: "nie" before verb
Deduction:
  - polarity: NEGATIVE
  - in PL: negation of transitive verb changes direct object ACC → GEN
    "jabłko" (ACC) → "jabłka" (GEN) — this is a grammatical rule
```

#### 5. Verb Subcategorization

```
Each verb concept defines required case frames in the lexicon:

"dać"    → [agent:NOM, recipient:DAT, theme:ACC]
"szukać" → [agent:NOM, target:GEN]              ← NOT ACC!
"bać się" → [experiencer:NOM, stimulus:GEN]
"pomagać" → [agent:NOM, beneficiary:DAT]
```

This is stored in the lexicon (per concept), not hardcoded in the engine.

#### 6. Register Detection

```
"Czy mógłby Pan podać sól?"
  → modal "mógłby" (conditional) + "Pan" (honorific)
  → deduction: register = FORMAL

"Podaj sól."
  → imperative, 2sg
  → deduction: register = INFORMAL
```

### Deduction Pipeline

```
Surface Parse
    │
    ├─→ 1. Morphological analysis (each word → lemma + features)
    ├─→ 2. POS tagging + dependency parse
    ├─→ 3. Frame identification (verb → frame type)
    ├─→ 4. Role assignment (noun phrases → semantic roles via subcat)
    ├─→ 5. Case disambiguation (resolve ambiguous forms via subcat)
    ├─→ 6. Reference resolution (pronouns → entities)
    ├─→ 7. Aspect/modality inference (from verb form + context)
    ├─→ 8. Polarity detection (negation + case changes)
    └─→ 9. Register/illocution detection (from form)
    │
    ▼
Complete Interlingua
```

### Deduction State

```rust
struct DeductionState {
    // Mutable during deduction, frozen after
    unresolved_pronouns: Vec<(String, FeatureBundle)>,
    ambiguous_cases: Vec<(String, Vec<Case>)>,
    
    // Filled during deduction
    resolved_refs: Vec<(String, EntityRef)>,
    frame_assignments: Vec<Frame>,
    
    // Final
    discourse: Discourse,
    sentences: Vec<Sentence>,
}
```

After deduction completes, `unresolved_pronouns` and `ambiguous_cases` should be empty. If not, they are marked as `Unresolved` in the final Interlingua — the generation engine can then request clarification or make a best-effort choice.

---

## Concept System

Concepts are language-neutral semantic units. They are the **master vocabulary** of Interlingua.

```rust
struct Concept {
    id: ConceptId,                       // e.g., GIVE, APPLE, PERSON
    frame_type: FrameType,               // what frame this concept participates in
    roles: Vec<SemanticRole>,            // what roles it can play
    features: FeatureBundle,             // inherent features
}

// Concept IDs are stable across all languages
// Each language maps its words to these concepts
enum ConceptId {
    // Entities
    PERSON, ANIMAL, OBJECT, PLACE, TIME,
    APPLE, BOOK, HOUSE, WATER, BREAD,
    MOTHER, FATHER, FRIEND, CHILD,
    
    // Actions (each defines a frame type)
    GIVE,    // → Frame::Transfer
    TAKE,    // → Frame::Transfer (reversed)
    GO,      // → Frame::Motion
    SEE,     // → Frame::Perception
    THINK,   // → Frame::Cognition
    LOVE,    // → Frame::Emotion
    SAY,     // → Frame::Communication
    EAT,     // → Frame::Destruction (consumes theme)
    MAKE,    // → Frame::Creation
    
    // Properties
    BIG, SMALL, GOOD, BAD, NEW, OLD,
    RED, BLUE, GREEN, HOT, COLD,
    
    // ... ~500 total concepts for MVP
}
```

> See [LEXICON.md](./LEXICON.md) for how concepts map to language-specific words.

---

## InterlinguaNode — Universal Meaning Representation

Interlingua is not just for natural languages. It's a **universal protocol** for representing meaning across any language type: natural (Polish, English), formal (mathematics, logic), or programming (Python, SQL).

### The InterlinguaNode Enum

```rust
/// Universal meaning representation — the core of lexFlex
pub enum InterlinguaNode {
    // ─── Natural Language ───────────────────────
    /// Natural language utterance (Polish, English, etc.)
    Natural(Utterance),

    // ─── Formal Languages ───────────────────────
    /// Mathematical expression
    MathExpression(MathExpr),

    /// Logical proposition
    LogicalProposition(LogicalExpr),

    /// Theorem with optional proof
    Theorem {
        statement: Box<InterlinguaNode>,
        proof: Option<Vec<InterlinguaNode>>,
    },

    /// Equation or inequality
    Equation {
        lhs: Box<InterlinguaNode>,
        rhs: Box<InterlinguaNode>,
        relation: Relation,
    },

    /// Quantified statement
    QuantifiedStatement {
        quantifier: Quantifier,
        variable: Variable,
        domain: Option<Box<InterlinguaNode>>,
        body: Box<InterlinguaNode>,
    },

    // ─── Programming Languages ──────────────────
    /// Program statement
    ProgramStatement(ProgramStmt),

    /// Program block (sequence of statements)
    ProgramBlock(Vec<ProgramStmt>),

    /// Function/method definition
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        body: Box<InterlinguaNode>,
        return_type: Option<Type>,
    },

    // ─── Universal Constructs ───────────────────
    /// Conditional expression (if-then-else)
    Conditional {
        condition: Box<InterlinguaNode>,
        then_branch: Box<InterlinguaNode>,
        else_branch: Option<Box<InterlinguaNode>>,
    },

    /// Loop construct
    Loop {
        kind: LoopKind,
        body: Box<InterlinguaNode>,
    },

    /// Set literal
    Set {
        elements: Vec<InterlinguaNode>,
    },

    /// Function application
    Application {
        function: Box<InterlinguaNode>,
        args: Vec<InterlinguaNode>,
    },

    /// Variable reference
    Variable(String),

    /// Literal value
    Literal(Literal),
}

/// Type alias for convenience
pub type Interlingua = InterlinguaNode;
```

### Mathematical Expressions

```rust
pub enum MathExpr {
    /// Numeric literal
    Number(f64),
    
    /// Variable
    Variable(String),
    
    /// Binary operation: a + b, a * b, a / b
    BinaryOp {
        op: MathOp,
        lhs: Box<MathExpr>,
        rhs: Box<MathExpr>,
    },
    
    /// Unary operation: -a, |a|
    UnaryOp {
        op: UnaryMathOp,
        operand: Box<MathExpr>,
    },
    
    /// Function call: sin(x), log(x)
    Function {
        name: String,
        args: Vec<MathExpr>,
    },
    
    /// Summation: Σ(i=1 to n) i²
    Summation {
        variable: String,
        from: Box<MathExpr>,
        to: Box<MathExpr>,
        body: Box<MathExpr>,
    },
    
    /// Integral: ∫(a to b) f(x) dx
    Integral {
        variable: String,
        from: Option<Box<MathExpr>>,
        to: Option<Box<MathExpr>>,
        body: Box<MathExpr>,
    },
    
    /// Limit: lim(x→a) f(x)
    Limit {
        variable: String,
        approaches: Box<MathExpr>,
        body: Box<MathExpr>,
    },
}

pub enum MathOp {
    Add, Subtract, Multiply, Divide,
    Power, Modulo,
}

pub enum UnaryMathOp {
    Negate,
    Abs,
    Sqrt,
}

pub enum Relation {
    Equals,
    NotEquals,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    ApproxEquals,
}
```

### Logical Expressions

```rust
pub enum LogicalExpr {
    /// Atomic proposition: P, Q, R
    Proposition(String),
    
    /// Negation: ¬P
    Not(Box<LogicalExpr>),
    
    /// Conjunction: P ∧ Q
    And(Box<LogicalExpr>, Box<LogicalExpr>),
    
    /// Disjunction: P ∨ Q
    Or(Box<LogicalExpr>, Box<LogicalExpr>),
    
    /// Implication: P → Q
    Implies(Box<LogicalExpr>, Box<LogicalExpr>),
    
    /// Biconditional: P ↔ Q
    Iff(Box<LogicalExpr>, Box<LogicalExpr>),
    
    /// Quantified: ∀x P(x), ∃x Q(x)
    Quantified {
        quantifier: Quantifier,
        variable: String,
        body: Box<LogicalExpr>,
    },
    
    /// Predicate application: P(x, y)
    Predicate {
        name: String,
        args: Vec<InterlinguaNode>,
    },
}
```

### Program Statements

```rust
pub enum ProgramStmt {
    /// Assignment: x = expr
    Assignment {
        target: String,
        value: Box<InterlinguaNode>,
    },
    
    /// Conditional: if cond then ... else ...
    Conditional {
        condition: Box<InterlinguaNode>,
        then_branch: Vec<ProgramStmt>,
        else_branch: Option<Vec<ProgramStmt>>,
    },
    
    /// Loop: for/while/repeat
    Loop {
        kind: LoopKind,
        body: Vec<ProgramStmt>,
    },
    
    /// Function call: f(args)
    FunctionCall {
        name: String,
        args: Vec<InterlinguaNode>,
    },
    
    /// Return statement
    Return {
        value: Option<Box<InterlinguaNode>>,
    },
    
    /// Print/output statement
    Print {
        value: Box<InterlinguaNode>,
    },
}

pub enum LoopKind {
    /// for i in iterable
    For {
        variable: String,
        iterable: Box<InterlinguaNode>,
    },
    
    /// while condition
    While {
        condition: Box<InterlinguaNode>,
    },
    
    /// repeat N times
    Repeat {
        count: Box<InterlinguaNode>,
    },
}

pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub default_value: Option<Box<InterlinguaNode>>,
}

pub enum Type {
    /// Primitive types
    Int,
    Float,
    String,
    Bool,
    
    /// Collection types
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),
    
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    /// Custom/user-defined type
    Custom(String),
}

pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

pub struct Variable {
    pub name: String,
    pub var_type: Option<Type>,
}
```

---

## Temporal Reasoning

Natural languages express time through tense, aspect, and temporal adverbs. Interlingua must capture all temporal information for accurate translation and reasoning.

### Temporal Reference

See [TEMPORAL.md](./TEMPORAL.md) for complete definitions of temporal types:
- `TemporalReference` (Absolute, Relative, Deictic, Duration, Frequency, Sequence, Interval)
- `TemporalAnchor` (Now, SpeechTime, Event, Temporal)
- `TemporalRelation` (Before, After, During, Simultaneous, Until, Since)
- `Timestamp`, `Duration`, `TemporalResolver`

**Summary**: Temporal references can be absolute (specific timestamp), relative to an anchor point (now, speech time, event), or deictic (context-dependent words like "yesterday"). The `TemporalResolver` resolves these to absolute timestamps for cross-language translation.

### Temporal in Sentences

The `Sentence` struct (defined above) includes temporal fields:

```rust
struct Sentence {
    // ... all fields from above ...

    // Temporal information (extends the base Sentence)
    temporal: Option<TemporalReference>,
    event_time: Option<Timestamp>,
    reference_time: Option<Timestamp>,
}
```

### Temporal Examples

```
"wczoraj" → TemporalReference::Deictic {
    word: "wczoraj",
    resolved: Some(2024-01-14),  // if today is 2024-01-15
}

"za godzinę" → TemporalReference::Relative {
    offset: Duration { hours: 1, .. },
    anchor: TemporalAnchor::Now,
}

"w 2020 roku" → TemporalReference::Absolute {
    timestamp: Timestamp { year: 2020, .. }
}

"przed obiadem" → TemporalReference::Sequence {
    relation: TemporalRelation::Before,
    reference: Box::new(TemporalReference::Event("obiad")),
}

"codziennie" → TemporalReference::Frequency {
    count: 1,
    period: Duration { days: 1, .. }
}
```

---

## Quantification

Natural languages express quantity and scope through quantifiers: "all", "some", "none", "many". Interlingua must capture these for accurate semantic representation.

### Quantification

See [QUANTIFICATION.md](./QUANTIFICATION.md) for complete definitions of quantification types:
- `Quantifier` (Universal, Existential, NegatedExistential, Unique, Numerical, Proportional)
- `Proportion` (Most, Many, Few, Several)
- `QuantifiedExpression`, `Variable`
- `Scope`, `ScopeResolution`

**Summary**: Quantifiers express quantity and scope over variables. Universal ("all") uses implication (→), existential ("some") uses conjunction (∧), negated existential ("none") is equivalent to universal negation. Scope ambiguity occurs when multiple quantifiers can have different relative scopes.

### Quantification in Natural Language

```rust
/// Polish quantifiers
pub enum PolishQuantifier {
    /// "wszyscy", "każdy" — universal
    Wszyscy,
    
    /// "niektórzy", "jakiś" — existential
    Niektorzy,
    
    /// "żaden", "nikt" — negated existential
    Zaden,
    
    /// "dokładnie jeden" — unique
    DokladnieJeden,
    
    /// "trzech", "pięciu" — numerical
    Numeral(u32),
    
    /// "większość", "wielu", "kilku" — proportional
    Wiekszosc,
    Wielu,
    Kilku,
}
```

### Quantification Examples

```
"Wszyscy studenci zdali egzamin."
→ QuantifiedExpression {
    quantifier: Universal,
    variable: Variable { name: "x", var_type: None },
    domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
    body: Box::new(InterlinguaNode::Application {
        function: Box::new(InterlinguaNode::Variable("PASS")),
        args: vec![InterlinguaNode::Variable("x")],
    }),
    restrictions: vec![],
}

Logical form: ∀x (Student(x) → Pass(x, exam))

---

"Niektórzy studenci nie zdali."
→ QuantifiedExpression {
    quantifier: Existential,
    variable: Variable { name: "x", var_type: None },
    domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
    body: Box::new(InterlinguaNode::LogicalProposition(
        LogicalExpr::Not(Box::new(LogicalExpr::Predicate {
            name: "PASS",
            args: vec![InterlinguaNode::Variable("x")],
        }))
    )),
    restrictions: vec![],
}

Logical form: ∃x (Student(x) ∧ ¬Pass(x))

---

"Żaden student nie przyszedł."
→ QuantifiedExpression {
    quantifier: NegatedExistential,
    variable: Variable { name: "x", var_type: None },
    domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
    body: Box::new(InterlinguaNode::Application {
        function: Box::new(InterlinguaNode::Variable("COME")),
        args: vec![InterlinguaNode::Variable("x")],
    }),
    restrictions: vec![],
}

Logical form: ¬∃x (Student(x) ∧ Come(x))
Equivalent to: ∀x (Student(x) → ¬Come(x))

---

"Trzech studentów czytało książkę."
→ QuantifiedExpression {
    quantifier: Numerical(3),
    variable: Variable { name: "x", var_type: None },
    domain: Some(Box::new(InterlinguaNode::Variable("STUDENT"))),
    body: Box::new(InterlinguaNode::Application {
        function: Box::new(InterlinguaNode::Variable("READ")),
        args: vec![
            InterlinguaNode::Variable("x"),
            InterlinguaNode::Variable("book"),
        ],
    }),
    restrictions: vec![],
}

Logical form: ∃≥3 x (Student(x) ∧ Read(x, book))
```

### Scope Ambiguity

Natural language quantifiers can have ambiguous scope:

```
"Każdy student przeczytał jakąś książkę."

Reading 1: ∀x ∃y (Student(x) → (Book(y) ∧ Read(x, y)))
  → For each student, there exists a book (possibly different) that they read

Reading 2: ∃y ∀x (Book(y) ∧ (Student(x) → Read(x, y)))
  → There exists a single book that every student read

Resolution requires context or clarification.
```

---

## Capability System

Different language types can express different things. The capability system tracks what each language can and cannot represent.

### Capability System

See [ENGINE.md](./ENGINE.md#capabilities-and-limitations) for complete definitions of:
- `Capability` enum (TemporalReference, Deixis, EmotionExpression, Pragmatics, ProDrop, FreeWordOrder, MorphologicalInflection, Quantification, Ambiguity, FormalProof, NumericPrecision, SetTheory, LogicalConnectives, Procedures, ControlFlow, SideEffects, TypeSystem, ErrorHandling, Negation, Coordination, Conditionality, Reference)
- `Limitation` enum (NoEmotionExpression, NoDeixis, NoFormalProofs, NoProcedures, NoProDrop, NoAmbiguity)
- Capability matrix showing what each language type can express

### Capability Checking Example

```rust
impl InterlinguaNode {
    /// What capabilities are required to express this node?
    pub fn required_capabilities(&self) -> Vec<Capability> {
        match self {
            InterlinguaNode::Natural(_) => vec![
                Capability::TemporalReference,
                Capability::Deixis,
                Capability::EmotionExpression,
            ],

            InterlinguaNode::MathExpression(_) => vec![
                Capability::NumericPrecision,
            ],

            InterlinguaNode::QuantifiedStatement { .. } => vec![
                Capability::Quantification,
            ],

            InterlinguaNode::FunctionDef { .. } => vec![
                Capability::Procedures,
            ],

            InterlinguaNode::Loop { .. } => vec![
                Capability::ControlFlow,
            ],

            _ => vec![],
        }
    }
}
```

---

## Examples: Full Interlingua Representations

**Note:** Examples below show full Interlingua with discourse context (feature v0.2+). For MVP v0.1, `Utterance` contains only `sentences` without discourse.

### Example 1: "Tomek dał jabłko Izie"

```rust
// Feature v0.2+ (with discourse):
Utterance {
    discourse: Discourse {
        speaker: Entity { concept: PERSON, features: [1sg] },
        addressee: Entity { concept: PERSON, features: [2sg] },
        participants: [speaker, addressee, tomeck, iza],
        purpose: Inform,
        register: Neutral,
    },
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity {
                        concept: PERSON,
                        name: "Tomek",
                        features: { gender: M, number: Sg, person: Third, animacy: Animate },
                        reference: Direct,
                    },
                    recipient: Entity {
                        concept: PERSON,
                        name: "Iza",
                        features: { gender: F, number: Sg, person: Third, animacy: Animate },
                        reference: Direct,
                    },
                    theme: Entity {
                        concept: APPLE,
                        features: { gender: N, number: Sg, definiteness: None, countability: Count },
                        reference: Direct,
                    },
                }
            ],
            tense: Past,
            aspect: Perfective,
            modality: Realis,
            polarity: Positive,
            illocution: Statement,
            topic: Some(EntityRef("Tomek")),
            focus: None,
            resolved_refs: [],
            coordination: None,
        }
    ],
}
```

### Example 2: "Tomek dał jabłko Izie a czekoladę Tomkowi"

```rust
// Feature v0.2+ (with discourse):
Utterance {
    discourse: Discourse { /* same as above */ },
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { concept: PERSON, name: "Tomek", ... },
                    recipient: Entity { concept: PERSON, name: "Iza", ... },
                    theme: Entity { concept: APPLE, ... },
                }
            ],
            coordination: Some(Coordination {
                conjunct_type: Contrast,     // "a" = contrast/whereas
                position: First,
            }),
            /* ... */
        },
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { concept: PERSON, name: "Iza", ... },  // implied from context
                    recipient: Entity { concept: PERSON, name: "Tomek", ... },
                    theme: Entity { concept: CHOCOLATE, ... },
                }
            ],
            coordination: Some(Coordination {
                conjunct_type: Contrast,
                position: Last,
            }),
            /* ... */
        }
    ],
}
```

### Example 3: "Czy Tomek dał jej jabłko?" (with unresolved reference)

```rust
// Feature v0.2+ (with discourse):
Utterance {
    discourse: Discourse {
        participants: [speaker, addressee, tomeck],  // no feminine entity in context
        // ...
    },
    sentences: [
        Sentence {
            frames: [
                Frame::Transfer {
                    agent: Entity { name: "Tomek", ... },
                    recipient: Entity {
                        concept: PERSON,
                        features: { gender: F, number: Sg },
                        reference: Unresolved,  // "jej" — no feminine entity in discourse!
                    },
                    theme: Entity { concept: APPLE, ... },
                }
            ],
            illocution: YesNoQuestion,
            resolved_refs: [("jej", Unresolved)],
            /* ... */
        }
    ],
}
```

---

## Linguistic Graph (Augmenting View)

Each `Sentence` may carry an optional `graph: LinguisticGraph` — a directed multi-layer view populated during parsing:

```rust
Sentence {
    frames: vec![...],
    graph: Some(LinguisticGraph { nodes, edges }),
    sentence_node_id: None,
    // ...
}

Utterance {
    sentences: vec![...],
    discourse: Some(Discourse {
        entities_in_focus: vec![NodeId(...)],
        recent_mentions: vec![(NodeId(...), 0)],
        coref_edges: vec![EdgeId(...)],
        // ...
    }),
}
```

**Layers**: surface words (`WordNode` + `Next`/`Prev`), syntactic phrases (`PhraseNode`), semantic IL materialization (`EntityNode`, `FrameNode` + `Realizes`/`HasRole`), lexical concepts (`ConceptNode` + `EvokesConcept`).

**Invariant**: the graph must reflect the **final** IL frame after any post-parse frame adjustments (e.g. age idiom `BE`/`YEAR`). Materialize semantic nodes only after all frame mutations.

**Serialization**: IL RON omits graph by default (`skip_serializing_if`); benchmark traces export `GraphSnapshot` separately.
