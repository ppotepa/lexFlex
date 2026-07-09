# Grammar Cases — Role-to-Case Mapping

This document describes how **semantic roles** in the Interlingua map to **grammatical cases** (Polish) or **syntactic positions / prepositions** (English).

---

## Polish Case System

Polish has 7 grammatical cases. Each case has a core semantic function, and the mapping from Interlingua roles to cases is governed by **verb subcategorization frames** stored in the lexicon.

### The 7 Cases

| Case | Polish Name | Core Function | Example |
|------|-------------|---------------|---------|
| **Nominative** (NOM) | Mianownik | Subject, agent | **Tomek** dał... |
| **Genitive** (GEN) | Dopełniacz | Possession, partitive, negated ACC | nie dał **jabłka**, książka **Tomka** |
| **Dative** (DAT) | Celownik | Recipient, beneficiary | dał **Izie** |
| **Accusative** (ACC) | Biernik | Direct object, theme, goal of motion | dał **jabłko** |
| **Instrumental** (INST) | Narzędnik | Instrument, predicate nominal | pisze **nożem**, jest **nauczycielem** |
| **Locative** (LOC) | Miejscownik | Location (with preposition) | w **domu**, o **książce** |
| **Vocative** (VOC) | Wołacz | Direct address | **Tomku!** |

### Case Usage by Function

#### Nominative (Mianownik)

```
• Subject/agent of transitive verb:    "Tomek dał jabłko"
• Subject of intransitive verb:        "Tomek idzie"
• Predicate with "to be":              "Tomek jest studentem" (note: INST for predicate)
• Subject of passive:                  "Książka została dana" (passive — out of MVP scope)
```

#### Genitive (Dopełniacz)

```
• Possession:           "książka Tomka"          (Tomek's book)
• Negated direct object: "nie dał jabłka"        (ACC → GEN under negation!)
• Some verbs require:   "szukać kluczy"          (szukać + GEN, not ACC)
• Partitive:            "kawałek chleba"         (a piece of bread)
• After some prepositions: "z domu" (from the house), "do sklepu" (to the shop)
• After numerals ≥5:    "pięć jabłek"           (five apples)
```

#### Dative (Celownik)

```
• Recipient:            "dał Izie jabłko"        (gave Iza an apple)
• Beneficiary:          "kupił jej prezent"      (bought her a gift)
• Some verbs require:   "pomagać bratu"          (pomagać + DAT)
• Experiencer (dative subject): "jest mi zimno"  (I feel cold)
```

#### Accusative (Biernik)

```
• Direct object:        "dał jabłko"             (gave an apple)
• Theme of transfer:    "wziął książkę"          (took a book)
• Goal of motion:       "idę do szkoły"          (I go to school — "do" + ACC)
• After most prepositions of direction: "na stół" (onto the table)
• Duration:             "czekałem godzinę"       (I waited an hour)
```

#### Instrumental (Narzędnik)

```
• Instrument:           "pisze długopisem"       (writes with a pen)
• Means of transport:   "jedzie autobusem"       (goes by bus)
• Predicate nominal:    "jest nauczycielem"      (is a teacher)
• After "z" (= with):  "idę z Tomkiem"          (I go with Tomek)
• Manner:              "mówi szeptem"            (speaks in a whisper)
```

#### Locative (Miejscownik)

```
• Location (always with preposition):
    "w domu"          (in the house)
    "na stole"        (on the table)
    "o książce"       (about the book)
    "przy oknie"      (by the window)
```

#### Vocative (Wołacz)

```
• Direct address:       "Tomku, chodź!"          (Tomek, come!)
• Formal address:       "Panie Profesorze!"      (Professor!)
```

---

## Role → Case Mapping

### Default Mapping

The default mapping from semantic roles to Polish cases:

```
┌──────────────┬──────────────────────┬────────────────────────────────┐
│ Semantic Role│ Default Case         │ Override Condition             │
├──────────────┼──────────────────────┼────────────────────────────────┤
│ Agent        │ NOMINATIVE           │ (always NOM)                   │
│ Patient      │ ACCUSATIVE           │ NEG polarity → GENITIVE        │
│ Theme        │ ACCUSATIVE           │ NEG polarity → GENITIVE        │
│ Recipient    │ DATIVE               │ (always DAT)                   │
│ Beneficiary  │ DATIVE               │ (always DAT)                   │
│ Experiencer  │ NOMINATIVE           │ some verbs → DATIVE            │
│ Stimulus     │ ACCUSATIVE           │ some verbs → GENITIVE          │
│ Source       │ GENITIVE (with prep) │ "z" + GEN, "od" + GEN          │
│ Goal         │ ACCUSATIVE (w/ prep) │ "do" + GEN, "na" + ACC         │
│ Location     │ LOCATIVE (with prep) │ "w" + LOC, "na" + LOC          │
│ Instrument   │ INSTRUMENTAL         │ "z" + INST                     │
└──────────────┴──────────────────────┴────────────────────────────────┘
```

### Negation Effect on Case

**This is a critical rule for Polish.** When a transitive verb is negated, the direct object changes from Accusative to Genitive:

```
Positive:  "Tomek dał jabłko Izie"
           jabłko = ACC (neuter ACC = NOM form)

Negative:  "Tomek nie dał jabłka Izie"
           jabłko → jabłka = GEN (negated ACC → GEN)
```

This applies to ALL verbs with accusative direct objects:

```
"kupił książkę"     → "nie kupił książki"      (ACC → GEN)
"widzę kota"        → "nie widzę kota"          (ACC → GEN, animate masc: ACC=GEN)
"czytam gazetę"     → "nie czytam gazety"       (ACC → GEN)
```

**Implementation**: the deduction engine marks polarity; the generation engine applies the case shift.

```rust
fn resolve_case(role: SemanticRole, polarity: Polarity, subcat: &SubcatFrame) -> Case {
    let base_case = subcat.case_for_role(role);
    
    match (base_case, polarity) {
        (Case::Accusative, Polarity::Negative) => Case::Genitive,  // THE RULE
        _ => base_case,
    }
}
```

### Verb Subcategorization Overrides

Some verbs require **non-default cases** for their arguments. These are stored in the lexicon:

```
Verb         Role         Case         Example
────────     ──────       ──────       ────────────────────
dać          recipient    DAT          dał Izie (DAT)
dać          theme        ACC          dał jabłko (ACC)

szukać       theme        GEN          szukam kluczy (GEN, not ACC!)
bać się      stimulus     GEN          boję się psa (GEN)
słuchać      stimulus     GEN          słucham muzyki (GEN)
używać       theme        GEN          używam komputera (GEN)

pomagać      beneficiary  DAT          pomagam bratu (DAT)
dziękować    beneficiary  DAT          dziękuję ci (DAT)
wierzyć      stimulus     DAT          wierzę tobie (DAT)

interesować  stimulus     INST         interesuję się muzyką (INST)
zostać       predicate    INST         został nauczycielem (INST)
```

These are encoded in each verb's `SubcatFrame` in the lexicon — see [LEXICON.md](./LEXICON.md).

---

## Prepositions and Case

Many spatial and temporal relations use prepositions that **govern specific cases**:

```
Preposition    Case            Meaning                 Example
───────────    ──────          ───────                 ─────────────
w              LOC             in (location)           w domu
w              ACC             into (direction)        wchodzę w las
na             LOC             on (location)           na stole
na             ACC             onto (direction)        kładę na stół
z              GEN             from, out of            z domu
z              INST            with (accompaniment)    z Tomkiem
do             GEN             to, toward              do szkoły
od             GEN             from (person/source)    od Tomka
o              LOC             about                   o książce
przez          ACC             through, by             przez las
nad            INST            above                   nad rzeką
pod            INST            under                   pod stołem
przed          INST            before (spatial)        przed domem
za             INST            behind                  za domem
```

Prepositions are stored in the lexicon with their governed case:

```rust
PrepositionEntry(
    lemma: "w",
    governed_cases: {
        LOC: "in (static location)",
        ACC: "into (direction of motion)",
    },
)
```

The deduction engine selects the correct case based on context (location vs direction).

---

## Adjective-Noun Agreement

Adjectives agree with their noun in **gender, number, and case**. The morphology engine handles the actual inflection (see [MORPHOLOGY.md](./MORPHOLOGY.md)), but the case system determines which form is needed.

```
"duży dom"          (big house)
  duży  = NOM.SG.M
  dom   = NOM.SG.M   → agreement: same case, gender, number

"w dużym domu"      (in a big house)
  dużym = LOC.SG.M
  domu  = LOC.SG.M   → preposition "w" + LOC governs both

"dał dużą książkę"  (gave a big book)
  dużą   = ACC.SG.F
  książkę = ACC.SG.F → agreement in ACC
```

---

## English: Syntactic Position Instead of Case

English has largely lost its case system (remnants: I/me, he/him, who/whom). Instead, it uses **syntactic position** and **prepositions**.

### Role → Position Mapping

```
┌──────────────┬──────────────────────────────────┬───────────────────────┐
│ Semantic Role│ English Realization              │ Example               │
├──────────────┼──────────────────────────────────┼───────────────────────┤
│ Agent        │ Subject (before verb)            │ **Tomek** gave...     │
│ Theme        │ Direct object (after verb)       │ ...gave **an apple**  │
│ Recipient    │ Indirect obj OR "to" + NP        │ ...gave **Iza** /     │
│              │ (ditransitive alternation)       │ ...gave an apple      │
│              │                                  │ **to Iza**            │
│ Beneficiary  │ "for" + NP                       │ ...bought a gift      │
│              │                                  │ **for her**           │
│ Source       │ "from" + NP                      │ ...came **from Warsaw**│
│ Goal         │ "to" + NP                        │ ...went **to Krakow** │
│ Location     │ "in/at/on" + NP                  │ ...is **at home**     │
│ Instrument   │ "with" + NP                      │ ...cut **with a knife**│
│ Stimulus     │ Direct object OR "of" + NP       │ fears **dogs** /      │
│              │ (verb-dependent)                 │ afraid **of dogs**    │
└──────────────┴──────────────────────────────────┴───────────────────────┘
```

### Ditransitive Alternation

English verbs of transfer allow two constructions:

```
"Tomek gave Iza an apple"       → [V + IO + DO]    (indirect object first)
"Tomek gave an apple to Iza"    → [V + DO + to IO]  (prepositional dative)
```

Both map to the same Interlingua Frame::TRANSFER. The generation engine chooses based on discourse factors (information structure, weight, focus).

### English Negation

English negation does NOT change the case of objects (no ACC→GEN rule):

```
"Tomek gave Iza an apple"     → "Tomek didn't give Iza an apple"
```

No morphological change to "apple" — negation is expressed purely through auxiliary "do not".

---

## Case Resolution Algorithm

During generation, the engine determines the case for each entity:

```rust
fn resolve_case_for_entity(
    entity: &Entity,
    role: SemanticRole,
    frame: &Frame,
    polarity: Polarity,
    descriptor: &LanguageDescriptor,
) -> Case {
    // 1. Check verb subcategorization frame
    if let Some(subcat_case) = frame.subcat.case_for_role(role) {
        let case = subcat_case;
        
        // 2. Apply negation rule (PL only)
        if descriptor.language == "pl" 
            && case == Case::Accusative 
            && polarity == Polarity::Negative 
        {
            return Case::Genitive;
        }
        
        return case;
    }
    
    // 3. Fall back to default role→case mapping
    default_case_for_role(role)
}

fn default_case_for_role(role: SemanticRole) -> Case {
    match role {
        SemanticRole::Agent       => Case::Nominative,
        SemanticRole::Patient     => Case::Accusative,
        SemanticRole::Theme       => Case::Accusative,
        SemanticRole::Recipient   => Case::Dative,
        SemanticRole::Beneficiary => Case::Dative,
        SemanticRole::Experiencer => Case::Nominative,
        SemanticRole::Stimulus    => Case::Accusative,
        SemanticRole::Source      => Case::Genitive,
        SemanticRole::Goal        => Case::Accusative,
        SemanticRole::Location    => Case::Locative,
        SemanticRole::Instrument  => Case::Instrumental,
    }
}
```

---

## Case Ambiguity Resolution (Parsing)

During parsing, some noun forms are ambiguous between cases:

```
"jabłko"  → could be NOM.SG.N or ACC.SG.N (identical for neuter)
"Tomka"   → could be GEN.SG.M or ACC.SG.M (animate masculine: GEN=ACC)
"książki" → could be GEN.SG.F, NOM.PL.F, or ACC.PL.F
```

The parser resolves these by:

1. **Verb subcategorization** — "dać" requires theme:ACC → "jabłko" is ACC
2. **Word position** — subject usually precedes verb → NOM
3. **Preposition** — "w" + LOC or ACC depending on static/dynamic
4. **Context** — discourse topic tends to be NOM/subject

```rust
fn disambiguate_case(
    word: &str,
    possible_cases: Vec<Case>,
    verb_subcat: &SubcatFrame,
    position: SentencePosition,
    preposition: Option<&PrepositionEntry>,
) -> Case {
    // 1. Preposition governs case
    if let Some(prep) = preposition {
        return prep.governed_case;
    }

    // 2. Verb subcategorization narrows it
    let role = position.expected_role();
    if let Some(expected_case) = verb_subcat.case_for_role(role) {
        if possible_cases.contains(&expected_case) {
            return expected_case;
        }
    }

    // 3. Subject position → prefer NOM
    if position == SentencePosition::PreVerbal {
        if possible_cases.contains(&Case::Nominative) {
            return Case::Nominative;
        }
    }

    // 4. Default: first possible case (or flag as ambiguous)
    possible_cases[0]
}
```

---

## Voice

Voice indicates the relationship between the action and the participants (agent, patient).

### Voice Types

See [INTERLINGUA.md](./INTERLINGUA.md#voice) for canonical Voice definition.

The Voice enum includes:
- **Active**: subject is agent
- **Passive**: subject is patient/theme
- **Middle**: subject is affected, agent unspecified
- **Antipassive**: agent-focused, patient omitted/demoted
- **Causative**: added causer argument
- **Applicative**: added beneficiary/recipient
- **Reflexive**: agent = patient
- **Reciprocal**: multiple agents acting on each other

### Voice in Polish

Polish has active and passive voice. Passive is formed with auxiliary "być/zostać" + passive participle.

```
Active:   "Tomek dał jabłko Izie"
          (Tomek gave the apple to Iza)

Passive:  "Jabłko zostało dane Izie przez Tomka"
          (The apple was given to Iza by Tomek)

Passive (without agent):
          "Jabłko zostało dane Izie"
          (The apple was given to Iza)
```

### Voice in English

English has active, passive, and middle voice.

```
Active:   "Tomek gave Iza an apple"

Passive:  "An apple was given to Iza by Tomek"

Passive (without agent):
          "An apple was given to Iza"

Middle:   "The book sells well"
          (no agent, subject is affected)
```

### Voice Transformation

See [INTERLINGUA.md](./INTERLINGUA.md#voice) for canonical Voice and VoiceTransformation definitions.

Voice transformations include:
- **Passivization** (Active → Passive): Subject becomes by-phrase, object becomes subject
- **Antipassivization** (Passive → Active): Reverse of passivization
- **Causativization**: Add causer argument
- **Applicativization**: Add beneficiary/recipient argument

### Voice and Case Mapping

Voice changes the case/position mapping of semantic roles:

| Voice | Agent | Patient/Theme |
|-------|-------|---------------|
| Active | Subject (NOM) | Object (ACC) |
| Passive | by-phrase (INST in PL) | Subject (NOM) |
| Middle | (unspecified) | Subject (NOM) |

**Polish passive case changes:**
```
Active:   Agent(NOM) + Verb + Theme(ACC)
          "Tomek(NOM) dał jabłko(ACC)"

Passive:  Theme(NOM) + być/zostać + Participle + Agent(INST)
          "Jabłko(NOM) zostało dane przez Tomka(INST)"
```

**English passive:**
```
Active:   Agent(Subject) + Verb + Theme(Object)
          "Tomek gave an apple"

Passive:  Theme(Subject) + be + past participle + by-phrase
          "An apple was given by Tomek"
```

### Voice Detection

```rust
pub fn detect_voice(
    sentence: &Sentence,
    language: &LanguageDescriptor,
) -> Voice {
    // Check for passive markers
    match language.language.as_str() {
        "pl" => {
            // Polish: "być/zostać" + passive participle
            if sentence.has_auxiliary("być") || sentence.has_auxiliary("zostać") {
                if sentence.has_passive_participle() {
                    return Voice::Passive;
                }
            }
            
            // Polish: "się" can indicate middle/reflexive
            if sentence.has_reflexive_particle("się") {
                if sentence.subject_is_patient() {
                    return Voice::Middle;
                } else {
                    return Voice::Reflexive;
                }
            }
            
            Voice::Active
        }
        
        "en" => {
            // English: "be" + past participle
            if sentence.has_auxiliary("be") && sentence.has_past_participle() {
                return Voice::Passive;
            }
            
            Voice::Active
        }
        
        _ => Voice::Active,
    }
}
```

### Voice in Generation

When generating, the engine must apply voice transformations:

```rust
pub fn apply_voice(
    frame: &Frame,
    voice: Voice,
    language: &LanguageDescriptor,
) -> GeneratedSentence {
    match voice {
        Voice::Active => {
            // Default: agent as subject, theme as object
            generate_active(frame, language)
        }
        
        Voice::Passive => {
            // Theme as subject, agent in by-phrase (optional)
            generate_passive(frame, language)
        }
        
        Voice::Middle => {
            // Theme as subject, no agent
            generate_middle(frame, language)
        }
        
        _ => generate_active(frame, language),
    }
}
```

---

## Universal Case System

The `Case` enum in Interlingua is a **universal superset** containing cases from all languages. Each language uses only the subset relevant to it.

### Finno-Ugric Cases (Finnish, Hungarian, Estonian)

Finnish has 15 grammatical cases, encoding rich spatial and relational information morphologically:

| Case | Finnish | Function | Example |
|------|---------|----------|---------|
| **Partitive** | osittainen | Partial object, ongoing action | luen **kirjaa** (I'm reading a book) |
| **Inessive** | -ssa | Location: inside | **talossa** (in the house) |
| **Elative** | -sta | Motion: from inside | **talosta** (from the house) |
| **Illative** | -Vn | Motion: into | **taloon** (into the house) |
| **Adessive** | -lla | Location: on surface | **pöydällä** (on the table) |
| **Ablative** | -lta | Motion: from surface | **pöydältä** (from the table) |
| **Allative** | -lle | Motion: onto/toward | **pöydälle** (onto the table) |
| **Essive** | -na | State/role: as | **opettajana** (as a teacher) |
| **Translative** | -ksi | Change: becoming | **opettajaksi** (into a teacher) |
| **Comitative** | -ne | Accompaniment: with | **ystävineen** (with friends) |
| **Abessive** | -tta | Absence: without | **rahatta** (without money) |
| **Instructive** | -n | Manner: by means of | **käsin** (by hand) |

**Mapping to Interlingua:**
```
Finnish: "Luin kirjaa"
Interlingua: Frame::Cognition {
    agent: Entity { case: Some(Nominative) },
    theme: Entity { case: Some(Partitive) }
}

Finnish: "Kissa on pöydällä"
Interlingua: Frame::Existence {
    entity: Entity { case: Some(Nominative) },
    location: Entity { case: Some(Adessive) }
}
```

### Ergative-Absolutive Languages (Basque, Georgian, Hindi)

Ergative languages mark the subject of transitive verbs differently from subjects of intransitive verbs:

| Case | Function | Example |
|------|----------|---------|
| **Ergative** | Subject of transitive verb | Basque: **gizonak** (the man-ERG) |
| **Absolutive** | Subject of intransitive, object of transitive | Basque: **gizona** (the man-ABS) |
| **Dative** | Indirect object | Basque: **gizonari** (to the man) |

**Mapping to Interlingua:**
```
Basque: "Gizonak ogia jan du" (The man ate bread)
Interlingua: Frame::Consumption {
    agent: Entity { case: Some(Ergative) },      // gizonak
    theme: Entity { case: Some(Absolutive) }     // ogia (bread)
}

Basque: "Gizona etorri da" (The man came)
Interlingua: Frame::Motion {
    agent: Entity { case: Some(Absolutive) }     // gizona (same form!)
}
```

### Cross-Language Case Mapping

The same semantic role maps to different cases across languages:

| Role | Polish | Finnish | Basque | English |
|------|--------|---------|--------|---------|
| Agent (transitive) | NOM | NOM | ERG | Subject position |
| Theme (transitive) | ACC | PART/ACC | ABS | Direct object |
| Agent (intransitive) | NOM | NOM | ABS | Subject position |
| Location (in) | LOC (w + LOC) | INESSIVE | INESSIVE | "in" + NP |
| Location (on) | LOC (na + LOC) | ADESSIVE | ADESSIVE | "on" + NP |
| Source (from) | GEN (z + GEN) | ELATIVE | ABLATIVE | "from" + NP |
| Goal (into) | ACC (w + ACC) | ILLATIVE | ALLATIVE | "into" + NP |
| Instrument | INST | ADESSIVE/INSTR | INSTRUMENTAL | "with" + NP |

### Implementation: Universal Case Resolution

```rust
pub fn resolve_case_universal(
    role: SemanticRole,
    language: &LanguageDescriptor,
    context: &ResolutionContext,
) -> Option<Case> {
    match language.case_system {
        CaseSystem::Accusative => {
            // PL, DE, RU, LA: NOM/ACC alignment
            resolve_case_accusative(role, context)
        }
        
        CaseSystem::Ergative => {
            // BASQUE, GEORGIAN: ERG/ABS alignment
            resolve_case_ergative(role, context)
        }
        
        CaseSystem::Rich => {
            // FI, HU: many spatial cases
            resolve_case_rich(role, context)
        }
        
        CaseSystem::None => {
            // EN, ZH: no morphological cases, use word order/prepositions
            None
        }
    }
}

fn resolve_case_accusative(role: SemanticRole, context: &ResolutionContext) -> Option<Case> {
    match role {
        SemanticRole::Agent => Some(Case::Nominative),
        SemanticRole::Theme => {
            if context.is_negated {
                Some(Case::Genitive)  // PL: negation changes ACC→GEN
            } else {
                Some(Case::Accusative)
            }
        }
        SemanticRole::Recipient => Some(Case::Dative),
        SemanticRole::Instrument => Some(Case::Instrumental),
        SemanticRole::Location => Some(Case::Locative),
        _ => None
    }
}

fn resolve_case_rich(role: SemanticRole, context: &ResolutionContext) -> Option<Case> {
    match role {
        SemanticRole::Location => {
            match context.spatial_relation {
                SpatialRelation::Inside => Some(Case::Inessive),
                SpatialRelation::OnSurface => Some(Case::Adessive),
                SpatialRelation::FromInside => Some(Case::Elative),
                SpatialRelation::IntoInside => Some(Case::Illative),
                SpatialRelation::FromSurface => Some(Case::Ablative),
                SpatialRelation::OntoSurface => Some(Case::Allative),
            }
        }
        SemanticRole::Theme => Some(Case::Partitive),  // FI: partial object
        _ => resolve_case_accusative(role, context)
    }
}
```

