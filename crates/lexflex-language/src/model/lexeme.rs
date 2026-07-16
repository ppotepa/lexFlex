use crate::LexemeId;
use lexflex_model::LanguageId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lexeme {
    pub id: LexemeId,
    pub language: LanguageId,
    pub lemma: String,
    #[serde(default)]
    pub normalized_lemma: String,
}
