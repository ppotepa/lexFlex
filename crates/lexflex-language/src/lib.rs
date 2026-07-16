#![forbid(unsafe_code)]

mod id;
mod index;
mod loader;
mod model;
mod validation;

pub use id::{
    FeatureName, FeatureValue, FormId, LexemeId, LexicalSenseId, MeaningTemplateId, ParadigmId,
    SurfaceRelationId, ValencySlotId,
};
pub use index::{FormIndex, SenseIndex};
pub use lexflex_model::LanguageId;
pub use loader::{
    CompiledLexicalSense, LanguageLoadError, LanguageModel, LanguagePackageLoader,
    LanguagePackageManifest,
};
pub use model::{
    AtomicCategoryKind, CategoryType, FeatureConflict, FeatureStructure, Form, Lexeme,
    LexicalSense, MeaningTemplate, MorphologyParadigm, SemanticAnchor, SlashDirection,
    SyntacticCategory, ValencySlot,
};
pub use validation::{LanguageModelValidator, LanguageValidationError, LanguageValidationIssue};
