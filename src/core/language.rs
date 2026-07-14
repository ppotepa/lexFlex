//! Data-shaped language contracts shared by surface parsers and the engine.
//!
//! The parser implementations may use optimized indexes internally, but the
//! semantic boundary is expressed through these serializable profiles. This
//! keeps language-specific morphology and constructions out of query and
//! knowledge code.

use serde::{Deserialize, Serialize};

use super::interlingua::{ConceptId, FeatureBundle, LanguageId, SemanticRole};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructionToken {
    Surface(String),
    Concept(ConceptId),
    PartOfSpeech(String),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterlinguaTemplate {
    Concept(ConceptId),
    Predicate(ConceptId),
    Role { role: SemanticRole, variable: String },
    Question { predicate: ConceptId, projection: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionProfile {
    pub id: ConstructionId,
    pub language: LanguageId,
    pub pattern: Vec<ConstructionToken>,
    pub output: InterlinguaTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorphologyCategory {
    Noun,
    Verb,
    Adjective,
    Pronoun,
    Case,
    Number,
    Gender,
    Definiteness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MorphologyTransformation {
    LexiconEntry,
    Paradigm(String),
    FeatureRewrite(FeatureBundle),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphologyException {
    pub surface: String,
    pub lemma: String,
    pub features: FeatureBundle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphologyRule {
    pub language: LanguageId,
    pub category: MorphologyCategory,
    pub transformation: MorphologyTransformation,
    pub exceptions: Vec<MorphologyException>,
}
