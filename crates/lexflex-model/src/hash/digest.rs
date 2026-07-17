use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CanonicalDigest(String);

impl CanonicalDigest {
    pub const HEX_LENGTH: usize = 64;

    pub fn new(value: impl Into<String>) -> Result<Self, CanonicalDigestError> {
        let value = value.into();
        if value.len() != Self::HEX_LENGTH {
            return Err(CanonicalDigestError::Length { found: value.len() });
        }
        if !value.chars().all(|character| {
            character.is_ascii_hexdigit() && !character.is_ascii_uppercase()
        })
        {
            return Err(CanonicalDigestError::Alphabet);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CanonicalDigest {
    type Error = CanonicalDigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CanonicalDigest> for String {
    fn from(value: CanonicalDigest) -> Self {
        value.0
    }
}

impl fmt::Display for CanonicalDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CanonicalDigestError {
    #[error("canonical digest must contain 64 characters, found {found}")]
    Length { found: usize },

    #[error("canonical digest must contain lowercase hexadecimal characters")]
    Alphabet,
}
