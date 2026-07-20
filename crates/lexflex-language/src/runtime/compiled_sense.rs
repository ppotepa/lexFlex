use crate::{
    FeatureStructure, LexemeId, LexicalSenseId, MeaningTemplate, SemanticAnchor, SyntacticCategory,
    ValencySlot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledLexicalSense {
    pub id: LexicalSenseId,
    pub lexeme_id: LexemeId,
    pub anchor: Option<SemanticAnchor>,
    pub category: SyntacticCategory,
    pub meaning: MeaningTemplate,
    pub lexical_features: FeatureStructure,
    pub valency: Vec<ValencySlot>,
    pub priority: i32,
}
