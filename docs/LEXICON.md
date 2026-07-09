# Lexicon — Structure and Format

The lexFlex lexicon has a **hub-and-spoke** architecture. Interlingua concepts form the **hub** (master list). Each language has a **sub-lexicon** (spoke) that maps its words to those concepts.

```
                    ┌──────────────────────┐
                    │  MASTER CONCEPTS      │
                    │  (Interlingua)        │
                    │                      │
                    │  GIVE, APPLE, PERSON, │
                    │  BIG, GO, LOVE, ...   │
                    │  ~500 concepts        │
                    └──────┬───────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ PL Sub-  │ │ EN Sub-  │ │ DE Sub-  │
        │ Lexicon  │ │ Lexicon  │ │ Lexicon  │
        │          │ │          │ │          │
        │ "dawać"  │ │ "give"   │ │ "geben"  │
        │ "dać"    │ │          │ │          │
        │ "jabłko" │ │ "apple"  │ │ "Apfel"  │
        └──────────┘ └──────────┘ └──────────┘
```

---

## Master Concepts

### Location

`data/concepts/concepts.ron`

### Purpose

The master concept list defines **what meanings exist** in the system, independent of any language. Each concept specifies:

- Its semantic category (entity, action, property, relation)
- What frame it participates in (for actions)
- Inherent features
- What semantic roles it can fill

### Concept Categories

```rust
enum ConceptCategory {
    Entity,        // nouns: PERSON, APPLE, HOUSE
    Action,        // verbs: GIVE, GO, SEE
    Property,      // adjectives: BIG, RED, GOOD
    Relation,      // prepositions/conceptual: IN, ON, WITH
    Function,      // grammatical words: AND, OR, NOT, QUESTION_MARKER
    Deictic,       // context-dependent: THIS, THAT, HERE, NOW
}
```

### Master Concept Format (RON)

```rust
// data/concepts/concepts.ron

MasterLexicon(
    version: "0.1.0",
    concepts: {
        // ─── ENTITIES ───────────────────────────────
        PERSON: Concept(
            category: Entity,
            inherent_features: FeatureBundle(
                animacy: Some(Animate),
            ),
            // Can play these roles in frames:
            playable_roles: [Agent, Recipient, Experiencer, Speaker],
        ),

        APPLE: Concept(
            category: Entity,
            inherent_features: FeatureBundle(
                concreteness: Some(Concrete),
                countability: Some(Count),
            ),
            playable_roles: [Theme, Patient, Stimulus],
        ),

        BOOK: Concept(
            category: Entity,
            inherent_features: FeatureBundle(
                concreteness: Some(Concrete),
                countability: Some(Count),
            ),
            playable_roles: [Theme, Patient, Stimulus],
        ),

        // ─── ACTIONS ────────────────────────────────
        GIVE: Concept(
            category: Action,
            frame: Transfer,
            required_roles: [Agent, Recipient, Theme],
            optional_roles: [Instrument, Beneficiary],
        ),

        GO: Concept(
            category: Action,
            frame: Motion,
            required_roles: [Agent],           // mover
            optional_roles: [Source, Goal, Path],
        ),

        SEE: Concept(
            category: Action,
            frame: Perception,
            required_roles: [Experiencer, Stimulus],
            optional_roles: [],
        ),

        THINK: Concept(
            category: Action,
            frame: Cognition,
            required_roles: [Experiencer, Stimulus],
            optional_roles: [],
        ),

        LOVE: Concept(
            category: Action,
            frame: Emotion,
            required_roles: [Experiencer, Stimulus],
            optional_roles: [],
        ),

        EAT: Concept(
            category: Action,
            frame: Destruction,
            required_roles: [Agent, Patient],
            optional_roles: [Instrument],
        ),

        SAY: Concept(
            category: Action,
            frame: Communication,
            required_roles: [Agent],           // speaker
            optional_roles: [Recipient, Theme],// addressee, message
        ),

        // ─── PROPERTIES ─────────────────────────────
        BIG: Concept(
            category: Property,
            inherent_features: FeatureBundle(),
            applicable_to: [Entity],           // can modify entities
            gradable: true,                    // big → bigger → biggest
        ),

        GOOD: Concept(
            category: Property,
            inherent_features: FeatureBundle(),
            applicable_to: [Entity, Action],
            gradable: true,
        ),

        RED: Concept(
            category: Property,
            inherent_features: FeatureBundle(),
            applicable_to: [Entity],
            gradable: false,                   // no "redder"
        ),

        // ─── FUNCTIONS ──────────────────────────────
        AND: Concept(
            category: Function,
            function_type: Conjunction(And),
        ),

        BUT: Concept(
            category: Function,
            function_type: Conjunction(But),
        ),

        NOT: Concept(
            category: Function,
            function_type: Negation,
        ),
    },
)
```

### Concept Target: ~500 for MVP

| Category | Count | Examples |
|----------|-------|---------|
| Entity | ~120 | PERSON, ANIMAL, APPLE, BOOK, HOUSE, WATER, MOTHER, CHILD, CITY, DAY... |
| Action | ~150 | GIVE, TAKE, GO, COME, SEE, HEAR, THINK, KNOW, LOVE, HATE, EAT, DRINK, SAY, ASK, MAKE, BREAK, WORK, PLAY, SLEEP, WALK, RUN, READ, WRITE, BUY, SELL... |
| Property | ~80 | BIG, SMALL, GOOD, BAD, NEW, OLD, LONG, SHORT, HOT, COLD, RED, BLUE, GREEN, FAST, SLOW, EASY, HARD, HAPPY, SAD, BEAUTIFUL, UGLY... |
| Relation | ~40 | IN, ON, UNDER, ABOVE, NEAR, BETWEEN, BEFORE, AFTER, WITH, WITHOUT, FOR, ABOUT, THROUGH... |
| Function | ~30 | AND, OR, BUT, IF, BECAUSE, NOT, QUESTION, THIS, THAT, HERE, THERE, NOW, THEN, ALL, SOME, NONE, EACH... |
| Deictic | ~20 | I, YOU, HE, SHE, IT, WE, THEY, THIS, THAT, HERE, THERE, NOW, THEN, TODAY, TOMORROW, YESTERDAY... |

---

## Sub-Lexicons

### Location

`data/lexicons/<lang>/lexicon.ron`

### Purpose

Each sub-lexicon maps **language-specific words** to **master concepts**. A single concept may map to multiple words (synonyms, aspect pairs), and a single word may map to multiple concepts (polysemy — resolved by deduction).

### Sub-Lexicon Entry Format

Each entry contains:

1. **Word forms** — all surface forms (lemma + inflected forms for irregular words)
2. **Morphological data** — paradigm class, gender, animacy, etc.
3. **Syntactic data** — subcategorization frame (what cases/roles it requires)
4. **Semantic link** — which master concept(s) it maps to
5. **Usage notes** — register, collocations, restrictions

### Polish Sub-Lexicon Examples

```rust
// data/lexicons/pl/lexicon.ron

SubLexicon(
    language: "pl",
    version: "0.1.0",
    entries: {
        // ─── NOUNS ──────────────────────────────────

        // Maps to concept: APPLE
        APPLE: LexEntry(
            lemma: "jabłko",
            pos: Noun,
            morphology: NounMorph(
                gender: Neuter,
                paradigm: NounParadigm("o"),        // declension class
                animacy: None,                       // neuter = no animacy
            ),
            senses: [
                Sense(
                    concept: APPLE,                  // → master concept
                    register: Neutral,
                    collocations: [],                // common word pairings
                ),
            ],
        ),

        // Maps to concept: BOOK
        BOOK: LexEntry(
            lemma: "książka",
            pos: Noun,
            morphology: NounMorph(
                gender: Feminine,
                paradigm: NounParadigm("ka"),
                animacy: None,
            ),
            senses: [
                Sense(
                    concept: BOOK,
                    register: Neutral,
                    collocations: [],
                ),
            ],
        ),

        // Polysemous: "bank" = financial institution OR river bank
        // → Two senses, two different concepts
        BANK: LexEntry(
            lemma: "bank",
            pos: Noun,
            morphology: NounMorph(
                gender: MasculineInanimate,
                paradigm: NounParadigm("consonant_hard"),
                animacy: Inanimate,
            ),
            senses: [
                Sense(
                    concept: FINANCIAL_INSTITUTION,
                    register: Neutral,
                    collocations: ["w banku", "z banku"],
                ),
                // Note: PL uses "brzeg" for river bank, not "bank"
            ],
        ),

        // ─── VERBS ──────────────────────────────────

        // Aspect pair: dawać (impf) / dać (perf)
        GIVE: LexEntry(
            lemma: "dać",
            pos: Verb,
            morphology: VerbMorph(
                aspect: Perfective,
                partner: Some("dawać"),              // imperfective partner
                paradigm: VerbParadigm("ać_irreg"),  // "dać" is slightly irregular
            ),
            syntax: SubcatFrame(
                roles: [
                    RoleCase(Agent, Nominative),     // kto daje
                    RoleCase(Recipient, Dative),     // komu daje
                    RoleCase(Theme, Accusative),     // co daje
                ],
            ),
            senses: [
                Sense(
                    concept: GIVE,
                    register: Neutral,
                    collocations: [],
                ),
            ],
        ),

        GIVE_IMPF: LexEntry(
            lemma: "dawać",
            pos: Verb,
            morphology: VerbMorph(
                aspect: Imperfective,
                partner: Some("dać"),
                paradigm: VerbParadigm("ać"),        // regular -ać conjugation
            ),
            syntax: SubcatFrame(
                roles: [
                    RoleCase(Agent, Nominative),
                    RoleCase(Recipient, Dative),
                    RoleCase(Theme, Accusative),
                ],
            ),
            senses: [
                Sense(concept: GIVE, register: Neutral, collocations: []),
            ],
        ),

        // Verb that requires GENITIVE (not accusative) for its object
        SEARCH: LexEntry(
            lemma: "szukać",
            pos: Verb,
            morphology: VerbMorph(
                aspect: Imperfective,
                partner: Some("poszukać"),
                paradigm: VerbParadigm("ać"),
            ),
            syntax: SubcatFrame(
                roles: [
                    RoleCase(Agent, Nominative),
                    RoleCase(Theme, Genitive),       // kogo/czego szuka — GEN!
                ],
            ),
            senses: [
                Sense(concept: SEARCH, register: Neutral, collocations: []),
            ],
        ),

        // ─── ADJECTIVES ─────────────────────────────

        BIG: LexEntry(
            lemma: "duży",
            pos: Adjective,
            morphology: AdjMorph(
                paradigm: AdjParadigm("y"),          // standard -y adjective
                gradable: true,
                comparative: "większy",              // irregular comparative
            ),
            senses: [
                Sense(concept: BIG, register: Neutral, collocations: []),
            ],
        ),

        GOOD: LexEntry(
            lemma: "dobry",
            pos: Adjective,
            morphology: AdjMorph(
                paradigm: AdjParadigm("ry"),
                gradable: true,
                comparative: "lepszy",               // suppletive
            ),
            senses: [
                Sense(concept: GOOD, register: Neutral, collocations: []),
            ],
        ),

        // ─── PRONOUNS ───────────────────────────────

        // Personal pronouns are special — they don't map to a single concept
        // but carry features for reference resolution
        I: LexEntry(
            lemma: "ja",
            pos: Pronoun,
            morphology: PronounMorph(
                person: First,
                number: Singular,
                // Forms across all cases — pronouns are always irregular
                forms: {
                    Nominative: "ja",
                    Genitive: "mnie",
                    Dative: "mi",
                    Accusative: "mnie",
                    Instrumental: "mną",
                    Locative: "mnie",
                },
            ),
            senses: [
                Sense(concept: PERSON, register: Neutral, collocations: []),
            ],
        ),

        SHE: LexEntry(
            lemma: "ona",
            pos: Pronoun,
            morphology: PronounMorph(
                person: Third,
                number: Singular,
                gender: Feminine,
                forms: {
                    Nominative: "ona",
                    Genitive: "jej",
                    Dative: "jej",
                    Accusative: "ją",
                    Instrumental: "nią",
                    Locative: "niej",
                },
            ),
            senses: [
                Sense(concept: PERSON, register: Neutral, collocations: []),
            ],
        ),

        // ─── FUNCTION WORDS ─────────────────────────

        NOT: LexEntry(
            lemma: "nie",
            pos: Particle,
            morphology: None,
            syntax: NegationParticle(
                position: PreVerbal,                 // "nie dał"
                case_effect: AccToGen,              // negation changes ACC → GEN
            ),
            senses: [
                Sense(concept: NOT, register: Neutral, collocations: []),
            ],
        ),

        AND: LexEntry(
            lemma: "i",
            pos: Conjunction,
            morphology: None,
            senses: [
                Sense(concept: AND, register: Neutral, collocations: []),
            ],
        ),

        CONTRAST: LexEntry(
            lemma: "a",
            pos: Conjunction,
            morphology: None,
            senses: [
                Sense(concept: BUT, register: Neutral, collocations: []),
                // "a" in PL is contrastive "whereas", not adversative "but" (="ale")
            ],
        ),
    },
)
```

### English Sub-Lexicon Examples

```rust
// data/lexicons/en/lexicon.ron

SubLexicon(
    language: "en",
    version: "0.1.0",
    entries: {
        APPLE: LexEntry(
            lemma: "apple",
            pos: Noun,
            morphology: NounMorph(
                paradigm: NounParadigm("regular"),   // apple → apples
                countability: Count,
            ),
            senses: [
                Sense(concept: APPLE, register: Neutral, collocations: []),
            ],
        ),

        GIVE: LexEntry(
            lemma: "give",
            pos: Verb,
            morphology: VerbMorph(
                paradigm: VerbParadigm("irregular"),
                forms: IrregularForms(
                    base: "give",
                    past: "gave",
                    past_participle: "given",
                    third_sg: "gives",
                    present_participle: "giving",
                ),
            ),
            syntax: SubcatFrame(
                roles: [
                    RolePosition(Agent, Subject),     // who gives
                    RolePosition(Recipient, IndirectObject), // to whom
                    RolePosition(Theme, DirectObject),       // what
                ],
                // English ditransitive: "give [IO] [DO]" or "give [DO] to [IO]"
                alternations: [
                    Ditransitive,                    // "give her an apple"
                    PrepositionalDative,             // "give an apple to her"
                ],
            ),
            senses: [
                Sense(concept: GIVE, register: Neutral, collocations: []),
            ],
        ),

        // Phrasal verb — multiple words, one concept
        GIVE_UP: LexEntry(
            lemma: "give up",
            pos: Verb,
            morphology: VerbMorph(
                paradigm: VerbParadigm("phrasal"),
                particle: "up",
                forms: IrregularForms(
                    base: "give up",
                    past: "gave up",
                    past_participle: "given up",
                    third_sg: "gives up",
                    present_participle: "giving up",
                ),
            ),
            syntax: SubcatFrame(
                roles: [
                    RolePosition(Agent, Subject),
                ],
            ),
            senses: [
                Sense(concept: SURRENDER, register: Neutral, collocations: []),
            ],
        ),

        // Articles — EN-specific grammatical words
        THE: LexEntry(
            lemma: "the",
            pos: Article,
            morphology: ArticleMorph(
                definiteness: Definite,
            ),
            senses: [],                               // grammatical, not conceptual
        ),

        A_AN: LexEntry(
            lemma: "a",
            pos: Article,
            morphology: ArticleMorph(
                definiteness: Indefinite,
                allomorphs: { "a": BeforeConsonant, "an": BeforeVowel },
            ),
            senses: [],
        ),
    },
)
```

---

## Key Design Decisions

### 1. Concept IDs are stable, words are not

The concept `GIVE` exists regardless of language. Polish maps it to "dać"/"dawać", English to "give", German to "geben". If a language has no word for a concept, it simply doesn't have an entry — the concept still exists in the master list.

### 2. Polysemy = multiple senses per entry

A single lemma can have multiple `Sense` entries, each pointing to a different concept. The parser uses context to disambiguate:

```
"bank" (PL) has 1 sense → FINANCIAL_INSTITUTION
"bank" (EN) has 2 senses → FINANCIAL_INSTITUTION, RIVER_BANK
```

### 3. Aspect pairs are separate entries

In Polish, "dawać" (impf) and "dać" (perf) are separate lexicon entries that link to the same concept (GIVE) but carry different `partner` references. This allows the engine to select the correct form based on the aspect in the Interlingua.

### 4. Subcategorization frames are per-language

The same concept (GIVE) has different syntactic requirements in each language:

```
PL: "dać" → [Agent:NOM, Recipient:DAT, Theme:ACC]
EN: "give" → [Agent:Subject, Recipient:IndirectObj, Theme:DirectObj]
```

### 5. Morphological data drives inflection

Each entry carries enough morphological information for the morphology engine to generate all word forms algorithmically. Irregular forms are listed explicitly; regular forms follow paradigm rules.

See: [MORPHOLOGY.md](./MORPHOLOGY.md) for how paradigms work.

---

## Lexicon Lookup Pipeline

### Parsing (surface → concept)

```
Input word: "jabłko"
    │
    ├─→ 1. Exact lemma match → found: APPLE entry
    ├─→ 2. Try inflected form match → "jabłko" matches NOM.SG of "jabłko"
    ├─→ 3. If ambiguous → return all candidates + features
    └─→ 4. Deduction picks the right sense based on context
```

### Generation (concept → surface)

```
Concept: APPLE, features: [N, SG, ACC]
    │
    ├─→ 1. Look up APPLE in target language lexicon
    ├─→ 2. Get lemma: "jabłko"
    ├─→ 3. Apply morphology: N.SG.ACC of "jabłko" → "jabłko" (neuter ACC = NOM form)
    └─→ 4. Return "jabłko"
```

---

## Polysemy Resolution

Many words have multiple meanings (polysemy). The lexicon must represent all senses, and the parser must disambiguate based on context.

### Representing Multiple Senses

```rust
// A single lexicon entry can have multiple senses
LexEntry(
    lemma: "bank",
    pos: "Noun",
    senses: vec![
        Sense {
            concept: "FINANCIAL_INSTITUTION",
            features: FeatureBundle {
                gender: Some(Masculine),
                countability: Some(Count),
                ..Default::default()
            },
            selectional_restrictions: vec![
                // Co-occurs with: pieniądze, konto, kredyt
            ],
        },
        Sense {
            concept: "RIVER_BANK",
            features: FeatureBundle {
                gender: Some(Masculine),
                countability: Some(Count),
                ..Default::default()
            },
            selectional_restrictions: vec![
                // Co-occurs with: rzeka, woda, brzeg
            ],
        },
    ],
)
```

### Disambiguation Strategies

**Strategy 1: Selectional Restrictions**

```rust
pub fn disambiguate_by_selectional_restrictions(
    word: &str,
    senses: &[Sense],
    context: &Sentence,
) -> Option<&Sense> {
    for sense in senses {
        // Check if context words match selectional restrictions
        if sense.selectional_restrictions.iter().any(|restriction| {
            context.contains_word(restriction)
        }) {
            return Some(sense);
        }
    }
    None
}
```

**Example:**
```
"I went to the bank to deposit money"
  → "bank" has 2 senses: FINANCIAL_INSTITUTION, RIVER_BANK
  → Context contains "deposit", "money"
  → Selectional restriction: FINANCIAL_INSTITUTION co-occurs with "pieniądze"
  → Selected: FINANCIAL_INSTITUTION

"I sat on the bank of the river"
  → "bank" has 2 senses: FINANCIAL_INSTITUTION, RIVER_BANK
  → Context contains "river", "sat"
  → Selectional restriction: RIVER_BANK co-occurs with "rzeka"
  → Selected: RIVER_BANK
```

**Strategy 2: Frame Compatibility**

```rust
pub fn disambiguate_by_frame(
    senses: &[Sense],
    frame: &Frame,
) -> Option<&Sense> {
    for sense in senses {
        // Check if sense concept fits the frame
        if frame.is_compatible_with(&sense.concept) {
            return Some(sense);
        }
    }
    None
}
```

**Example:**
```
"bank dał kredyt" (bank gave a loan)
  → Frame: TRANSFER { agent: bank, theme: kredyt }
  → FINANCIAL_INSTITUTION can be Agent of TRANSFER
  → RIVER_BANK cannot be Agent of TRANSFER
  → Selected: FINANCIAL_INSTITUTION
```

**Strategy 3: Semantic Similarity**

```rust
pub fn disambiguate_by_similarity(
    senses: &[Sense],
    context_words: &[String],
    ontology: &Ontology,
) -> Option<&Sense> {
    let mut best_sense = None;
    let mut best_score = 0.0;
    
    for sense in senses {
        let score = context_words.iter()
            .map(|word| ontology.semantic_similarity(&sense.concept, word))
            .sum::<f64>();
        
        if score > best_score {
            best_score = score;
            best_sense = Some(sense);
        }
    }
    
    best_sense
}
```

### Common Polysemous Words

| Word | Sense 1 | Sense 2 | Sense 3 |
|------|---------|---------|---------|
| "bank" | financial institution | river bank | — |
| "zamek" | castle | lock (door) | zipper |
| "klucz" | key (door) | key (answer) | clef (music) |
| "karta" | card (payment) | card (game) | menu (restaurant) |
| "list" | letter (mail) | list (items) | — |
| "ręcznik" | towel | — | — |
| "gwiazda" | star (celestial) | star (celebrity) | star (shape) |
| "korona" | crown | corona (virus) | currency |

---

## Collocations

Collocations are word combinations that occur together more often than by chance. They should be stored in the lexicon.

### Representing Collocations

```rust
LexEntry(
    lemma: "mocny",
    pos: "Adjective",
    senses: vec![
        Sense {
            concept: "STRONG",
            collocations: vec![
                Collocation {
                    partner: "kawa",
                    frequency: High,
                    meaning: "strong coffee",
                },
                Collocation {
                    partner: "herbata",
                    frequency: High,
                    meaning: "strong tea",
                },
                Collocation {
                    partner: "wiatr",
                    frequency: Medium,
                    meaning: "strong wind",
                },
            ],
        },
    ],
)
```

### Collocation Detection

```rust
pub fn check_collocation(
    adj: &str,
    noun: &str,
    lexicon: &Lexicon,
) -> CollocationResult {
    let adj_entry = lexicon.get(adj)?;
    
    for sense in &adj_entry.senses {
        for collocation in &sense.collocations {
            if collocation.partner == noun {
                return CollocationResult::Valid {
                    meaning: collocation.meaning.clone(),
                    frequency: collocation.frequency,
                };
            }
        }
    }
    
    CollocationResult::Unusual
}
```

### Common Collocations (Polish)

| Adjective | Noun | Meaning |
|-----------|------|---------|
| "mocny" | "kawa" | strong coffee |
| "mocny" | "herbata" | strong tea |
| "mocny" | "wiatr" | strong wind |
| "silny" | "człowiek" | strong person |
| "silny" | "ból" | strong pain |
| "duży" | "dom" | big house |
| "duży" | "problem" | big problem |
| "wysoki" | "człowiek" | tall person |
| "wysoki" | "budynek" | tall building |
| "dobry" | "przyjaciel" | good friend |
| "dobry" | "pomysł" | good idea |

### Collocation Errors

```
❌ "silna kawa" (strong coffee - WRONG)
✅ "mocna kawa" (strong coffee - CORRECT)

❌ "mocny człowiek" (strong person - WRONG)
✅ "silny człowiek" (strong person - CORRECT)

❌ "wysoki problem" (tall problem - WRONG)
✅ "duży problem" (big problem - CORRECT)
```

---

## Idioms

Idioms are fixed expressions where the meaning cannot be derived from individual words. They must be stored as complete units.

### Representing Idioms

```rust
LexEntry(
    lemma: "dać plamę",
    pos: "Idiom",
    senses: vec![
        Sense {
            concept: "FAIL",
            literal_meaning: "to give a stain",
            figurative_meaning: "to fail, to make a mistake",
            register: Informal,
            examples: vec![
                "Dałem plamę na egzaminie" (I failed the exam),
            ],
        },
    ],
)
```

### Idiom Detection

```rust
pub fn detect_idiom(
    tokens: &[String],
    lexicon: &Lexicon,
) -> Option<IdiomMatch> {
    // Check for multi-word idioms
    for window_size in (2..=5).rev() {
        for window in tokens.windows(window_size) {
            let phrase = window.join(" ");
            
            if let Some(entry) = lexicon.get(&phrase) {
                if entry.pos == "Idiom" {
                    return Some(IdiomMatch {
                        phrase,
                        entry: entry.clone(),
                        span: window.len(),
                    });
                }
            }
        }
    }
    
    None
}
```

### Common Idioms (Polish)

| Idiom | Literal | Figurative |
|-------|---------|------------|
| "dać plamę" | to give a stain | to fail |
| "mieć głowę w chmurach" | to have head in clouds | to daydream |
| "robić z igły widły" | to make forks from needle | to exaggerate |
| "lać wodę" | to pour water | to talk nonsense |
| "brać coś do siebie" | to take something to oneself | to take offense |
| "wyjść na prostą" | to go out to straight | to recover |
| "mieć ręce pełne roboty" | to have hands full of work | to be very busy |

---

## Light Verb Constructions

Light verbs are verbs with minimal semantic content that combine with a noun to form a predicate. The noun carries the main meaning.

### Examples

| Language | Light Verb | Noun | Full Expression | Meaning |
|----------|-----------|------|-----------------|---------|
| English | make | decision | make a decision | decide |
| English | take | walk | take a walk | walk |
| English | give | look | give a look | look |
| Polish | podjąć | decyzję | podjąć decyzję | decide |
| Polish | wziąć | kąpiel | wziąć kąpiel | bathe |
| Polish | rzucić | okiem | rzucić okiem | glance |

### Representing Light Verb Constructions

```rust
pub struct LightVerbConstruction {
    pub light_verb: String,
    pub noun: String,
    pub concept: ConceptId,
    pub verb_features: FeatureBundle,
    pub noun_features: FeatureBundle,
}

// Example: Polish "podjąć decyzję"
LightVerbConstruction {
    light_verb: "podjąć",
    noun: "decyzję",
    concept: ConceptId::DECIDE,
    verb_features: FeatureBundle {
        aspect: Some(Aspect::Perfective),
        ..Default::default()
    },
    noun_features: FeatureBundle {
        gender: Some(Gender::Feminine),
        number: Some(Number::Singular),
        case: Some(Case::Accusative),
        ..Default::default()
    },
}
```

---

## Phrasal Verbs (English)

Phrasal verbs are multi-word verbs consisting of a verb + particle that create a new meaning.

### Examples

| Phrasal Verb | Meaning | Example |
|-------------|---------|---------|
| give up | surrender | He gave up smoking |
| look for | search | I'm looking for my keys |
| take off | remove / depart | Take off your coat |
| turn on | activate | Turn on the light |

### Representing Phrasal Verbs

```rust
pub struct PhrasalVerb {
    pub verb: String,
    pub particle: String,
    pub concept: ConceptId,
    pub separable: bool,
    pub meanings: Vec<PhrasalVerbMeaning>,
}

pub struct PhrasalVerbMeaning {
    pub meaning: String,
    pub register: Register,
    pub examples: Vec<String>,
}

// Example: "give up"
PhrasalVerb {
    verb: "give",
    particle: "up",
    concept: ConceptId::SURRENDER,
    separable: false,
    meanings: vec![
        PhrasalVerbMeaning {
            meaning: "surrender, quit",
            register: Register::Neutral,
            examples: vec!["He gave up smoking".to_string()],
        }
    ],
}
```

---

## Multi-Word Expressions (MWE)

Multi-word expressions are fixed or semi-fixed phrases that function as a single unit.

### Types of MWE

```rust
pub enum MultiWordExpression {
    /// Fully fixed: "by the way", "in spite of"
    FixedExpression {
        expression: String,
        concept: ConceptId,
    },
    
    /// Semi-fixed: "as soon as possible"
    SemiFixedExpression {
        pattern: String,
        slots: Vec<String>,
    },
    
    /// Collocation: "strong coffee", "heavy rain"
    Collocation {
        words: Vec<String>,
        frequency: Frequency,
    },
    
    /// Idiom: "kick the bucket"
    Idiom {
        expression: String,
        figurative_meaning: String,
    },
    
    /// Light verb construction: "make a decision"
    LightVerbConstruction(LightVerbConstruction),
    
    /// Phrasal verb: "give up"
    PhrasalVerb(PhrasalVerb),
}
```

---

## Aktionsart (Verb Aspectual Classes)

Aktionsart classifies verbs by their inherent aspectual properties (how the action unfolds over time).

### Vendler's Classification

```rust
pub enum Aktionsart {
    /// State: no change, no endpoint
    /// "know", "love", "believe"
    State,
    
    /// Activity: ongoing, no endpoint
    /// "run", "walk", "swim"
    Activity,
    
    /// Accomplishment: has endpoint, takes time
    /// "build a house", "write a letter"
    Accomplishment,
    
    /// Achievement: instantaneous change
    /// "recognize", "find", "win"
    Achievement,
    
    /// Semelfactive: single instantaneous action
    /// "knock", "tap", "blink"
    Semelfactive,
}
```

### Aktionsart Properties

| Class | Telic (endpoint) | Durative | Dynamic |
|-------|-----------------|----------|---------|
| State | ✗ | ✓ | ✗ |
| Activity | ✗ | ✓ | ✓ |
| Accomplishment | ✓ | ✓ | ✓ |
| Achievement | ✓ | ✗ | ✓ |
| Semelfactive | ✗ | ✗ | ✓ |

### Representing Aktionsart in Lexicon

```rust
pub struct LexEntry {
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub concept: ConceptId,
    pub features: FeatureBundle,
    pub aktionsart: Option<Aktionsart>,
    // ...
}

// Example: "build" (Accomplishment)
LexEntry {
    lemma: "build",
    pos: PartOfSpeech::Verb,
    concept: ConceptId::BUILD,
    aktionsart: Some(Aktionsart::Accomplishment),
}

// Example: "know" (State)
LexEntry {
    lemma: "know",
    pos: PartOfSpeech::Verb,
    concept: ConceptId::KNOW,
    aktionsart: Some(Aktionsart::State),
}
```

---

## Data Validation

The compiler enforces consistency:

- Every `Sense.concept` must exist in the master concept list
- Every `SubcatFrame.roles` must be a subset of the concept's `required_roles + optional_roles`
- Every `NounParadigm` reference must exist in `data/morphology/<lang>/noun_paradigms.ron`
- Every `VerbParadigm` reference must exist in `data/morphology/<lang>/verb_paradigms.ron`

Build-time validation catches mismatches before runtime.
