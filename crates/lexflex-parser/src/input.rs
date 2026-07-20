use lexflex_language::LanguageId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseInput {
    pub source_id: String,
    pub language: LanguageId,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClauseMode {
    Declarative,
    Interrogative,
}
