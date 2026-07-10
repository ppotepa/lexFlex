# Language Descriptor — Declarative Language Description

A Language Descriptor is a **declarative specification** of how a particular language works. It tells the engine HOW to express Interlingua features in that language — without any procedural code.

The engine reads the descriptor and knows: what morphology exists, what word order to use, what grammatical features to encode, what can be omitted.

---

## How to Use Language Descriptor in Generator

The Language Descriptor drives generation decisions. Here's how to use it in practice.

**2026-07-10 Parallel Update (with ERRORS.MD iteration):** 
To achieve "strictly algorithmic" low-level processing (minimal hardcoding), descriptors + RON need extension for phonetic/article rules, default inference, virile, etc. See ERRORS.MD (new "Full Cross-Language (PL + EN)..." comprehensive audit section + "Required Low-Level Algorithmic Engines" + "RON extensions") for the exhaustive list of current leaks (article choice, degree, "ma", cross-lang leakage, NP structure, government) and proposed data extensions (PhoneticInitial, Prefix, surface_mappings.ron, suppletives). Keep in sync with GENERATOR.md, MORPHOLOGY.md, UNIFIED_ALGORITHMIC_GENERATION_PIPELINE.md, PLAN.md. Any descriptor change must trigger audit grep + doc sync.

### 1. Word Order Determination

```rust
fn determine_word_order(
    roles: &[(SemanticRole, String)],
    descriptor: &LanguageDescriptor,
) -> Vec<String> {
    match descriptor.word_order {
        WordOrder::SVO => {
            // Subject first, then verb, then objects
            let mut order = Vec::new();
            
            // Add subject (Agent)
            if let Some((_, subject)) = roles.iter().find(|(r, _)| *r == SemanticRole::Agent) {
                order.push(subject.clone());
            }
            
            // Verb will be added separately
            
            // Add indirect object (Recipient)
            if let Some((_, recipient)) = roles.iter().find(|(r, _)| *r == SemanticRole::Recipient) {
                order.push(recipient.clone());
            }
            
            // Add direct object (Theme)
            if let Some((_, theme)) = roles.iter().find(|(r, _)| *r == SemanticRole::Theme) {
                order.push(theme.clone());
            }
            
            order
        }
        
        WordOrder::SOV => {
            // Subject, objects, then verb
            let mut order = Vec::new();
            
            if let Some((_, subject)) = roles.iter().find(|(r, _)| *r == SemanticRole::Agent) {
                order.push(subject.clone());
            }
            
            if let Some((_, theme)) = roles.iter().find(|(r, _)| *r == SemanticRole::Theme) {
                order.push(theme.clone());
            }
            
            if let Some((_, recipient)) = roles.iter().find(|(r, _)| *r == SemanticRole::Recipient) {
                order.push(recipient.clone());
            }
            
            // Verb will be added at the end
            
            order
        }
        
        WordOrder::Free => {
            // Use information structure heuristics
            // Topic first, focus last
            determine_order_by_information_structure(roles, descriptor)
        }
        
        _ => {
            // Handle other word orders similarly
            vec![]
        }
    }
}
```

### 2. Pronoun Selection

```rust
fn select_pronoun_form(
    entity: &Entity,
    position_in_sentence: usize,
    descriptor: &LanguageDescriptor,
) -> String {
    match descriptor.pronoun_strategy {
        PronounStrategy::EncliticPreferred => {
            // Polish: use enclitic forms when possible
            if position_in_sentence > 0 && !is_emphatic_context(entity) {
                get_enclitic_form(entity)
            } else {
                get_full_form(entity)
            }
        }
        
        PronounStrategy::FullFormsOnly => {
            // English: always use full forms
            get_full_form(entity)
        }
        
        PronounStrategy::ProDropAllowed => {
            // If pro_drop is allowed and subject is clear, omit pronoun
            if descriptor.syntax.pro_drop && position_in_sentence == 0 {
                "".to_string() // Omit subject pronoun
            } else {
                get_full_form(entity)
            }
        }
    }
}
```

### 3. Article Selection (English-specific)

```rust
fn add_article(
    entity: &Entity,
    is_first_mention: bool,
    descriptor: &LanguageDescriptor,
) -> String {
    // Check if language has articles
    if !descriptor.morphology.has_articles {
        return "".to_string();
    }
    
    // Determine article based on definiteness
    match entity.features.definiteness {
        Some(Definiteness::Definite) => "the".to_string(),
        Some(Definiteness::Indefinite) => {
            if is_first_mention && entity.is_countable() {
                // Choose "a" or "an" based on phonetics
                let lemma = get_lemma(entity);
                if starts_with_vowel_sound(lemma) {
                    "an".to_string()
                } else {
                    "a".to_string()
                }
            } else {
                "".to_string()
            }
        }
        None => {
            // Default: use "a" for first mention
            if is_first_mention && entity.is_countable() {
                "a".to_string()
            } else {
                "".to_string()
            }
        }
    }
}
```

### 4. Case Assignment (Polish-specific)

```rust
fn assign_case(
    role: SemanticRole,
    frame_type: &str,
    polarity: Polarity,
    descriptor: &LanguageDescriptor,
) -> Option<Case> {
    // Check if language uses cases
    if !descriptor.morphology.has_cases {
        return None;
    }
    
    match (frame_type, role, polarity) {
        // Transfer frame
        ("Transfer", SemanticRole::Agent, _) => Some(Case::Nominative),
        ("Transfer", SemanticRole::Recipient, _) => Some(Case::Dative),
        ("Transfer", SemanticRole::Theme, Polarity::Positive) => Some(Case::Accusative),
        ("Transfer", SemanticRole::Theme, Polarity::Negative) => Some(Case::Genitive), // Polish-specific!
        
        // Motion frame
        ("Motion", SemanticRole::Agent, _) => Some(Case::Nominative),
        ("Motion", SemanticRole::Goal, _) => Some(Case::Accusative),
        ("Motion", SemanticRole::Source, _) => Some(Case::Genitive),
        
        // Perception frame
        ("Perception", SemanticRole::Experiencer, _) => Some(Case::Nominative),
        ("Perception", SemanticRole::Stimulus, _) => Some(Case::Accusative),
        
        _ => None,
    }
}
```

### 5. Verb Form Selection

Integrated into unified pipeline (see UNIFIED_ALGORITHMIC_GENERATION_PIPELINE.md). Descriptor drives common realizer.

```rust
fn select_verb_form(
    concept: &str,
    tense: Option<Tense>,
    aspect: Option<Aspect>,
    person: Option<Person>,
    number: Option<Number>,
    descriptor: &LanguageDescriptor,
) -> Result<String, GenerateError> {
    match descriptor.morphology.aspect_type {
        AspectType::Morphological => {
            // Polish: aspect is encoded in verb pairs
            let verb_entry = lookup_verb_with_aspect(concept, aspect)?;
            inflect_verb(verb_entry, tense, person, number)
        }
        
        AspectType::Periphrastic => {
            // English: aspect uses auxiliary verbs
            let base_verb = lookup_verb(concept)?;
            build_periphrastic_form(base_verb, tense, aspect, person, number)
        }
        
        AspectType::None => {
            // Language doesn't mark aspect
            let verb_entry = lookup_verb(concept)?;
            inflect_verb(verb_entry, tense, person, number)
        }
    }
}

fn build_periphrastic_form(
    verb: &str,
    tense: Option<Tense>,
    aspect: Option<Aspect>,
    person: Option<Person>,
    number: Option<Number>,
) -> String {
    match (tense, aspect) {
        (Some(Tense::Present), Some(Aspect::Progressive)) => {
            // "is giving"
            let aux = conjugate_auxiliary("be", person, number);
            let participle = verb.to_string() + "ing";
            format!("{} {}", aux, participle)
        }
        
        (Some(Tense::Past), Some(Aspect::Perfective)) => {
            // "gave" (simple past)
            conjugate_verb(verb, Tense::Past, person, number)
        }
        
        (Some(Tense::Past), Some(Aspect::Perfect)) => {
            // "has given"
            let aux = conjugate_auxiliary("have", person, number);
            let participle = get_past_participle(verb);
            format!("{} {}", aux, participle)
        }
        
        _ => {
            // Default: simple tense
            conjugate_verb(verb, tense.unwrap_or(Tense::Present), person, number)
        }
    }
}
```

### 6. Pro-Drop Decision

```rust
fn should_drop_subject(
    subject: &Entity,
    sentence_position: usize,
    descriptor: &LanguageDescriptor,
) -> bool {
    // Check if language allows pro-drop
    if !descriptor.syntax.pro_drop {
        return false;
    }
    
    // Only drop subject if it's at the beginning
    if sentence_position != 0 {
        return false;
    }
    
    // Don't drop if subject is emphatic or contrasted
    if subject.features.emphasis == Some(true) {
        return false;
    }
    
    // Don't drop if subject is a proper name (usually)
    if subject.name.is_some() {
        return false;
    }
    
    // Drop pronoun subjects when context is clear
    true
}
```

### 7. Complete Generator Using Descriptor

```rust
fn generate_with_descriptor(
    il: &InterlinguaNode,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<String, GenerateError> {
    
    match il {
        InterlinguaNode::Natural(utterance) => {
            let mut sentences = Vec::new();
            
            for sentence in &utterance.sentences {
                let generated = generate_sentence_with_descriptor(
                    sentence,
                    descriptor,
                    lexicon,
                    morphology,
                )?;
                sentences.push(generated);
            }
            
            Ok(sentences.join(" "))
        }
        
        _ => Err(GenerateError::UnsupportedInterlinguaType),
    }
}

fn generate_sentence_with_descriptor(
    sentence: &Sentence,
    descriptor: &LanguageDescriptor,
    lexicon: &Lexicon,
    morphology: &MorphologyEngine,
) -> Result<String, GenerateError> {
    
    let mut words = Vec::new();
    
    for frame in &sentence.frames {
        // Step 1: Analyze frame
        let roles = analyze_frame(frame);
        
        // Step 2: Determine word order using descriptor
        let ordered_roles = determine_word_order(&roles, descriptor);
        
        // Step 3: Generate each role
        for (i, (role, entity)) in ordered_roles.iter().enumerate() {
            // Handle subject pro-drop
            if i == 0 && role == &SemanticRole::Agent {
                if should_drop_subject(entity, i, descriptor) {
                    continue;
                }
            }
            
            // Add article if needed (English)
            let article = add_article(entity, is_first_mention(entity), descriptor);
            if !article.is_empty() {
                words.push(article);
            }
            
            // Assign case if language uses cases
            let case = assign_case(
                *role,
                &frame_type(frame),
                sentence.polarity,
                descriptor,
            );
            
            // Generate word form
            let mut features = entity.features.clone();
            if let Some(c) = case {
                features.case = Some(c);
            }
            
            let word = generate_entity(entity, &features, lexicon, morphology)?;
            words.push(word);
        }
        
        // Step 4: Generate verb
        let verb_form = select_verb_form(
            &get_verb_concept(frame),
            sentence.tense,
            sentence.aspect,
            get_subject_person(&roles),
            get_subject_number(&roles),
            descriptor,
        )?;
        
        // Insert verb at correct position based on word order
        insert_verb(&mut words, verb_form, descriptor);
    }
    
    // Step 5: Add temporal modifiers
    if let Some(temporal) = &sentence.temporal {
        let temporal_word = generate_temporal(temporal, lexicon)?;
        words.push(temporal_word);
    }
    
    Ok(words.join(" "))
}
```

---

## Polish Descriptor Example

```ron
LanguageDescriptor(
    language: "pl",
    name: "Polish",
    
    morphology: MorphologyDescriptor(
        morph_type: Fusional,
        has_cases: true,
        cases: [Nominative, Genitive, Dative, Accusative, Instrumental, Locative, Vocative],
        has_articles: false,
        aspect_type: Morphological, // Aspect encoded in verb pairs
    ),
    
    syntax: SyntaxDescriptor(
        word_order: SVO,
        word_order_flexibility: Flexible,
        pro_drop: true, // Can omit subject pronouns
    ),
    
    pragmatics: PragmaticsDescriptor(
        honorific_system: Politeness, // Pan/Pani vs ty
        pronoun_strategy: EncliticPreferred,
    ),
    
    classifier_system: None,
)
```

## English Descriptor Example

```ron
LanguageDescriptor(
    language: "en",
    name: "English",
    
    morphology: MorphologyDescriptor(
        morph_type: Analytic,
        has_cases: false,
        cases: [],
        has_articles: true,
        aspect_type: Periphrastic, // Aspect uses auxiliary verbs
    ),
    
    syntax: SyntaxDescriptor(
        word_order: SVO,
        word_order_flexibility: Strict,
        pro_drop: false, // Cannot omit subject
    ),
    
    pragmatics: PragmaticsDescriptor(
        honorific_system: None,
        pronoun_strategy: FullFormsOnly,
    ),
    
    classifier_system: None,
)
```

---

## Key Descriptor Fields for Generator

### Most Important Fields

1. **word_order** - Determines basic sentence structure
2. **word_order_flexibility** - How strictly to enforce word order
3. **has_cases** - Whether to assign grammatical cases
4. **has_articles** - Whether to add articles (a, the)
5. **aspect_type** - How to handle aspect (morphological vs periphrastic)
6. **pro_drop** - Whether subject can be omitted
7. **pronoun_strategy** - How to select pronoun forms

### Less Critical Fields (for v0.2+)

8. **honorific_system** - For formal/informal address
9. **classifier_system** - For languages with classifiers
10. **information_structure** - For topic/focus-based word order

---

## References

- [GENERATOR.md](./GENERATOR.md) - Generator implementation details
- [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) - Case assignment rules
- [MORPHOLOGY.md](./MORPHOLOGY.md) - Morphological rules


---

## Descriptor Structure

```rust
struct LanguageDescriptor {
    language: String,                    // "pl", "en"
    name: String,                        // "Polish", "English"

    morphology: MorphologyDescriptor,
    syntax: SyntaxDescriptor,
    pragmatics: PragmaticsDescriptor,
    
    // Word order configuration
    word_order: WordOrder,
    word_order_flexibility: WordOrderFlexibility,
    
    // Honorifics (JA, KO, JV: important; PL, EN: None)
    honorific_system: HonorificSystem,
    
    // Classifiers (ZH, JA, TH, VN: important; PL, EN: None)
    classifier_system: ClassifierSystem,
}

// ── Word order enums ──
enum WordOrder {
    SVO,    // EN, FR, ZH (basic)
    SOV,    // JA, KO, TR, DE (subordinate)
    VSO,    // AR, GA
    VOS,    // rare (Malagasy)
    OVS,    // very rare (Hixkaryana)
    OSV,    // very rare
    Free,   // PL, RU (depends on information structure)
}

enum WordOrderFlexibility {
    Strict,     // EN: SVO always (except questions)
    V2,         // DE: verb second in main clauses
    Flexible,   // PL: depends on topic/focus
    Free,       // very free (rare)
}

// ── Honorific system ──
enum HonorificSystem {
    None,           // EN, ZH: no grammatical honorifics
    Politeness,     // PL: Pan/Pani vs ty
    Complex,        // JA, KO, JV: multiple levels (plain, polite, honorific, humble)
}

// ── Classifier system ──
enum ClassifierSystem {
    None,           // PL, EN, RU, DE: no classifiers
    Optional,       // some languages use classifiers optionally
    Required,       // ZH, JA, TH, VN: classifiers required with numerals
}
```

**Examples:**

```
Polish:
  word_order: SVO
  word_order_flexibility: Flexible
  honorific_system: Politeness
  classifier_system: None

English:
  word_order: SVO
  word_order_flexibility: Strict
  honorific_system: None
  classifier_system: None

Japanese:
  word_order: SOV
  word_order_flexibility: Flexible
  honorific_system: Complex
  classifier_system: Required

Chinese:
  word_order: SVO
  word_order_flexibility: Strict
  honorific_system: None
  classifier_system: Required

German:
  word_order: SOV (subordinate), V2 (main)
  word_order_flexibility: V2
  honorific_system: Politeness
  classifier_system: None

Finnish:
  word_order: SVO
  word_order_flexibility: Flexible
  honorific_system: None
  classifier_system: None
```

---

## Morphology Descriptor

Describes what morphological categories the language has and how they are expressed.

```rust
struct MorphologyDescriptor {
    morph_type: MorphType,               // fusional, analytic, agglutinative
    
    noun_categories: NounCategories,
    verb_categories: VerbCategories,
    adj_categories: AdjCategories,
    
    // Language-specific morphological features
    special_features: Vec<SpecialFeature>,
}

enum MorphType {
    Fusional,        // PL, DE: one ending encodes multiple categories
    Analytic,        // EN: separate words for each category
    Agglutinative,   // TR, FI: one affix per category, stacked
}
```

### Noun Categories

```rust
struct NounCategories {
    gender: GenderSystem,
    number: Vec<Number>,
    case: Vec<Case>,                     // empty for EN
    definiteness: DefinitenessSystem,
}

enum GenderSystem {
    None,                                // EN: no grammatical gender
    Binary,                              // FR: masculine/feminine
    Tripartite,                          // PL: m/f/n (+ sub-genders)
    PolishFull,                          // PL: m_personal, m_animate, m_inanimate, f, n
}

// For PL:
// PolishFull genders:
//   MasculinePersonal    — male humans: chłopak, brat, nauczyciel
//   MasculineAnimate     — animals: kot, pies, koń
//   MasculineInanimate   — objects: dom, stół, komputer
//   Feminine             — kobieta, książka, noc
//   Neuter               — dziecko, jabłko, mięso
```

### Verb Categories

```rust
struct VerbCategories {
    tense: Vec<Tense>,
    aspect: AspectSystem,
    mood: Vec<Mood>,
    person: Vec<Person>,
    voice: Vec<Voice>,
    
    // How tense/aspect are expressed
    tense_expression: TenseExpression,
}

enum AspectSystem {
    None,                                // EN: aspect via auxiliary verbs
    LexicalPairs,                        // PL: perfective/imperfective verb pairs
    Morphological,                       // some languages: aspect is an inflection
}

enum TenseExpression {
    Synthetic,                           // PL: tense is an inflection ("dał")
    Analytic,                            // EN: tense via auxiliaries ("did give")
    Mixed,                               // EN: some synthetic ("gave"), some analytic ("has given")
}
```

### Definiteness System

```rust
enum DefinitenessSystem {
    None,                                // PL: no articles
    Articles {                           // EN: a/an + the
        definite: String,                // "the"
        indefinite: String,              // "a"
        indefinite_vowel: String,        // "an"
    },
}
```

---

## Syntax Descriptor

Describes word order, sentence structure, and grammatical constructions.

```rust
struct SyntaxDescriptor {
    word_order: WordOrder,
    flexibility: Flexibility,
    
    question_formation: QuestionFormation,
    negation: NegationSystem,
    pro_drop: bool,                      // can subject be omitted?
    
    coordination: CoordinationSystem,
}

struct WordOrder {
    default: BasicOrder,                 // SVO, SOV, VSO
    // For languages with flexible order, what determines deviations:
    information_structure: Option<InfoStructure>,
}

enum BasicOrder {
    SVO,     // PL, EN, FR, ES
    SOV,     // JA, KO, TR, DE (subordinate)
    VSO,     // AR, GA
}

enum Flexibility {
    Fixed,       // EN: word order is grammatically significant
    Flexible,    // PL: word order carries information, not grammar
    Free,        // some languages: truly free (rare)
}

enum InfoStructure {
    TopicFirst,  // PL: topic tends to come first, focus last
    TopicLast,   // some languages: new information first
}
```

### Question Formation

```rust
enum QuestionFormation {
    Particle,                            // PL: "Czy Tomek dał...?" or intonation only
    DoSupport,                           // EN: "Did Tomek give...?"
    Inversion,                           // FR: "Tomek a-t-il donné...?"
    WhFronting,                          // EN: "What did Tomek give...?"
    WhInSitu,                            // JA, ZH: wh-words stay in place
}

struct QuestionFormation {
    yes_no: YesNoQuestion,
    wh: WhQuestion,
}

// PL:
// yes_no: Particle { particle: "czy", position: SentenceInitial }
//         (or intonation-only: "Tomek dał jabłko?")
// wh: WhFronting + case marking on wh-word

// EN:
// yes_no: DoSupport { auxiliary: "do" }
// wh: WhFronting + DoSupport
```

### Negation

```rust
struct NegationSystem {
    particle: String,                    // PL: "nie", EN: "not"
    position: NegationPosition,
    
    // Morphological effects of negation
    case_effects: Vec<CaseEffect>,       // PL: negated ACC → GEN
    
    // Double negation
    double_negation: DoubleNegationType,
}

enum NegationPosition {
    PreVerbal,                           // PL: "nie dał"
    AuxiliaryBased,                      // EN: "did not give" (do-support)
}

struct CaseEffect {
    trigger: Case,                       // ACC
    result: Case,                        // GEN
    condition: NegCondition,             // direct object of negated transitive verb
}

// PL:
// negation: { particle: "nie", position: PreVerbal,
//             case_effects: [{trigger: ACC, result: GEN}] }
// EN:
// negation: { particle: "not", position: AuxiliaryBased,
//             case_effects: [] }  // no case changes
```

### Coordination

```rust
struct CoordinationSystem {
    conjunctions: Vec<ConjunctionEntry>,
}

struct ConjunctionEntry {
    lemma: String,
    type: ConjunctionType,              // And, Or, But, Contrast
    position: ConjunctionPosition,      // BetweenConjuncts, SentenceInitial
}

// PL conjunctions:
// "i" → And (additive)
// "a" → Contrast (whereas)
// "ale" → But (adversative)
// "lub" / "albo" → Or

// EN conjunctions:
// "and" → And
// "but" → But
// "or" → Or
// "while" / "whereas" → Contrast
```

---

## Pragmatics Descriptor

Describes how register, formality, and discourse conventions work.

```rust
struct PragmaticsDescriptor {
    formality: FormalitySystem,
    address: AddressSystem,
    diminutives: DiminutiveSystem,
    
    // Language-specific pragmatic features
    special_features: Vec<PragmaticFeature>,
}

struct FormalitySystem {
    levels: Vec<Register>,               // informal, neutral, formal
    triggers: Vec<FormalityTrigger>,     // what signals each level
}

enum FormalityTrigger {
    HonorificNoun,                       // PL: "Pan/Pani" + 3sg verb
    HonorificPronoun,                    // DE: "Sie" (formal you)
    ModalVerb,                           // EN: "could you...", "would you..."
    VerbPerson,                          // PL: formal = 3sg, informal = 2sg
}

struct AddressSystem {
    informal: AddressForm,               // PL: "ty" + 2sg verb
    formal: AddressForm,                 // PL: "Pan/Pani" + 3sg verb
    plural_formal: Option<AddressForm>,  // PL: "Państwo" + 3pl verb
}

struct AddressForm {
    pronoun: String,                     // "ty", "Pan", "you"
    verb_person: Person,                 // 2sg, 3sg
    verb_number: Number,                 // Sg, Pl
}

enum DiminutiveSystem {
    None,                                // EN: not productive
    Productive {                         // PL: regular diminutive formation
        suffixes: Vec<DiminutiveSuffix>,
    },
}

// PL diminutives:
// dom → domek, książka → książeczka, kot → kotek
// These are morphologically productive, stored as rules
```

---

## Full Descriptor Examples

### Polish Descriptor

```rust
// data/descriptors/pl.ron

LanguageDescriptor(
    language: "pl",
    name: "Polish",

    morphology: MorphologyDescriptor(
        morph_type: Fusional,

        noun_categories: NounCategories(
            gender: PolishFull,
            // genders: MasculinePersonal, MasculineAnimate,
            //          MasculineInanimate, Feminine, Neuter
            number: [Singular, Plural],
            case: [Nominative, Genitive, Dative, Accusative,
                   Instrumental, Locative, Vocative],
            definiteness: None,          // no articles in Polish
        ),

        verb_categories: VerbCategories(
            tense: [Past, Present, Future],
            aspect: LexicalPairs,        // perf/impf verb pairs
            mood: [Indicative, Imperative, Conditional],
            person: [First, Second, Third],
            voice: [Active],             // passive out of MVP scope
            tense_expression: Synthetic, // tense via inflection
        ),

        adj_categories: AdjCategories(
            degree: [Positive, Comparative, Superlative],
            agrees_with: Noun,           // gender + number + case
        ),

        special_features: [
            // Past tense verb encodes gender of subject
            PastGenderAgreement,
            // Virile vs non-virile distinction in plural
            VirileDistinction,
            // Negation changes ACC → GEN
            NegationCaseShift,
        ],
    ),

    syntax: SyntaxDescriptor(
        word_order: WordOrder(
            default: SVO,
            information_structure: Some(TopicFirst),
        ),
        flexibility: Flexible,           // word order is free but meaningful

        question_formation: QuestionFormation(
            yes_no: Particle { particle: "czy", optional: true },
            wh: WhFronting,
        ),

        negation: NegationSystem(
            particle: "nie",
            position: PreVerbal,
            case_effects: [
                CaseEffect {
                    trigger: Accusative,
                    result: Genitive,
                    condition: DirectObjectOfNegatedTransitive,
                },
            ],
            double_negation: Required,   // "nikt nie przyszedł" (nobody didn't come)
        ),

        pro_drop: true,                  // subject can be omitted
        coordination: CoordinationSystem(
            conjunctions: [
                ConjunctionEntry { lemma: "i", type: And, position: Between },
                ConjunctionEntry { lemma: "a", type: Contrast, position: Between },
                ConjunctionEntry { lemma: "ale", type: But, position: Between },
                ConjunctionEntry { lemma: "lub", type: Or, position: Between },
                ConjunctionEntry { lemma: "albo", type: Or, position: Between },
            ],
        ),
    ),

    pragmatics: PragmaticsDescriptor(
        formality: FormalitySystem(
            levels: [Informal, Neutral, Formal],
            triggers: [
                HonorificNoun,           // "Pan/Pani"
                VerbPerson,              // formal = 3sg
            ],
        ),
        address: AddressSystem(
            informal: AddressForm {
                pronoun: "ty",
                verb_person: Second,
                verb_number: Singular,
            },
            formal: AddressForm {
                pronoun: "Pan",          // or "Pani" for female
                verb_person: Third,
                verb_number: Singular,
            },
            plural_formal: Some(AddressForm {
                pronoun: "Państwo",
                verb_person: Third,
                verb_number: Plural,
            }),
        ),
        diminutives: Productive {
            suffixes: [
                DiminutiveSuffix { from: "dom", to: "domek", suffix: "ek" },
                // Algorithmic rules, not exhaustive list
            ],
        },
        special_features: [],
    ),
)
```

### English Descriptor

```rust
// data/descriptors/en.ron

LanguageDescriptor(
    language: "en",
    name: "English",

    morphology: MorphologyDescriptor(
        morph_type: Analytic,

        noun_categories: NounCategories(
            gender: None,                // no grammatical gender
            number: [Singular, Plural],
            case: [],                    // vestigial: I/me, he/him only
            definiteness: Articles {
                definite: "the",
                indefinite: "a",
                indefinite_vowel: "an",
            },
        ),

        verb_categories: VerbCategories(
            tense: [Present, Past, Future],
            aspect: None,                // aspect via auxiliaries (be + -ing, have + -en)
            mood: [Indicative, Imperative, Subjunctive],
            person: [First, Second, Third],  // but only 3sg has -s
            voice: [Active, Passive],
            tense_expression: Mixed,     // "gave" (synthetic) + "will give" (analytic)
        ),

        adj_categories: AdjCategories(
            degree: [Positive, Comparative, Superlative],
            agrees_with: None,           // adjectives don't inflect for agreement
        ),

        special_features: [
            // 3rd person singular present: -s
            ThirdSgPresent,
            // Do-support for negation and questions
            DoSupport,
            // Phrasal verbs are productive
            PhrasalVerbs,
        ],
    ),

    syntax: SyntaxDescriptor(
        word_order: WordOrder(
            default: SVO,
            information_structure: None, // word order is fixed, not info-structured
        ),
        flexibility: Fixed,              // changing order changes meaning

        question_formation: QuestionFormation(
            yes_no: DoSupport { auxiliary: "do" },
            wh: WhFrontingWithDoSupport,
        ),

        negation: NegationSystem(
            particle: "not",
            position: AuxiliaryBased,    // "do not give", "cannot give"
            case_effects: [],            // no case changes in English
            double_negation: Forbidden,  // standard English: single negation
        ),

        pro_drop: false,                 // subject is required ("It rains")
        coordination: CoordinationSystem(
            conjunctions: [
                ConjunctionEntry { lemma: "and", type: And, position: Between },
                ConjunctionEntry { lemma: "but", type: But, position: Between },
                ConjunctionEntry { lemma: "or", type: Or, position: Between },
                ConjunctionEntry { lemma: "while", type: Contrast, position: Between },
            ],
        ),
    ),

    pragmatics: PragmaticsDescriptor(
        formality: FormalitySystem(
            levels: [Informal, Neutral, Formal],
            triggers: [
                ModalVerb,               // "could you...", "would you mind..."
            ],
        ),
        address: AddressSystem(
            informal: AddressForm {
                pronoun: "you",
                verb_person: Second,
                verb_number: Singular,   // EN: same form for sg/pl "you"
            },
            formal: AddressForm {
                pronoun: "you",          // EN: same pronoun, different tone
                verb_person: Second,
                verb_number: Singular,
            },
            plural_formal: None,
        ),
        diminutives: None,               // not productive in English
        special_features: [
            // Politeness via modal verbs
            ModalPoliteness,
        ],
    ),
)
```

---

## How the Engine Uses the Descriptor

The descriptor is **read-only data** — the engine code interprets it:

```rust
fn generate_sentence(il: &Sentence, descriptor: &LanguageDescriptor) -> String {
    // 1. Determine word order
    let order = descriptor.syntax.word_order.default;
    
    // 2. For each entity, determine surface form
    for entity in il.entities() {
        let case = resolve_case(entity.role, il.polarity, descriptor);
        let form = inflect(entity, case, descriptor.morphology);
    }
    
    // 3. Handle negation
    if il.polarity == Negative {
        apply_negation(descriptor.syntax.negation);
        // PL: add "nie" before verb + ACC→GEN shift
        // EN: add "do not" / "did not" auxiliary
    }
    
    // 4. Handle pro-drop
    if descriptor.syntax.pro_drop && topic_is_clear() {
        omit_subject();     // PL: "Dał jabłko"
    } else {
        require_subject();  // EN: "He gave an apple"
    }
    
    // 5. Handle articles
    if let Articles { definite, indefinite } = descriptor.morphology.noun_categories.definiteness {
        add_articles(il.entities, definite, indefinite);
        // EN: "the apple" or "an apple"
        // PL: no-op (definiteness: None)
    }
    
    // 6. Assemble sentence
    assemble(order, forms, descriptor);
}
```

---

## Adding a New Language

To add a new language, you need:

1. **New descriptor** — `data/descriptors/<lang>.ron`
2. **New lexicon** — `data/lexicons/<lang>/lexicon.ron`
3. **New morphology rules** — `data/morphology/<lang>/`
4. **New engine plugin** — `src/engines/<lang>/`

The core Interlingua, deduction, and other engines remain **unchanged**. The descriptor-driven architecture means the same generation logic works for any language that has a proper descriptor.
