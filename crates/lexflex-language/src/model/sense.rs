use crate::{
    FeatureStructure, LexemeId, LexicalSenseId, MeaningTemplate, SemanticAnchor,
    SyntacticCategory, ValencySlot,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalSense {
    pub id: LexicalSenseId,
    pub lexeme_id: LexemeId,
    #[serde(default)]
    pub anchor: Option<SemanticAnchor>,
    pub base_category: SyntacticCategory,
    pub meaning: MeaningTemplate,
    #[serde(default)]
    pub features: FeatureStructure,
    #[serde(default)]
    pub valency: Vec<ValencySlot>,
    #[serde(default)]
    pub priority: i32,
}
