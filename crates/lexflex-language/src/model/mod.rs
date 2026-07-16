mod category;
mod denotation;
mod feature;
mod form;
mod lexeme;
mod meaning;
mod sense;
mod valency;

pub use crate::loader::MorphologyParadigm;
pub use category::{AtomicCategoryKind, CategoryType, SlashDirection, SyntacticCategory};
pub use denotation::SemanticAnchor;
pub use feature::{FeatureConflict, FeatureStructure};
pub use form::Form;
pub use lexeme::Lexeme;
pub use meaning::MeaningTemplate;
pub use sense::LexicalSense;
pub use valency::ValencySlot;
