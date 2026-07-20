#![forbid(unsafe_code)]

mod id;
mod index;
mod loader;
mod model;
mod runtime;
mod validation;

pub use id::{
    CategoryTypeVariableId, FeatureName, FeatureValue, FormId, LexemeId, LexicalSenseId,
    MeaningTemplateId, ParadigmId, SurfaceRelationId, ValencySlotId,
};
pub use index::{FormIndex, SenseIndex};
pub use lexflex_model::LanguageId;
pub use loader::{
    LanguageLoadError, LanguageModel, LanguagePackageLoader, LanguagePackageManifest,
    LanguageRealizationModel,
};
pub use model::{
    AtomicCategoryKind, CategoryType, FeatureConflict, FeatureStructure, Form, Lexeme,
    LexicalSense, MeaningTemplate, MorphologyParadigm, SemanticAnchor, SlashDirection,
    SyntacticCategory, ValencySlot,
};
pub use runtime::{compile_lexical_sense, CompiledLexicalSense, LanguageCompileError};
pub use validation::{LanguageModelValidator, LanguageValidationError, LanguageValidationIssue};
