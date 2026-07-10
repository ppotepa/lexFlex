use thiserror::Error;

use crate::core::interlingua::{ConceptId, SemanticRole};

#[derive(Error, Debug)]
pub enum LexFlexError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Deduction error: {0}")]
    Deduction(#[from] DeductionError),

    #[error("Generate error: {0}")]
    Generate(#[from] GenerateError),

    #[error("Translate error: {0}")]
    Translate(#[from] TranslateError),

    #[error("Data error: {0}")]
    Data(#[from] DataError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unknown word: '{word}'")]
    UnknownWord { word: String },

    #[error("No verb found in sentence")]
    NoVerbFound,

    #[error("Empty input")]
    EmptyInput,

    #[error("Tokenization failed: {message}")]
    TokenizationFailed { message: String },
}

#[derive(Error, Debug)]
pub enum DeductionError {
    #[error("Unresolvable case for token '{token}': possible roles: {possible_roles:?}")]
    UnresolvableCase {
        token: String,
        possible_roles: Vec<SemanticRole>,
    },

    #[error("Semantic type violation: role {role:?} expected {expected}, found {found}")]
    SemanticTypeViolation {
        role: SemanticRole,
        expected: ConceptId,
        found: ConceptId,
    },

    #[error("Ambiguous pronoun: '{pronoun}'")]
    AmbiguousPronoun { pronoun: String },

    #[error("Missing required role {role:?} for frame {frame_type}")]
    MissingRequiredRole {
        role: SemanticRole,
        frame_type: String,
    },

    #[error("Unknown verb lemma: '{lemma}'")]
    UnknownVerb { lemma: String },

    #[error("Unknown frame type: '{frame_type}'")]
    UnknownFrameType { frame_type: String },

    #[error("No verb found for deduction")]
    NoVerbFound,
}

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Unknown concept: '{concept}'")]
    UnknownConcept { concept: String },

    #[error("Morphological inflection failed for '{lemma}': {reason}")]
    InflectionFailed { lemma: String, reason: String },

    #[error("No lexeme found for concept '{concept}' in language '{language}'")]
    NoLexemeForConcept { concept: String, language: String },

    #[error("Unsupported interlingua type")]
    UnsupportedInterlinguaType,
}

#[derive(Error, Debug)]
pub enum TranslateError {
    #[error("Unsupported source language: {language}")]
    UnsupportedSourceLanguage { language: String },

    #[error("Unsupported target language: {language}")]
    UnsupportedTargetLanguage { language: String },

    #[error("Feature inexpressible in target language '{target}': {features:?}")]
    InexpressibleInTarget {
        target: String,
        features: Vec<String>,
    },
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Failed to load RON file '{path}': {message}")]
    LoadFailed { path: String, message: String },

    #[error("Invalid data in '{path}': {message}")]
    InvalidData { path: String, message: String },

    #[error("File not found: '{path}'")]
    FileNotFound { path: String },
}
