use lexflex_language::LanguageId;
use serde::{Deserialize, Serialize};

use crate::api::input_error::TextInputError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInput {
    pub source_id: String,
    pub language: LanguageId,
    pub text: String,
}

impl TextInput {
    pub fn validate(&self) -> Result<(), TextInputError> {
        if self.source_id.trim().is_empty() {
            return Err(TextInputError::EmptySourceId);
        }
        if self.text.trim().is_empty() {
            return Err(TextInputError::EmptyText);
        }
        Ok(())
    }
}
