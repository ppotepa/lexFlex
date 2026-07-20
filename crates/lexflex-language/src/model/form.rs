use crate::{FeatureStructure, FormId, LexemeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Form {
    pub id: FormId,
    pub lexeme_id: LexemeId,
    pub surface: String,
    pub normalized: String,
    #[serde(default)]
    pub features: FeatureStructure,
    #[serde(default)]
    pub priority: i32,
}

impl Form {
    pub fn normalize_surface(surface: &str) -> String {
        surface
            .chars()
            .flat_map(char::to_lowercase)
            .map(|character| match character {
                '’' => '\'',
                other => other,
            })
            .collect()
    }
}
