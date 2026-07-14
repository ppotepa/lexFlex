use serde::{Deserialize, Serialize};
use std::fmt;

mod frame;
mod questions;
mod sentence;

pub use frame::*;
pub use questions::*;
pub use sentence::*;

// ─── Identifiers ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub String);

impl ConceptId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl fmt::Display for ConceptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageId(pub String);

impl LanguageId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Morphological Enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    MasculinePersonal,
    MasculineAnimate,
    MasculineInanimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Number {
    Singular,
    Plural,
    Dual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Person {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Animacy {
    Animate,
    Inanimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Definiteness {
    Definite,
    Indefinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Countability {
    Count,
    Mass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Concreteness {
    Concrete,
    Abstract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
    Prepositional,
    Partitive,
    Inessive,
    Elative,
    Illative,
    Adessive,
    Ablative,
    Allative,
    Essive,
    Translative,
    Comitative,
    Abessive,
    Ergative,
    Absolutive,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aspect {
    Perfective,
    Imperfective,
    Progressive,
    Habitual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mood {
    Indicative,
    Subjunctive,
    Imperative,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Voice {
    Active,
    Passive,
    Middle,
    Antipassive,
    Causative,
    Applicative,
    Reflexive,
    Reciprocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Evidentiality {
    Direct,
    Reported,
    Inferred,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HonorificLevel {
    Plain,
    Polite,
    Honorific,
    Humble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Classifier {
    General,
    Person,
    Animal,
    Flat,
    Long,
    Book,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

// ─── FeatureBundle ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeatureBundle {
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub case: Option<Case>,
    pub animacy: Option<Animacy>,
    pub person: Option<Person>,
    pub definiteness: Option<Definiteness>,
    pub countability: Option<Countability>,
    pub concreteness: Option<Concreteness>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
    pub evidentiality: Option<Evidentiality>,
    pub honorific_level: Option<HonorificLevel>,
    pub classifier: Option<Classifier>,
    pub degree: Option<Degree>,
    /// Phonetic / article hint from lexicon data (e.g. "vowel", "consonant", or first letter class).
    /// Enables descriptor/policy driven a/an (RESOLVED: no surface spelling logic).
    pub initial_sound: Option<String>,
    /// Suppletive stem for comparative (data-driven, e.g. special stem for positive base).
    pub suppletive_comparative: Option<String>,
    /// Suppletive stem for superlative.
    pub suppletive_superlative: Option<String>,
    /// Semantic role assigned by preposition (data-driven, e.g. Goal, Source, Location).
    /// Enables direct role assignment without case inference.
    pub semantic_role: Option<SemanticRole>,
}

// ─── Semantic Roles ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRole {
    Agent,
    Patient,
    Theme,
    Recipient,
    Experiencer,
    Stimulus,
    Source,
    Goal,
    Location,
    Instrument,
    Beneficiary,
    Topic,
    Creator,
    Created,
    Cognizer,
    Content,
    Speaker,
    Message,
    Accompaniment,
}

// ─── Reference ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reference {
    Direct,
    Anaphoric(String),
    Cataphoric(String),
    Deictic,
    Generic,
    Unresolved,
}

// ─── Coordination (for enumerations/lists) ───────────────────────────────────
// Defined here so Entity can reference it directly (first-class support).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordination {
    pub items: Vec<Entity>,
    pub conjunction: String,  // "i", "and", etc.
}

// ─── Entity ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub concept: ConceptId,
    pub name: Option<String>,
    pub features: FeatureBundle,
    pub reference: Reference,
    pub id: Option<EntityId>,
    pub coordination: Option<Coordination>,  // first-class lists/enumerations
    /// Adjectival modifiers for this NP head. Populated by parser grouping (structural, no concat of names per 21pts).
    /// This eliminates the need for split/concat (RESOLVED 21pts).
    pub adjectives: Vec<Entity>,
}

impl Entity {
    pub fn new(concept: ConceptId) -> Self {
        Self {
            concept,
            name: None,
            features: FeatureBundle::default(),
            reference: Reference::Direct,
            id: None,
            coordination: None,
            adjectives: vec![],
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_features(mut self, features: FeatureBundle) -> Self {
        self.features = features;
        self
    }

    pub fn with_coordination(mut self, coord: Coordination) -> Self {
        self.coordination = Some(coord);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarity {
    Positive,
    Negative,
}
