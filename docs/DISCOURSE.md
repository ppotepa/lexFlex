# Discourse Management

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Discourse management is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

Discourse is the context that spans multiple utterances in a conversation. It tracks who is speaking, what has been mentioned, and what can be inferred from prior context.

## Linguistic Graph — Discourse Layer (implemented)

lexFlex materializes discourse context as **edges on `LinguisticGraph`** plus fields on `Discourse`:

| Edge / field | Purpose |
|--------------|---------|
| `Corefers` | Same entity across sentences (name/concept match) |
| `NextSentence` | Frame-to-frame link between consecutive sentences |
| `InFocus` | Salient entity after a frame |
| `RecentMention` | Recency tracking per entity |
| `ContinuesTopic` | Cross-utterance topic carry-over in `DialogueGraph` |
| `Discourse.entities_in_focus` | Queryable focus stack (`NodeId`s) |
| `Discourse.recent_mentions` | `(entity_id, recency)` pairs |

**Module:** `src/core/context.rs` — `track_discourse()`, `recent_entities_of_type()`.

**Example query after parse:**
```rust
let g = sentence.graph.as_ref().unwrap();
let focus = g.in_focus_entities();
let chain = g.coreference_chain(entity_id);
```

Multi-sentence PL input `"Tomek ma kota. Tomek ma psa."` produces `Corefers` + `NextSentence` edges testable via `tests/graph_tests.rs`.

### Zero anaphora / continuing subject (implemented)

Polish pro-drop for 3rd-person singular (`"Kupił mleko."` after `"Tomek poszedł."`) is resolved in `resolve_discourse_context()` (`src/core/context.rs`), called at the end of `track_discourse()`:

- Salient subject from sentence *N* becomes `Discourse.current_topic`.
- Sentence *N+1* frames with placeholder agents (`unknown`, unnamed `PERSON`) inherit the topic entity with `Reference::Anaphoric`.
- Construction concepts `ZERO_ANAPHORA`, `TOPIC_CONTINUATION`, `CONTINUING_AGENT` attach to IL (`Sentence.construction_concepts`) and graph (`PartOfConstruction` edges).
- English generation realizes anaphoric subjects as pronouns (`he`/`she`/`they`).

Example: `"Tomek poszedł. Kupił mleko."` → IL sentence-2 agent = Tomek (anaphoric) → EN `"Tomek went. He bought a milk."`

### IL construction tree + compile pipeline

Each `Sentence` carries `constructions: Vec<ConstructionInstance>` — first-class tree nodes wrapping inner `Frame`s with `construction_concept` (`IDENTIFICATION`, `DEMONSTRATIVE_REFERENCE`, `ZERO_ANAPHORA`, etc.). Graph `PartOfConstruction` edges use the same concept IDs from `data/concepts/concepts.ron`.

Public API: `LexFlexAPI::compile(input, from, to)` runs parse → discourse resolution → target generation (alias of `translate`).

### Cross-utterance dialogue

`resolve_dialogue_context()` runs after `link_cross_utterance_context()` so implicit subjects propagate across `DialogueGraph` utterances (e.g. `["Tomek poszedł.", "Kupił mleko."]`).

## Discourse Model

```rust
pub struct Discourse {
    /// Current speaker
    pub speaker: Entity,

    /// Current addressee
    pub addressee: Entity,

    /// All entities mentioned in the discourse, in order of introduction
    pub entities: Vec<DiscourseEntity>,

    /// Current topic (most salient entity)
    pub topic: Option<EntityId>,

    /// Discourse history (recent utterances)
    pub history: Vec<UtteranceSummary>,

    /// Shared knowledge (entities both participants know about)
    pub shared_knowledge: HashSet<EntityId>,
    
    /// Honorific level for this discourse (JA, KO, JV: important; PL, EN: None)
    pub honorific_level: Option<HonorificLevel>,
    
    /// Relationship between speaker and addressee
    pub relationship: RelationshipType,
}

/// Honorific levels for languages with complex honorific systems
pub enum HonorificLevel {
    Plain,          // informal (PL: ty, EN: you, JA: タメ口)
    Polite,         // formal (PL: Pan/Pani, EN: sir/madam, JA: 丁寧語)
    Honorific,      // respectful (JA: 尊敬語)
    Humble,         // self-deprecating (JA: 謙譲語)
}

/// Relationship between speaker and addressee
pub enum RelationshipType {
    Intimate,       // family, close friends
    Familiar,       // friends, peers
    Neutral,        // strangers, acquaintances
    Formal,         // business, official
    Hierarchical,   // boss/employee, teacher/student
}

pub struct DiscourseEntity {
    pub id: EntityId,
    pub entity: Entity,
    pub introduced_in: UtteranceId,
    pub salience: SalienceScore,
    pub last_mentioned: UtteranceId,
    pub mention_count: usize,
    pub grammatical_role: GrammaticalRole,  // subject, object, etc.
    
    /// All mentions of this entity in the discourse
    pub mentions: Vec<Mention>,
    
    /// Coreference chain (links to other DiscourseEntity IDs that refer to the same real-world entity)
    pub coreference_chain: Vec<EntityId>,
    
    /// Binding domain (for anaphora resolution)
    pub binding_domain: Option<BindingDomain>,
    
    /// Anaphor type (for binding theory)
    pub anaphor_type: Option<AnaphorType>,
}

pub struct Mention {
    pub id: MentionId,
    pub entity_id: EntityId,
    pub utterance_id: UtteranceId,
    pub sentence_id: SentenceId,
    pub span: (usize, usize),  // start, end positions in text
    pub mention_type: MentionType,
    pub features: FeatureBundle,
    pub resolved: bool,  // whether this mention has been resolved to an entity
}

pub enum MentionType {
    /// Proper name: "Tomek", "Iza"
    ProperName(String),
    
    /// Definite description: "the book", "ta książka"
    DefiniteDescription(String),
    
    /// Indefinite description: "a book", "jakaś książka"
    IndefiniteDescription(String),
    
    /// Pronoun: "he", "she", "it", "on", "ona"
    Pronoun(String),
    
    /// Reflexive: "himself", "siebie"
    Reflexive(String),
    
    /// Demonstrative: "this", "that", "ten", "tamten"
    Demonstrative(String),
    
    /// Nominal: "the student", "student"
    Nominal(String),
}

pub struct SalienceScore {
    pub recency: f32,      // 0.0-1.0, based on last_mentioned
    pub frequency: f32,    // 0.0-1.0, based on mention_count
    pub grammatical: f32,  // 0.0-1.0, subjects > objects > oblique
    pub topicality: f32,   // 0.0-1.0, is this the topic?
}

pub enum GrammaticalRole {
    Subject,
    DirectObject,
    IndirectObject,
    Oblique,
    Predicate,
}
```

---

## Coreference Resolution

Coreference resolution identifies when multiple mentions refer to the same entity.

### Coreference Resolution Algorithm

```rust
pub struct CoreferenceResolver {
    discourse: Discourse,
}

impl CoreferenceResolver {
    pub fn resolve(
        &mut self,
        mention: &Mention,
        sentence: &Sentence,
    ) -> Result<EntityId, CoreferenceError> {
        match &mention.mention_type {
            MentionType::Pronoun(text) => {
                self.resolve_pronoun(text, &mention.features, sentence)
            }
            
            MentionType::Reflexive(text) => {
                self.resolve_reflexive(text, sentence)
            }
            
            MentionType::DefiniteDescription(text) => {
                self.resolve_definite_description(text, sentence)
            }
            
            MentionType::Demonstrative(text) => {
                self.resolve_demonstrative(text, sentence)
            }
            
            MentionType::ProperName(name) => {
                self.resolve_proper_name(name, sentence)
            }
            
            _ => Err(CoreferenceError::Unresolvable),
        }
    }
    
    fn resolve_pronoun(
        &self,
        text: &str,
        features: &FeatureBundle,
        sentence: &Sentence,
    ) -> Result<EntityId, CoreferenceError> {
        // Rule 1: Reflexive pronouns → subject of current clause
        if Self::is_reflexive(text) {
            return sentence.subject()
                .map(|subj| subj.entity_id)
                .ok_or(CoreferenceError::NoSubject);
        }
        
        // Rule 2: Speaker/addressee pronouns
        if text == "ja" || text == "I" {
            return Ok(self.discourse.speaker.id);
        }
        if text == "ty" || text == "you" {
            return Ok(self.discourse.addressee.id);
        }
        
        // Rule 3: Third person pronouns → most salient matching entity
        let candidates = self.find_matching_entities(features);
        
        if candidates.is_empty() {
            return Err(CoreferenceError::NoCandidates);
        }
        
        // Apply binding theory constraints
        let filtered = self.apply_binding_constraints(&candidates, sentence);
        
        if filtered.is_empty() {
            return Err(CoreferenceError::BindingViolation);
        }
        
        // Rank by salience
        let best = filtered.iter()
            .max_by(|a, b| a.salience.total().partial_cmp(&b.salience.total()).unwrap())
            .unwrap();
        
        Ok(best.id)
    }
    
    fn find_matching_entities(
        &self,
        features: &FeatureBundle,
    ) -> Vec<&DiscourseEntity> {
        self.discourse.entities.iter()
            .filter(|e| {
                // Match gender
                let gender_match = features.gender.is_none()
                    || e.entity.features.gender == features.gender;
                
                // Match number
                let number_match = features.number.is_none()
                    || e.entity.features.number == features.number;
                
                // Match person (exclude speaker/addressee for 3rd person)
                let person_match = features.person.is_none()
                    || (features.person != Some(Person::First)
                        && features.person != Some(Person::Second));
                
                gender_match && number_match && person_match
            })
            .collect()
    }
    
    fn apply_binding_constraints(
        &self,
        candidates: &[&DiscourseEntity],
        sentence: &Sentence,
    ) -> Vec<&DiscourseEntity> {
        candidates.iter()
            .filter(|candidate| {
                // Principle B: Pronouns must be free in their local domain
                // (cannot refer to subject of same clause)
                if let Some(subject) = sentence.subject() {
                    if subject.entity_id == candidate.id {
                        return false;  // violates Principle B
                    }
                }
                
                // Principle C: R-expressions must be free everywhere
                // (proper names cannot be bound by pronouns)
                if candidate.anaphor_type == Some(AnaphorType::RExpression) {
                    // Check if candidate is a proper name being bound
                    // This is complex - simplified version
                    return true;
                }
                
                true
            })
            .copied()
            .collect()
    }
}
```

---

## Binding Theory (Extended)

Binding theory constrains coreference relations based on syntactic structure.

### Binding Domains

```rust
pub struct BindingDomain {
    pub domain_id: usize,
    pub governor: EntityId,  // head of the domain
    pub domain_type: BindingDomainType,
    pub clause_boundary: ClauseBoundary,
}

pub enum BindingDomainType {
    /// Smallest clause containing the governor and a subject
    LocalDomain,
    
    /// Matrix clause (main clause)
    MatrixDomain,
    
    /// Entire discourse
    DiscourseDomain,
}

pub struct ClauseBoundary {
    pub clause_id: usize,
    pub clause_type: ClauseType,
    pub subject: Option<EntityId>,
}

pub enum ClauseType {
    MainClause,
    SubordinateClause,
    RelativeClause,
    ComplementClause,
    InfinitivalClause,
}
```

### Binding Principles

```rust
pub enum BindingPrinciple {
    /// Principle A: Anaphors must be bound in their local domain
    /// "Tom criticized himself" ✓ (himself bound by Tom in same clause)
    /// "Tom said that Mary criticized himself" ✗ (himself not bound locally)
    PrincipleA,
    
    /// Principle B: Pronouns must be free in their local domain
    /// "Tom criticized him" ✓ (him refers to someone else)
    /// "Tom criticized him" where him = Tom ✗ (violates Principle B)
    PrincipleB,
    
    /// Principle C: R-expressions must be free everywhere
    /// "He criticized Tom" ✓ (Tom is free)
    /// "He criticized Tom" where he = Tom ✗ (Tom is bound by he)
    PrincipleC,
}

pub enum AnaphorType {
    /// Anaphors: "himself", "herself", "siebie" (must be bound - Principle A)
    Anaphor,
    
    /// Pronouns: "he", "she", "on" (must be free locally - Principle B)
    Pronoun,
    
    /// R-expressions: "Tom", "Iza" (must be free everywhere - Principle C)
    RExpression,
}
```

### Binding Theory Examples

```
Principle A (Anaphors must be bound locally):

✓ "Tom criticized himself"
  - "himself" is an anaphor
  - Bound by "Tom" in same clause
  - Satisfies Principle A

✗ "Tom said that Mary criticized himself"
  - "himself" is an anaphor
  - Not bound in local clause (subject is "Mary")
  - Violates Principle A

---

Principle B (Pronouns must be free locally):

✓ "Tom criticized him" (where him ≠ Tom)
  - "him" is a pronoun
  - Free in local clause (not bound by "Tom")
  - Satisfies Principle B

✗ "Tom criticized him" (where him = Tom)
  - "him" is a pronoun
  - Bound by "Tom" in same clause
  - Violates Principle B

---

Principle C (R-expressions must be free everywhere):

✓ "He criticized Tom"
  - "Tom" is an R-expression
  - Free everywhere (not bound by "he")
  - Satisfies Principle C

✗ "He criticized Tom" (where he = Tom)
  - "Tom" is an R-expression
  - Bound by "he"
  - Violates Principle C
```

---

## Mention Tracking

```rust
pub struct MentionTracker {
    mentions: Vec<Mention>,
    entity_mentions: HashMap<EntityId, Vec<MentionId>>,
}

impl MentionTracker {
    pub fn add_mention(&mut self, mention: Mention) {
        let entity_id = mention.entity_id;
        let mention_id = mention.id;
        
        self.mentions.push(mention);
        self.entity_mentions
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(mention_id);
    }
    
    pub fn get_mentions_for_entity(&self, entity_id: EntityId) -> Vec<&Mention> {
        self.entity_mentions
            .get(&entity_id)
            .map(|mention_ids| {
                mention_ids.iter()
                    .filter_map(|id| self.mentions.iter().find(|m| m.id == *id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn get_coreference_chain(&self, entity_id: EntityId) -> Vec<EntityId> {
        // Return all entities that are coreferential with this entity
        // This requires maintaining coreference chains during resolution
        vec![entity_id]  // simplified
    }
}
```

---

## Discourse Update Pipeline

After each utterance is parsed, the discourse is updated:

```
Utterance parsed
    │
    ├─→ 1. Extract new entities
    │       Find entities not yet in discourse
    │       Assign new EntityId
    │       Add to entities list
    │
    ├─→ 2. Update entity salience
    │       For each entity in utterance:
    │         - Increment mention_count
    │         - Update last_mentioned
    │         - Update grammatical_role (subject/object)
    │         - Recalculate salience score
    │
    ├─→ 3. Update topic
    │       Find entity with highest salience
    │       If salience > threshold:
    │         Set as new topic
    │
    ├─→ 4. Record utterance summary
    │       Store: entities mentioned, frames, illocution
    │       Keep last N utterances (configurable)
    │
    └─→ 5. Update shared knowledge
            If entity mentioned 2+ times:
              Add to shared_knowledge
```

## Reference Resolution

When parsing a pronoun or definite description, the engine resolves it to a specific entity in the discourse.

### Resolution Algorithm

```rust
pub fn resolve_reference(
    ref_expr: ReferenceExpression,
    discourse: &Discourse,
) -> Result<EntityId, ResolutionError> {
    match ref_expr {
        // Personal pronoun: "he", "she", "it", "they"
        ReferenceExpression::Pronoun { person, number, gender } => {
            resolve_pronoun(person, number, gender, discourse)
        }
        
        // Reflexive: "himself", "się", "sobie"
        ReferenceExpression::Reflexive => {
            resolve_reflexive(discourse)
        }
        
        // Definite description: "the book", "ten dom"
        ReferenceExpression::Definite { noun, features } => {
            resolve_definite(noun, features, discourse)
        }
        
        // Demonstrative: "this", "that", "ten", "tamten"
        ReferenceExpression::Demonstrative { distance } => {
            resolve_demonstrative(distance, discourse)
        }
    }
}
```

### Pronoun Resolution

```rust
fn resolve_pronoun(
    person: Person,
    number: Number,
    gender: Option<Gender>,
    discourse: &Discourse,
) -> Result<EntityId, ResolutionError> {
    // Speaker/addressee are special cases
    if person == Person::First && number == Number::Singular {
        return Ok(discourse.speaker.id);
    }
    if person == Person::Second && number == Number::Singular {
        return Ok(discourse.addressee.id);
    }
    
    // Search discourse entities
    let candidates: Vec<&DiscourseEntity> = discourse.entities
        .iter()
        .filter(|e| {
            // Filter by grammatical features
            let entity_gender = e.entity.features.gender;
            let entity_number = e.entity.features.number;
            
            entity_number == Some(number)
                && (gender.is_none() || entity_gender == gender)
                && e.id != discourse.speaker.id  // not speaker
                && e.id != discourse.addressee.id // not addressee
        })
        .collect();
    
    if candidates.is_empty() {
        return Err(ResolutionError::NoCandidate);
    }
    
    // Rank by salience
    let mut ranked = candidates;
    ranked.sort_by(|a, b| {
        b.salience.total().partial_cmp(&a.salience.total()).unwrap()
    });
    
    Ok(ranked[0].id)
}
```

### Salience Calculation

```rust
impl SalienceScore {
    pub fn calculate(
        entity: &DiscourseEntity,
        current_utterance: UtteranceId,
        is_topic: bool,
    ) -> Self {
        // Recency: exponential decay
        let utterances_ago = current_utterance - entity.last_mentioned;
        let recency = (-0.5 * utterances_ago as f32).exp();
        
        // Frequency: logarithmic scaling
        let frequency = (entity.mention_count as f32 + 1.0).ln() / 5.0;
        let frequency = frequency.min(1.0);
        
        // Grammatical role: subjects are more salient
        let grammatical = match entity.grammatical_role {
            GrammaticalRole::Subject => 1.0,
            GrammaticalRole::Object => 0.6,
            GrammaticalRole::Oblique => 0.3,
        };
        
        // Topicality
        let topicality = if is_topic { 1.0 } else { 0.0 };
        
        Self { recency, frequency, grammatical, topicality }
    }
    
    pub fn total(&self) -> f32 {
        // Weighted sum
        self.recency * 0.3
            + self.frequency * 0.2
            + self.grammatical * 0.3
            + self.topicality * 0.2
    }
}
```

## Pro-Drop Logic (Polish)

Polish allows subject omission (pro-drop) when the subject is recoverable from context. The engine must decide when to omit the subject during generation.

### When to Drop Subject

```rust
pub fn should_drop_subject(
    utterance: &Utterance,
    discourse: &Discourse,
    engine: &PolishEngine,
) -> bool {
    // Polish allows pro-drop, but not always appropriate
    
    let subject = utterance.frames[0].subject();
    
    // Rule 1: If subject is speaker (1st person), can drop
    if subject.features.person == Some(Person::First) {
        return true;
    }
    
    // Rule 2: If subject is topic and highly salient, can drop
    if let Some(topic_id) = discourse.topic {
        if subject.id == topic_id {
            let topic_entity = discourse.get_entity(topic_id);
            if topic_entity.salience.total() > 0.7 {
                return true;
            }
        }
    }
    
    // Rule 3: If subject was just mentioned in previous utterance, can drop
    if let Some(last_utt) = discourse.history.last() {
        if last_utt.subject_id == Some(subject.id) {
            return true;
        }
    }
    
    // Rule 4: Verb morphology encodes person/number/gender
    // So subject is often redundant in Polish
    // Default: drop if salience > 0.5
    let subject_entity = discourse.get_entity(subject.id);
    subject_entity.salience.total() > 0.5
}
```

### Examples

```
Context: "Tomek poszedł do sklepu."
         (Tomek went to the store.)
         → discourse.topic = Tomek, salience = 1.0

Utterance: "Kupił jabłko."
           (He bought an apple.)
           
Generation (PL):
  - Subject: Tomek (3rd person, masculine)
  - Topic: yes, salience = 1.0
  - should_drop_subject() → true
  - Verb "kupił" encodes 3sg masculine
  - Output: "Kupił jabłko." (subject omitted)

Generation (EN):
  - English doesn't allow pro-drop
  - Must include subject
  - Resolve "Tomek" → "He" (pronoun, high salience)
  - Output: "He bought an apple."
```

## Topic Continuity

The topic is the entity that the discourse is currently "about". It persists across utterances until a new topic is introduced.

### Topic Detection

```rust
pub fn detect_topic(
    utterance: &Utterance,
    discourse: &Discourse,
) -> Option<EntityId> {
    // Heuristic: subject of first sentence is likely the topic
    
    if let Some(first_sentence) = utterance.sentences.first() {
        if let Some(subject) = first_sentence.frames[0].subject() {
            let entity = discourse.get_entity(subject.id);
            
            // If this entity is already highly salient, it's the topic
            if entity.salience.total() > 0.8 {
                return Some(entity.id);
            }
            
            // If this is a new entity introduced as subject, it might be new topic
            if entity.mention_count == 1 {
                return Some(entity.id);
            }
        }
    }
    
    // Otherwise, keep existing topic
    discourse.topic
}
```

### Topic Shift

When a new topic is introduced, the old topic's salience decreases but it remains in the discourse:

```
Utterance 1: "Tomek poszedł do sklepu."
             → topic = Tomek

Utterance 2: "W sklepie spotkał Annę."
             → topic shifts to Anna (new entity, subject position)
             → Tomek remains in discourse, salience decreases

Utterance 3: "Ona kupiła chleb."
             → "Ona" resolves to Anna (current topic, high salience)
             → Output: "She bought bread."
```

## Discourse History

The discourse maintains a summary of recent utterances for reference resolution and coherence.

```rust
pub struct UtteranceSummary {
    pub id: UtteranceId,
    pub speaker: EntityId,
    pub addressee: EntityId,
    pub entities_mentioned: Vec<EntityId>,
    pub subject: Option<EntityId>,
    pub frames: Vec<FrameSummary>,
    pub illocution: Illocution,
}

pub struct FrameSummary {
    pub frame_type: FrameType,
    pub roles: HashMap<Role, EntityId>,
}
```

### History Management

```rust
impl Discourse {
    pub fn add_utterance(&mut self, utterance: &Utterance) {
        // Create summary
        let summary = UtteranceSummary::from(utterance, self);
        
        // Add to history
        self.history.push(summary);
        
        // Keep only last N utterances (e.g., 10)
        const MAX_HISTORY: usize = 10;
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        
        // Update entity salience
        self.update_salience(utterance);
        
        // Update topic
        self.topic = detect_topic(utterance, self);
    }
}
```

## Shared Knowledge

Entities that have been mentioned multiple times become "shared knowledge" — both participants know about them. This affects definiteness and reference:

```rust
impl Discourse {
    pub fn update_shared_knowledge(&mut self, utterance: &Utterance) {
        for entity_id in utterance.all_entities() {
            let entity = self.get_entity(entity_id);
            
            // If mentioned 2+ times, it's shared knowledge
            if entity.mention_count >= 2 {
                self.shared_knowledge.insert(entity_id);
            }
        }
    }
}
```

### Impact on Generation

Shared knowledge affects article choice (English) and reference strategy:

```
Context: "Mam książkę. Książka jest ciekawa."
         (I have a book. The book is interesting.)
         
Utterance 3: "Książka jest długa."
             (The book is long.)

Generation (EN):
  - "książka" (book) is in shared_knowledge (mentioned 2+ times)
  - Use definite article: "The book"
  - High salience → could also use pronoun: "It"
  - Decision: if salience > 0.8, use pronoun
              else use "the book"
```

## Multi-Turn Conversation

For future bot integration, discourse must persist across multiple turns:

```rust
pub struct Conversation {
    pub participants: Vec<Entity>,
    pub discourse: Discourse,
    pub turn_count: usize,
}

impl Conversation {
    pub fn new(speaker: Entity, addressee: Entity) -> Self {
        Self {
            participants: vec![speaker.clone(), addressee.clone()],
            discourse: Discourse::new(speaker, addressee),
            turn_count: 0,
        }
    }
    
    pub fn process_turn(&mut self, utterance: &str, speaker: EntityId) -> String {
        // Update speaker for this turn
        self.discourse.speaker = self.participants.iter()
            .find(|e| e.id == speaker)
            .unwrap()
            .clone();
        
        // Parse utterance
        let parsed = self.engine.parse(utterance, &self.discourse);
        
        // Update discourse
        self.discourse.add_utterance(&parsed);
        
        // Generate response (future: use LLM or rules)
        let response = self.generate_response(&parsed);
        
        self.turn_count += 1;
        response
    }
}
```

## Discourse Serialization

For persistence or debugging, discourse can be serialized:

```rust
impl Discourse {
    pub fn to_ron(&self) -> String {
        ron::to_string(self).unwrap()
    }
    
    pub fn from_ron(ron: &str) -> Result<Self, ron::Error> {
        ron::from_str(ron)
    }
}
```

This allows saving conversation state and resuming later.

---

## Coreference Resolution

Coreference resolution identifies when multiple expressions refer to the same entity. This is crucial for understanding pronouns, definite descriptions, and repeated mentions.

### Types of Coreference

**1. Pronominal Coreference**

Pronouns refer back to previously mentioned entities.

```
"Tomek dał jabłko Izie. Ona się ucieszyła."
  → "Ona" refers to "Iza"
```

```rust
pub struct CoreferenceLink {
    /// The referring expression (pronoun, description)
    pub anaphor: CorefExpression,
    
    /// The entity being referred to
    pub antecedent: EntityId,
    
    /// Confidence in the resolution
    pub confidence: f64,
}

pub enum CorefExpression {
    Pronoun {
        text: String,
        features: FeatureBundle,
    },
    DefiniteDescription {
        text: String,
    },
    Demonstrative {
        text: String,
        distance: Distance,
    },
}
```

**2. Identity Coreference**

Different expressions referring to the same real-world entity.

```
"Prezydent przyszedł. Andrzej Duda przemówił."
  → "Prezydent" and "Andrzej Duda" refer to the same person
```

**3. Bridging Reference**

Reference to something not explicitly mentioned but inferable.

```
"Tomek kupił samochód. Silnik jest głośny."
  → "Silnik" refers to the engine of the car (not explicitly mentioned)
```

### Coreference Resolution Algorithm

See [Coreference Resolution](#coreference-resolution) above for the complete CoreferenceResolver implementation with binding theory constraints.

The CoreferenceResolver handles:
- Pronouns (personal, reflexive)
- Definite descriptions
- Demonstratives
- Proper names

It applies binding theory constraints (Principles A, B, C) and ranks candidates by salience score.

### Constraints on Coreference

**1. Gender Agreement**

Pronouns must agree in gender with their antecedents.

```
"Tomek dał jabłko Izie. On* się ucieszył."
  → "On" (masculine) → "Tomek" (masculine) ✓
  → "On" (masculine) → "Iza" (feminine) ✗

"Tomek dał jabłko Izie. Ona* się ucieszyła."
  → "Ona" (feminine) → "Iza" (feminine) ✓
  → "Ona" (feminine) → "Tomek" (masculine) ✗
```

**2. Number Agreement**

Pronouns must agree in number.

```
"Studenci zdali egzamin. Oni* byli zadowoleni."
  → "Oni" (plural) → "Studenci" (plural) ✓
```

**3. Syntactic Constraints**

Reflexive pronouns must refer to the subject of their clause.

```
"Tomek widzi siebie w lustrze."
  → "siebie" → "Tomek" (subject) ✓

"Tomek widzi go w lustrze."
  → "go" → someone else (not Tomek) ✓
```

**4. Recency Bias**

More recently mentioned entities are preferred.

```
"Tomek dał jabłko Izie. Ona się ucieszyła."
  → "Ona" → "Iza" (most recent feminine entity) ✓
  → "Ona" → "mama" (mentioned earlier) ✗ (less preferred)
```

---

## Discourse Markers

Discourse markers are words/phrases that signal relationships between utterances or guide the flow of discourse.

### Types of Discourse Markers

**1. Additive Markers**

Signal addition or continuation.

| Polish | English | Function |
|--------|---------|----------|
| "i" | "and" | simple addition |
| "oraz" | "and also" | formal addition |
| "ponadto" | "moreover" | adding information |
| "co więcej" | "what's more" | emphasizing addition |
| "dodatkowo" | "additionally" | formal addition |

**2. Adversative Markers**

Signal contrast or opposition.

| Polish | English | Function |
|--------|---------|----------|
| "ale" | "but" | contrast |
| "jednak" | "however" | formal contrast |
| "natomiast" | "on the other hand" | contrast |
| "a" | "whereas" | contrast (Polish-specific) |
| "chociaż" | "although" | concession |

**3. Causal Markers**

Signal cause-effect relationships.

| Polish | English | Function |
|--------|---------|----------|
| "bo" | "because" | reason |
| "dlatego" | "therefore" | result |
| "więc" | "so" | consequence |
| "zatem" | "thus" | formal consequence |
| "w związku z tym" | "consequently" | formal consequence |

**4. Temporal Markers**

Signal temporal relationships.

| Polish | English | Function |
|--------|---------|----------|
| "potem" | "then" | sequence |
| "następnie" | "next" | sequence |
| "w końcu" | "finally" | conclusion |
| "tymczasem" | "meanwhile" | simultaneous |
| "wcześniej" | "earlier" | prior |

**5. Elaboration Markers**

Signal clarification or exemplification.

| Polish | English | Function |
|--------|---------|----------|
| "mianowicie" | "namely" | specification |
| "na przykład" | "for example" | exemplification |
| "innymi słowy" | "in other words" | paraphrase |
| "to znaczy" | "that is" | clarification |

### Representing Discourse Markers

```rust
pub enum DiscourseMarker {
    Additive(AdditiveType),
    Adversative(AdversativeType),
    Causal(CausalType),
    Temporal(TemporalType),
    Elaboration(ElaborationType),
}

pub enum CausalType {
    Reason,       // "bo", "because"
    Result,       // "dlatego", "therefore"
    Consequence,  // "więc", "so"
}

pub struct DiscourseRelation {
    pub marker: DiscourseMarker,
    pub source: UtteranceId,
    pub target: UtteranceId,
    pub relation_type: RelationType,
}
```

### Discourse Marker Detection

```rust
pub fn detect_discourse_marker(
    utterance: &str,
) -> Option<DiscourseMarker> {
    let words: Vec<&str> = utterance.split_whitespace().collect();
    
    if words.is_empty() {
        return None;
    }
    
    // Check first word (most common position for discourse markers)
    match words[0] {
        "Ale" | "ale" => Some(DiscourseMarker::Adversative(AdversativeType::Contrast)),
        "Bo" | "bo" => Some(DiscourseMarker::Causal(CausalType::Reason)),
        "Więc" | "więc" => Some(DiscourseMarker::Causal(CausalType::Consequence)),
        "Potem" | "potem" => Some(DiscourseMarker::Temporal(TemporalType::Sequence)),
        "Ponadto" | "ponadto" => Some(DiscourseMarker::Additive(AdditiveType::Addition)),
        _ => None,
    }
}
```

### Discourse Markers in Interlingua

Discourse markers create relations between sentences/utterances:

```rust
// See INTERLINGUA.md for canonical Utterance definition
// Utterance includes discourse_relations for tracking relationships between sentences

pub struct Utterance {
    pub sentences: Vec<Sentence>,
    pub discourse_relations: Vec<DiscourseRelation>,
}

// Example: "Tomek przyszedł. Ale Iza nie przyszła."
Utterance {
    sentences: [
        Sentence { /* Tomek przyszedł */ },
        Sentence { /* Iza nie przyszła */ },
    ],
    discourse_relations: [
        DiscourseRelation {
            marker: DiscourseMarker::Adversative(AdversativeType::Contrast),
            source: UtteranceId(0),
            target: UtteranceId(1),
            relation_type: RelationType::Contrast,
        }
    ],
}
```

---

## Information Structure

Information structure describes how information is organized in terms of what is given (known) vs. new, and what is the topic vs. focus.

### Topic and Focus

**Topic** — what the sentence is about (given information)
**Focus** — what is new or emphasized

```
"Tomek dał jabłko Izie."
  Topic: Tomek (what we're talking about)
  Focus: dał jabłko Izie (what we're saying about Tomek)

"Jabłko dał Tomek Izie."
  Topic: jabłko (what we're talking about)
  Focus: dał Tomek Izie (what we're saying about the apple)
```

### Given vs. New Information

**Given** — already established in discourse
**New** — newly introduced

```
"Tomek przyszedł. Dał jabłko Izie."
  Sentence 1: "Tomek" = new, "przyszedł" = new
  Sentence 2: "Tomek" = given (from sentence 1), "jabłko" = new, "Iza" = new
```

### Representing Information Structure

### Sentence with Information Structure

See [INTERLINGUA.md](./INTERLINGUA.md#sentence) for the canonical Sentence definition.

Sentence includes information structure fields:
- `topic: Option<EntityRef>` - what the sentence is about
- `focus: Option<EntityRef>` - what carries emphasis
- `given: Vec<EntityRef>` - already established in discourse
- `new: Vec<EntityRef>` - newly introduced information

### Information Structure

See [INTERLINGUA.md](./INTERLINGUA.md#information-structure-extended) for complete definitions of:
- `InformationStructure` (with presupposition)
- `TopicStatus` (CurrentTopic, ContinuingTopic, ShiftedTopic)
- `FocusStatus` (Broad, Narrow, Contrastive, Verum)

### Topic-Focus Articulation in Polish

Polish has relatively free word order, which is used to mark topic and focus:

```
Neutral (SVO):
  "Tomek dał jabłko Izie."
  Topic: Tomek, Focus: dał jabłko Izie

Topic-first (emphasizing topic):
  "Tomek, on dał jabłko Izie."
  Topic: Tomek (emphasized), Focus: dał jabłko Izie

Focus-first (emphasizing new info):
  "Jabłko dał Tomek Izie."
  Topic: jabłko, Focus: dał Tomek Izie

Contrastive focus:
  "Jabłko dał Tomek, nie gruszkę."
  Focus: jabłko (contrasted with gruszkę)
```

### Topic-Focus Articulation in English

English uses intonation and cleft constructions:

```
Neutral (SVO):
  "Tomek gave Iza an apple."
  Topic: Tomek, Focus: gave Iza an apple

Cleft construction (emphasizing focus):
  "It was Tomek who gave Iza an apple."
  Focus: Tomek

Pseudo-cleft:
  "What Tomek gave Iza was an apple."
  Focus: an apple

Topicalization:
  "The apple, Tomek gave to Iza."
  Topic: the apple (fronted)
```

### Information Structure and Generation

The generator uses information structure to decide word order and pronoun usage:

```rust
pub fn generate_with_info_structure(
    sentence: &Sentence,
    info_structure: &InformationStructure,
) -> String {
    match info_structure.topic {
        TopicStatus::CurrentTopic => {
            // Topic is already established → can use pronoun or pro-drop
            if sentence.language == "pl" {
                // Polish: pro-drop if topic is clear
                generate_without_subject(sentence)
            } else {
                // English: use pronoun
                generate_with_pronoun(sentence)
            }
        }
        
        TopicStatus::ShiftedTopic => {
            // New topic → use full noun phrase
            generate_with_full_np(sentence)
        }
        
        _ => generate_neutral(sentence),
    }
}
```

---

## Summary

Advanced discourse features:

1. **Coreference resolution** — identifying when expressions refer to the same entity
   - Pronominal coreference (pronouns → antecedents)
   - Identity coreference (different expressions, same entity)
   - Bridging reference (inferable entities)
   - Constraints: gender, number, syntax, recency

2. **Discourse markers** — signaling relationships between utterances
   - Additive (and, moreover)
   - Adversative (but, however)
   - Causal (because, therefore)
   - Temporal (then, next)
   - Elaboration (namely, for example)

3. **Information structure** — organizing given vs. new, topic vs. focus
   - Topic: what the sentence is about
   - Focus: what is new or emphasized
   - Given/New: information status
   - Word order reflects information structure (especially in Polish)

These features enable coherent multi-sentence and multi-utterance discourse, allowing the system to track entities across sentences, understand discourse relations, and generate appropriate word order and pronoun usage.
