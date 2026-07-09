# Memory — Long-Term Knowledge Storage

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Long-term memory is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

Long-term memory stores information about users, their preferences, conversation history, and learned facts that persist across sessions.

---

## Memory Architecture

```rust
pub struct LongTermMemory {
    /// User profile and preferences
    pub user_profile: UserProfile,
    
    /// Known facts about the world
    pub facts: FactStore,
    
    /// Conversation history (summarized)
    pub conversation_history: Vec<ConversationSummary>,
    
    /// Learned patterns
    pub patterns: PatternStore,
    
    /// Persistence backend
    backend: MemoryBackend,
}

pub trait MemoryBackend {
    fn save(&self, key: &str, value: &dyn Serialize) -> Result<(), MemoryError>;
    fn load(&self, key: &str) -> Result<Option<Box<dyn DeserializeOwned>>, MemoryError>;
    fn delete(&self, key: &str) -> Result<(), MemoryError>;
    fn list_keys(&self, prefix: &str) -> Vec<String>;
}
```

---

## User Profile

```rust
pub struct UserProfile {
    /// User's name (if known)
    pub name: Option<String>,
    
    /// Preferred language
    pub preferred_language: LanguageId,
    
    /// Preferred register
    pub preferred_register: Register,
    
    /// Communication style preferences
    pub style: CommunicationStyle,
    
    /// Known facts about the user
    pub personal_facts: Vec<Fact>,
    
    /// User's interests
    pub interests: Vec<Entity>,
    
    /// Topics to avoid
    pub avoid_topics: Vec<Entity>,
    
    /// Interaction history summary
    pub interaction_summary: InteractionSummary,
}

pub struct CommunicationStyle {
    /// Preferred verbosity
    pub verbosity: Verbosity,
    
    /// Preferred formality
    pub formality: Register,
    
    /// Emoji usage preference
    pub emoji_preference: EmojiPreference,
    
    /// Response length preference
    pub response_length: ResponseLength,
}

pub enum Verbosity { Brief, Normal, Detailed }
pub enum EmojiPreference { Never, Sometimes, Often }
pub enum ResponseLength { Short, Medium, Long }
```

---

## Fact Store

```rust
pub struct FactStore {
    facts: Vec<Fact>,
    index: HashMap<String, Vec<usize>>,  // topic → fact indices
}

pub struct Fact {
    /// Unique identifier
    pub id: FactId,
    
    /// Subject of the fact
    pub subject: Entity,
    
    /// Predicate (relationship)
    pub predicate: ConceptId,
    
    /// Object (what is being said about subject)
    pub object: Option<FactValue>,
    
    /// Confidence (0.0 - 1.0)
    pub confidence: f64,
    
    /// Source of the fact
    pub source: FactSource,
    
    /// When the fact was learned
    pub learned_at: Timestamp,
    
    /// When the fact was last verified
    pub last_verified: Option<Timestamp>,
    
    /// Expiration (facts can become outdated)
    pub expires_at: Option<Timestamp>,
}

pub enum FactValue {
    Entity(Entity),
    Number(f64),
    String(String),
    Boolean(bool),
    Date(Timestamp),
    Set(Vec<FactValue>),
}

pub enum FactSource {
    /// User explicitly stated this
    UserStatement { turn_id: usize },
    
    /// Inferred from conversation
    Inferred { from_turns: Vec<usize> },
    
    /// From external source (API, database)
    External { source: String },
    
    /// Default/assumed
    Default,
}
```

### Fact Extraction

```rust
impl FactStore {
    /// Extract facts from an utterance
    pub fn extract_facts(
        &mut self,
        utterance: &InterlinguaNode,
        turn_id: usize,
    ) -> Vec<Fact> {
        let mut new_facts = vec![];
        
        match utterance {
            InterlinguaNode::Natural(utt) => {
                for sentence in &utt.sentences {
                    for frame in &sentence.frames {
                        if let Some(fact) = Self::frame_to_fact(frame, turn_id) {
                            new_facts.push(fact);
                        }
                    }
                }
            }
            _ => {}
        }
        
        // Add to store
        for fact in &new_facts {
            self.add_fact(fact.clone());
        }
        
        new_facts
    }
    
    fn frame_to_fact(frame: &Frame, turn_id: usize) -> Option<Fact> {
        match frame {
            Frame::Statement { subject, property } => {
                Some(Fact {
                    id: FactId::new(),
                    subject: subject.clone(),
                    predicate: property.concept,
                    object: Some(FactValue::Entity(property.clone())),
                    confidence: 0.9,
                    source: FactSource::UserStatement { turn_id },
                    learned_at: Timestamp::now(),
                    last_verified: None,
                    expires_at: None,
                })
            }
            
            Frame::Possession { possessor, possessed } => {
                Some(Fact {
                    id: FactId::new(),
                    subject: possessor.clone(),
                    predicate: ConceptId::POSSESS,
                    object: Some(FactValue::Entity(possessed.clone())),
                    confidence: 0.9,
                    source: FactSource::UserStatement { turn_id },
                    learned_at: Timestamp::now(),
                    last_verified: None,
                    expires_at: None,
                })
            }
            
            _ => None,
        }
    }
}
```

### Fact Retrieval

```rust
impl FactStore {
    /// Find facts relevant to a query
    pub fn retrieve_relevant(
        &self,
        query: &InterlinguaNode,
        max_results: usize,
    ) -> Vec<&Fact> {
        let query_entities = Self::extract_entities(query);
        let query_concepts = Self::extract_concepts(query);
        
        let mut scored: Vec<(&Fact, f64)> = self.facts.iter()
            .map(|fact| {
                let score = Self::relevance_score(fact, &query_entities, &query_concepts);
                (fact, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.into_iter()
            .take(max_results)
            .map(|(fact, _)| fact)
            .collect()
    }
    
    fn relevance_score(
        fact: &Fact,
        query_entities: &[Entity],
        query_concepts: &[ConceptId],
    ) -> f64 {
        let mut score = 0.0;
        
        // Entity match
        if query_entities.iter().any(|e| *e == fact.subject) {
            score += 0.5;
        }
        
        // Concept match
        if query_concepts.contains(&fact.predicate) {
            score += 0.3;
        }
        
        // Recency bonus
        let age_hours = fact.learned_at.hours_ago();
        score += 0.2 * (-age_hours / 24.0).exp();
        
        // Confidence
        score *= fact.confidence;
        
        score
    }
}
```

---

## Conversation History

```rust
pub struct ConversationSummary {
    /// Session identifier
    pub session_id: String,
    
    /// Start and end time
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    
    /// Number of turns
    pub turn_count: usize,
    
    /// Main topics discussed
    pub topics: Vec<Entity>,
    
    /// Key facts exchanged
    pub key_facts: Vec<FactId>,
    
    /// User intents
    pub intents: Vec<Intent>,
    
    /// Outcome (satisfied, unresolved, etc.)
    pub outcome: ConversationOutcome,
}

pub enum ConversationOutcome {
    /// User got what they wanted
    Satisfied,
    
    /// Conversation ended without resolution
    Unresolved,
    
    /// User was confused/frustrated
    Negative,
    
    /// Still ongoing
    Ongoing,
}
```

---

## Pattern Learning

```rust
pub struct PatternStore {
    /// Learned user preferences
    pub user_preferences: Vec<LearnedPreference>,
    
    /// Learned conversation patterns
    pub conversation_patterns: Vec<ConversationPattern>,
}

pub struct LearnedPreference {
    /// What the user prefers
    pub preference: Preference,
    
    /// How confident we are (0.0 - 1.0)
    pub confidence: f64,
    
    /// Evidence (which turns led to this)
    pub evidence: Vec<usize>,
}

pub struct ConversationPattern {
    /// Pattern description
    pub description: String,
    
    /// When this pattern occurs
    pub trigger: PatternTrigger,
    
    /// What to do
    pub response: PatternResponse,
    
    /// Success rate
    pub success_rate: f64,
}
```

---

## Memory Update

```rust
impl LongTermMemory {
    /// Update memory from a new utterance
    pub fn update_from_utterance(&mut self, utterance: &InterlinguaNode) {
        // Extract new facts
        let new_facts = self.facts.extract_facts(utterance, self.current_turn);
        
        // Update user profile if applicable
        self.update_profile(utterance);
        
        // Update patterns
        self.patterns.update(utterance, &new_facts);
        
        // Persist to backend
        self.persist();
    }
    
    fn update_profile(&mut self, utterance: &InterlinguaNode) {
        // Check for explicit preference statements
        // "I prefer..." / "Wolę..." / "Nie lubię..."
        
        if let Some(preference) = Self::extract_preference(utterance) {
            self.user_profile.style.verbosity = preference.verbosity;
        }
        
        // Check for name introduction
        // "My name is..." / "Mam na imię..."
        if let Some(name) = Self::extract_name(utterance) {
            self.user_profile.name = Some(name);
        }
    }
}
```

---

## Memory Persistence

```rust
pub enum PersistenceStrategy {
    /// Save after every utterance
    Immediate,
    
    /// Save at end of conversation
    OnSessionEnd,
    
    /// Save periodically (every N turns)
    Periodic { interval: usize },
}

pub struct FileBackend {
    base_dir: PathBuf,
}

impl MemoryBackend for FileBackend {
    fn save(&self, key: &str, value: &dyn Serialize) -> Result<(), MemoryError> {
        let path = self.base_dir.join(format!("{}.ron", key));
        let ron = ron::to_string(value).map_err(MemoryError::SerializationError)?;
        std::fs::write(path, ron).map_err(MemoryError::IOError)?;
        Ok(())
    }
    
    fn load(&self, key: &str) -> Result<Option<Box<dyn DeserializeOwned>>, MemoryError> {
        let path = self.base_dir.join(format!("{}.ron", key));
        if !path.exists() {
            return Ok(None);
        }
        
        let ron = std::fs::read_to_string(path).map_err(MemoryError::IOError)?;
        let value = ron::from_str(&ron).map_err(MemoryError::DeserializationError)?;
        Ok(Some(Box::new(value)))
    }
}
```

---

## Memory in Response Planning

```rust
impl ResponsePlanner {
    /// Use memory to personalize response
    fn personalize_response(
        plan: ResponsePlan,
        memory: &LongTermMemory,
    ) -> ResponsePlan {
        let mut personalized = plan;
        
        // Use user's name if known
        if let Some(name) = &memory.user_profile.name {
            personalized = Self::add_name(personalized, name);
        }
        
        // Match user's preferred verbosity
        personalized = Self::adjust_verbosity(
            personalized,
            memory.user_profile.style.verbosity,
        );
        
        // Reference known facts
        personalized = Self::reference_known_facts(personalized, &memory.facts);
        
        // Avoid topics user doesn't like
        personalized = Self::filter_avoided_topics(
            personalized,
            &memory.user_profile.avoid_topics,
        );
        
        personalized
    }
}
```

---

## Summary

Long-term memory:

1. **Stores user profile** — name, preferences, communication style
2. **Accumulates facts** — learned from conversations
3. **Summarizes history** — condensed conversation summaries
4. **Learns patterns** — user preferences, common interactions
5. **Persists across sessions** — backed by file/database storage
6. **Retrieves relevant facts** — scored by relevance, recency, confidence
7. **Personalizes responses** — adapts to user's style and preferences
