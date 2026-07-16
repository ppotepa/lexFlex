use lexflex_language::LanguageId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInput {
    pub source_id: String,
    pub language: LanguageId,
    pub text: String,
}
