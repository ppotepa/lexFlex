# Pronouns — Forms, Selection, and Placement

Pronouns are a complex area, especially in Polish. This document covers the pronoun system, enclitic forms, reflexives, and the algorithms for selecting the correct form and position.

## Pronoun Categories

```rust
pub enum PronounCategory {
    /// Personal pronouns: I, you, he, she, it, we, they
    Personal,
    
    /// Reflexive pronouns: myself, yourself, himself, siebie, sobie, się
    Reflexive,
    
    /// Possessive pronouns: my, your, his, her, its, our, their
    Possessive,
    
    /// Demonstrative pronouns: this, that, these, those
    Demonstrative,
    
    /// Interrogative pronouns: who, what, which
    Interrogative,
    
    /// Relative pronouns: who, which, that (in relative clauses)
    Relative,
}
```

## Polish Pronoun System

### Personal Pronouns

Polish personal pronouns have two forms: **full (stressed)** and **enclitic (unstressed)**.

```rust
pub struct PolishPronoun {
    pub person: Person,
    pub number: Number,
    pub gender: Option<Gender>,  // only for 3rd person
    pub case: Case,
    pub form_type: PronounFormType,
}

pub enum PronounFormType {
    /// Full form (stressed, can appear in any position)
    /// "mnie", "tobie", "jego", "jej", "nas", "was", "ich"
    Full,
    
    /// Enclitic form (unstressed, must follow verb)
    /// "mi", "ci", "go", "ją", "nam", "wam", "im"
    Enclitic,
    
    /// Special enclitic for accusative/genitive
    /// "niego", "niej", "nich" (after prepositions)
    PrepositionalEnclitic,
}
```

### Complete Pronoun Table

#### First Person Singular (ja)

| Case | Full Form | Enclitic | Example |
|------|-----------|----------|---------|
| Nominative | ja | — | **Ja** to zrobiłem |
| Genitive | mnie | — | nie ma **mnie** |
| Dative | mnie | **mi** | dał **mi** / **mnie** prezent |
| Accusative | mnie | — | widzi **mnie** |
| Instrumental | mną | — | idzie ze **mną** |
| Locative | mnie | — | myśli o **mnie** |

#### Second Person Singular (ty)

| Case | Full Form | Enclitic | Example |
|------|-----------|----------|---------|
| Nominative | ty | — | **Ty** to zrobiłeś |
| Genitive | ciebie | **cię** | nie ma **cię** |
| Dative | tobie | **ci** | dał **ci** / **tobie** prezent |
| Accusative | ciebie | **cię** | widzi **cię** |
| Instrumental | tobą | — | idzie z **tobą** |
| Locative | tobie | — | myśli o **tobie** |

#### Third Person Singular Masculine (on)

| Case | Full Form | Enclitic | Prepositional | Example |
|------|-----------|----------|---------------|---------|
| Nominative | on | — | — | **On** przyszedł |
| Genitive | jego | **go** | niego | nie ma **go** / bez **niego** |
| Dative | jemu | **mu** | niemu | dał **mu** / **jemu** |
| Accusative | jego | **go** | niego | widzi **go** / dla **niego** |
| Instrumental | nim | — | — | idzie z **nim** |
| Locative | nim | — | — | myśli o **nim** |

#### Third Person Singular Feminine (ona)

| Case | Full Form | Enclitic | Prepositional | Example |
|------|-----------|----------|---------------|---------|
| Nominative | ona | — | — | **Ona** przyszła |
| Genitive | jej | — | niej | nie ma **jej** / bez **niej** |
| Dative | jej | — | niej | dał **jej** prezent |
| Accusative | ją | — | nią | widzi **ją** / dla **niej** |
| Instrumental | nią | — | — | idzie z **nią** |
| Locative | niej | — | — | myśli o **niej** |

#### Third Person Singular Neuter (ono)

| Case | Full Form | Enclitic | Example |
|------|-----------|----------|---------|
| Nominative | ono | — | **Ono** jest małe |
| Genitive | jego | **go** | nie ma **go** |
| Dative | jemu | **mu** | dał **mu** zabawkę |
| Accusative | je | — | widzi **je** |
| Instrumental | nim | — | bawi się **nim** |
| Locative | nim | — | myśli o **nim** |

#### First Person Plural (my)

| Case | Full Form | Enclitic | Example |
|------|-----------|----------|---------|
| Nominative | my | — | **My** poszliśmy |
| Genitive | nas | — | nie ma **nas** |
| Dative | nam | — | dał **nam** prezent |
| Accusative | nas | — | widzi **nas** |
| Instrumental | nami | — | idzie z **nami** |
| Locative | nas | — | myśli o **nas** |

#### Second Person Plural (wy)

| Case | Full Form | Enclitic | Example |
|------|-----------|----------|---------|
| Nominative | wy | — | **Wy** poszliście |
| Genitive | was | — | nie ma **was** |
| Dative | wam | — | dał **wam** prezent |
| Accusative | was | — | widzi **was** |
| Instrumental | wami | — | idzie z **wami** |
| Locative | was | — | myśli o **was** |

#### Third Person Plural (oni/one)

| Case | Full Form | Enclitic | Prepositional | Example |
|------|-----------|----------|---------------|---------|
| Nominative | oni/one | — | — | **Oni** przyszli |
| Genitive | ich | — | nich | nie ma **ich** / bez **nich** |
| Dative | im | — | nim | dał **im** prezenty |
| Accusative | ich/je | — | nich | widzi **ich** / dla **nich** |
| Instrumental | nimi | — | — | idzie z **nimi** |
| Locative | nich | — | — | myśli o **nich** |

## Enclitic Form Selection Algorithm

Enclitic forms are used when the pronoun is **unstressed** and follows the verb directly.

```rust
pub fn select_pronoun_form(
    pronoun: &Pronoun,
    context: &PronounContext,
) -> PronounFormType {
    // Rule 1: After preposition, use prepositional enclitic
    if context.after_preposition {
        if pronoun.has_prepositional_form() {
            return PronounFormType::PrepositionalEnclitic;
        }
        return PronounFormType::Full;
    }
    
    // Rule 2: If pronoun is stressed/emphasized, use full form
    if context.is_stressed {
        return PronounFormType::Full;
    }
    
    // Rule 3: If pronoun is sentence-initial, use full form
    if context.is_sentence_initial {
        return PronounFormType::Full;
    }
    
    // Rule 4: If pronoun is contrasted, use full form
    // "Mnie dał, nie tobie" (He gave to ME, not to you)
    if context.is_contrasted {
        return PronounFormType::Full;
    }
    
    // Rule 5: If pronoun immediately follows verb, use enclitic
    if context.follows_verb && pronoun.has_enclitic_form() {
        return PronounFormType::Enclitic;
    }
    
    // Rule 6: Default to enclitic if available, otherwise full
    if pronoun.has_enclitic_form() && !context.is_stressed {
        return PronounFormType::Enclitic;
    }
    
    PronounFormType::Full
}
```

### Context Structure

```rust
pub struct PronounContext {
    /// Does this pronoun follow a preposition?
    pub after_preposition: bool,
    
    /// Is this pronoun stressed/emphasized?
    pub is_stressed: bool,
    
    /// Is this pronoun at the beginning of the sentence?
    pub is_sentence_initial: bool,
    
    /// Is this pronoun being contrasted with another?
    pub is_contrasted: bool,
    
    /// Does this pronoun immediately follow the verb?
    pub follows_verb: bool,
    
    /// Is this pronoun in a coordinated structure?
    pub in_coordination: bool,
}
```

### Examples

```
Example 1: Enclitic after verb
  "Dał mi prezent." (He gave me a gift.)
  
  Context:
    - after_preposition: false
    - is_stressed: false
    - is_sentence_initial: false
    - is_contrasted: false
    - follows_verb: true
  
  Decision: Enclitic → "mi"

---

Example 2: Full form when stressed
  "Mnie dał prezent!" (He gave ME a gift!)
  
  Context:
    - after_preposition: false
    - is_stressed: true
    - is_sentence_initial: true
    - is_contrasted: false
    - follows_verb: false
  
  Decision: Full → "mnie"

---

Example 3: Full form when contrasted
  "Mnie dał, nie tobie." (He gave to ME, not to you.)
  
  Context:
    - after_preposition: false
    - is_stressed: false
    - is_sentence_initial: true
    - is_contrasted: true
    - follows_verb: false
  
  Decision: Full → "mnie"

---

Example 4: Prepositional enclitic
  "Dla niego to zrobiłem." (I did it for him.)
  
  Context:
    - after_preposition: true (after "dla")
    - is_stressed: false
    - is_sentence_initial: false
    - is_contrasted: false
    - follows_verb: false
  
  Decision: PrepositionalEnclitic → "niego"

---

Example 5: Full form in coordination
  "Dał mnie i tobie prezenty." (He gave me and you gifts.)
  
  Context:
    - after_preposition: false
    - is_stressed: false
    - is_sentence_initial: false
    - is_contrasted: false
    - follows_verb: false
    - in_coordination: true
  
  Decision: Full → "mnie" (enclitics don't appear in coordination)
```

## Pronoun Placement in Polish

**⚠️ FEATURE - v0.2+** (placement uses discourse context for information structure)

Polish has relatively free word order, but pronouns follow specific placement rules:

```rust
pub fn place_pronoun(
    pronoun: &Pronoun,
    sentence: &mut Sentence,
    discourse: &Discourse,
) {
    match pronoun.form_type {
        PronounFormType::Enclitic => {
            // Enclitic pronouns must follow the verb immediately
            // They cannot be sentence-initial
            let verb_position = sentence.find_verb_position();
            sentence.insert_after(verb_position, pronoun);
        }
        
        PronounFormType::Full => {
            // Full forms can appear in various positions
            // Use information structure to determine placement
            
            if pronoun.is_topic(discourse) {
                // Topic pronouns tend to come early
                sentence.place_in_topic_position(pronoun);
            } else if pronoun.is_focus(discourse) {
                // Focus pronouns tend to come late
                sentence.place_in_focus_position(pronoun);
            } else {
                // Neutral position: follow default word order
                sentence.place_in_default_position(pronoun);
            }
        }
        
        PronounFormType::PrepositionalEnclitic => {
            // Must follow the preposition immediately
            let prep_position = sentence.find_preposition_position(pronoun.preposition);
            sentence.insert_after(prep_position, pronoun);
        }
    }
}
```

### Placement Examples

```
Sentence: "Tomek dał prezent Izie."
          (Tomek gave a gift to Iza.)

With pronoun "mi" (enclitic):
  "Tomek dał mi prezent."
  (Tomek gave me a gift.)
  
  Placement: immediately after verb "dał"

With pronoun "mnie" (full, stressed):
  "Tomek mnie dał prezent!"
  (Tomek gave ME a gift!)
  
  Placement: before verb for emphasis

With pronoun "mnie" (full, sentence-initial):
  "Mnie dał prezent."
  (He gave ME a gift.)
  
  Placement: sentence-initial for emphasis
```

## Reflexive Pronouns

### Polish Reflexive System

Polish has three reflexive pronouns:

```rust
pub enum PolishReflexive {
    /// "się" — accusative reflexive (most common)
    /// "Tomek myje się" (Tomek washes himself)
    Sie,
    
    /// "sobie" — dative reflexive
    /// "Kupił sobie prezent" (He bought himself a gift)
    Sobie,
    
    /// "siebie" — full form (genitive/accusative, used for emphasis or after prepositions)
    /// "Myśli o sobie" (He thinks about himself)
    /// "Kocha siebie" (He loves himself — emphasized)
    Siebie,
}
```

### Reflexive Form Selection

```rust
pub fn select_reflexive_form(
    role: SemanticRole,
    case: Case,
    context: &ReflexiveContext,
) -> PolishReflexive {
    // Rule 1: If reflexive is after preposition, use "siebie"
    if context.after_preposition {
        return PolishReflexive::Siebie;
    }
    
    // Rule 2: If reflexive is stressed/emphasized, use "siebie"
    if context.is_stressed {
        return PolishReflexive::Siebie;
    }
    
    // Rule 3: If role is dative, use "sobie"
    if role == SemanticRole::Recipient || case == Case::Dative {
        return PolishReflexive::Sobie;
    }
    
    // Rule 4: If role is accusative, use "się"
    if role == SemanticRole::Theme || case == Case::Accusative {
        return PolishReflexive::Sie;
    }
    
    // Rule 5: For other cases, use "siebie"
    PolishReflexive::Siebie
}
```

### Reflexive Examples

```
Example 1: Accusative reflexive
  "Tomek myje się."
  (Tomek washes himself.)
  
  Role: Theme (what is being washed)
  Case: Accusative
  Form: "się"

---

Example 2: Dative reflexive
  "Kupił sobie prezent."
  (He bought himself a gift.)
  
  Role: Recipient (for whom)
  Case: Dative
  Form: "sobie"

---

Example 3: Reflexive after preposition
  "Myśli o sobie."
  (He thinks about himself.)
  
  Context: after preposition "o"
  Form: "siebie"

---

Example 4: Emphasized reflexive
  "Kocha siebie, nie innych."
  (He loves HIMSELF, not others.)
  
  Context: emphasized/contrasted
  Form: "siebie"

---

Example 5: Reflexive verb (lexical reflexive)
  "Boi się psa."
  (He is afraid of the dog.)
  
  Note: "bać się" is a lexical reflexive verb
  Form: "się" (part of the verb)
```

## English Pronoun System

English pronouns are simpler but have their own complexities:

### Personal Pronouns

| Person | Number | Gender | Subject | Object | Possessive |
|--------|--------|--------|---------|--------|------------|
| 1st | Singular | — | I | me | my/mine |
| 2nd | Singular | — | you | you | your/yours |
| 3rd | Singular | Masculine | he | him | his |
| 3rd | Singular | Feminine | she | her | her/hers |
| 3rd | Singular | Neuter | it | it | its |
| 1st | Plural | — | we | us | our/ours |
| 2nd | Plural | — | you | you | your/yours |
| 3rd | Plural | — | they | them | their/theirs |

### Case Selection in English

```rust
pub fn select_english_case(
    pronoun: &Pronoun,
    role: SemanticRole,
    position: SentencePosition,
) -> EnglishCase {
    // Rule 1: Subject position → subject case
    if position == SentencePosition::Subject {
        return EnglishCase::Subject;
    }
    
    // Rule 2: Object of verb → object case
    if position == SentencePosition::DirectObject 
        || position == SentencePosition::IndirectObject {
        return EnglishCase::Object;
    }
    
    // Rule 3: Object of preposition → object case
    if position == SentencePosition::PrepositionalObject {
        return EnglishCase::Object;
    }
    
    // Rule 4: Predicate nominative → subject case (formal)
    // "It is I" (formal) vs "It's me" (informal)
    if position == SentencePosition::PredicateNominative {
        return EnglishCase::Subject;  // or Object for informal
    }
    
    EnglishCase::Object  // default
}
```

### English Reflexives

```rust
pub enum EnglishReflexive {
    Myself,
    Yourself,
    Himself,
    Herself,
    Itself,
    Ourselves,
    Yourselves,
    Themselves,
}

pub fn select_english_reflexive(
    person: Person,
    number: Number,
    gender: Option<Gender>,
) -> EnglishReflexive {
    match (person, number, gender) {
        (Person::First, Number::Singular, _) => EnglishReflexive::Myself,
        (Person::Second, Number::Singular, _) => EnglishReflexive::Yourself,
        (Person::Third, Number::Singular, Some(Gender::Masculine)) => EnglishReflexive::Himself,
        (Person::Third, Number::Singular, Some(Gender::Feminine)) => EnglishReflexive::Herself,
        (Person::Third, Number::Singular, Some(Gender::Neuter)) => EnglishReflexive::Itself,
        (Person::First, Number::Plural, _) => EnglishReflexive::Ourselves,
        (Person::Second, Number::Plural, _) => EnglishReflexive::Yourselves,
        (Person::Third, Number::Plural, _) => EnglishReflexive::Themselves,
        _ => EnglishReflexive::Themselves,  // default
    }
}
```

## Pronoun Resolution in Interlingua

**⚠️ FEATURE - v0.2+** (requires discourse context)

When parsing, pronouns are represented as unresolved references that must be resolved against the discourse:

```rust
pub struct PronounReference {
    /// The pronoun form in the surface language
    pub surface_form: String,
    
    /// Grammatical features of the pronoun
    pub features: PronounFeatures,
    
    /// The resolved entity (filled in during deduction)
    pub resolved_to: Option<EntityId>,
    
    /// Whether this is a reflexive
    pub is_reflexive: bool,
}

pub struct PronounFeatures {
    pub person: Person,
    pub number: Number,
    pub gender: Option<Gender>,
    pub case: Case,
}
```

### Resolution Algorithm

```rust
pub fn resolve_pronoun(
    pronoun: &PronounReference,
    discourse: &Discourse,
) -> Result<EntityId, ResolutionError> {
    // Rule 1: Reflexive pronouns resolve to the subject
    if pronoun.is_reflexive {
        return discourse.current_subject()
            .ok_or(ResolutionError::NoSubject);
    }
    
    // Rule 2: Speaker/addressee pronouns
    if pronoun.features.person == Person::First {
        return Ok(discourse.speaker.id);
    }
    if pronoun.features.person == Person::Second {
        return Ok(discourse.addressee.id);
    }
    
    // Rule 3: Third person pronouns resolve to most salient matching entity
    let candidates = discourse.find_entities(|e| {
        let entity_gender = e.entity.features.gender;
        let entity_number = e.entity.features.number;
        
        entity_number == Some(pronoun.features.number)
            && (pronoun.features.gender.is_none() 
                || entity_gender == pronoun.features.gender)
            && e.id != discourse.speaker.id
            && e.id != discourse.addressee.id
    });
    
    if candidates.is_empty() {
        return Err(ResolutionError::NoCandidate);
    }
    
    // Select most salient candidate
    let mut ranked = candidates;
    ranked.sort_by(|a, b| {
        b.salience.total().partial_cmp(&a.salience.total()).unwrap()
    });
    
    Ok(ranked[0].id)
}
```

## Pronoun Generation

When generating, the engine must select the correct pronoun form based on the entity and context:

```rust
pub fn generate_pronoun(
    entity: &Entity,
    case: Case,
    context: &GenerationContext,
    language: &str,
) -> String {
    match language {
        "pl" => generate_polish_pronoun(entity, case, context),
        "en" => generate_english_pronoun(entity, case, context),
        _ => panic!("Unsupported language"),
    }
}

fn generate_polish_pronoun(
    entity: &Entity,
    case: Case,
    context: &GenerationContext,
) -> String {
    let person = entity.features.person.unwrap();
    let number = entity.features.number.unwrap();
    let gender = entity.features.gender;
    
    // Determine form type (full vs enclitic)
    let form_type = select_pronoun_form(
        &Pronoun { person, number, gender, case, form_type: Full },
        &context.pronoun_context,
    );
    
    // Look up form in pronoun table
    match (person, number, gender, case, form_type) {
        // First person singular
        (Person::First, Number::Singular, _, Case::Nominative, _) => "ja",
        (Person::First, Number::Singular, _, Case::Genitive, Full) => "mnie",
        (Person::First, Number::Singular, _, Case::Dative, Full) => "mnie",
        (Person::First, Number::Singular, _, Case::Dative, Enclitic) => "mi",
        (Person::First, Number::Singular, _, Case::Accusative, Full) => "mnie",
        (Person::First, Number::Singular, _, Case::Instrumental, _) => "mną",
        (Person::First, Number::Singular, _, Case::Locative, _) => "mnie",
        
        // Second person singular
        (Person::Second, Number::Singular, _, Case::Nominative, _) => "ty",
        (Person::Second, Number::Singular, _, Case::Genitive, Full) => "ciebie",
        (Person::Second, Number::Singular, _, Case::Genitive, Enclitic) => "cię",
        (Person::Second, Number::Singular, _, Case::Dative, Full) => "tobie",
        (Person::Second, Number::Singular, _, Case::Dative, Enclitic) => "ci",
        (Person::Second, Number::Singular, _, Case::Accusative, Full) => "ciebie",
        (Person::Second, Number::Singular, _, Case::Accusative, Enclitic) => "cię",
        (Person::Second, Number::Singular, _, Case::Instrumental, _) => "tobą",
        (Person::Second, Number::Singular, _, Case::Locative, _) => "tobie",
        
        // Third person singular masculine
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Nominative, _) => "on",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Genitive, Full) => "jego",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Genitive, Enclitic) => "go",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Dative, Full) => "jemu",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Dative, Enclitic) => "mu",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Accusative, Full) => "jego",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Accusative, Enclitic) => "go",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Instrumental, _) => "nim",
        (Person::Third, Number::Singular, Some(Gender::Masculine), Case::Locative, _) => "nim",
        
        // ... more cases for feminine, neuter, plural
        
        _ => panic!("Unsupported pronoun form"),
    }.to_string()
}
```

## Pronoun Avoidance

**⚠️ FEATURE - v0.2+** (requires discourse context)

In some contexts, it's better to repeat the noun rather than use a pronoun:

```rust
pub fn should_use_pronoun(
    entity: &Entity,
    discourse: &Discourse,
    context: &GenerationContext,
) -> bool {
    let entity_ref = discourse.get_entity(entity.id);
    
    // Rule 1: If entity was just mentioned in previous clause, use pronoun
    if entity_ref.last_mentioned == discourse.current_utterance - 1 {
        return true;
    }
    
    // Rule 2: If entity is highly salient (topic), use pronoun
    if entity_ref.salience.total() > 0.8 {
        return true;
    }
    
    // Rule 3: If there are multiple entities with same gender/number,
    //         avoid pronoun to prevent ambiguity
    let same_features = discourse.find_entities(|e| {
        e.entity.features.gender == entity.features.gender
            && e.entity.features.number == entity.features.number
            && e.id != entity.id
    });
    
    if same_features.len() > 1 {
        return false;  // ambiguous, use noun
    }
    
    // Rule 4: If entity hasn't been mentioned recently, use noun
    if entity_ref.last_mentioned < discourse.current_utterance - 3 {
        return false;
    }
    
    true
}
```

## Summary

The pronoun system handles:

1. **Form selection** — choosing between full and enclitic forms in Polish
2. **Placement** — positioning pronouns correctly in the sentence
3. **Reflexives** — selecting the correct reflexive form (się/sobie/siebie)
4. **Resolution** — linking pronouns to their antecedents in discourse
5. **Generation** — producing the correct pronoun form for a given entity and context
6. **Avoidance** — deciding when to use a pronoun vs repeating the noun

Pronouns are stored in the lexicon as special entries with their complete paradigm tables. The engine uses the algorithms described here to select and place pronouns correctly during generation.
