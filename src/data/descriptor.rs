use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::interlingua::{Case, SemanticRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDescriptor {
    pub language: String,
    pub name: String,
    pub morphology: MorphologyDescriptor,
    pub syntax: SyntaxDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologyDescriptor {
    pub has_cases: bool,
    pub cases: Vec<Case>,
    pub has_articles: bool,
    pub aspect_type: AspectType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectType {
    Morphological,
    Periphrastic,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxDescriptor {
    pub word_order: WordOrder,
    pub pro_drop: bool,
    pub negation_particle: String,
    /// Mapowanie przyimków na role semantyczne (data-driven)
    pub preposition_roles: HashMap<String, SemanticRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordOrder {
    SVO,
    SOV,
    VSO,
    Free,
}
