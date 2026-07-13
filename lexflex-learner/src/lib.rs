//! lexflex-learner
//!
//! Lexical deduction service for lexFlex.
//! Queries multiple free APIs (ConceptNet, DictionaryAPI, Wiktionary, etc.)
//! to gather maximum data about unknown words and infer:
//! - Surface forms and grammatical features
//! - Semantic relations (especially IsA for concept mapping)
//! - Best matching Interlingua ConceptId
//!
//! Can be used as:
//! - CLI: `lexlearn deduce "słowo" --lang pl`
//! - Library crate in other projects (including main lexFlex)

pub mod bulk;
pub mod error;
pub mod evidence;
pub mod scoring;
pub mod service;
pub mod sources;
pub mod types;

pub use error::LearnerError;
pub use service::LexicalDeductionService;
pub use types::{
    DeductionResult, Language, SurfaceInfo, SemanticInfo, ConceptMatch,
    ProposedConcept, ProposedConceptDef, LexiconProposal,
};

// Re-export for convenience when used as crate
pub use serde_json;
pub use ron;
