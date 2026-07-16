use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdError {
    kind: &'static str,
    value: Option<String>,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(value) => write!(f, "invalid {}: {}", self.kind, value),
            None => write!(f, "empty {}", self.kind),
        }
    }
}

impl std::error::Error for IdError {}

pub fn validate_identifier(kind: &'static str, value: &str) -> Result<(), IdError> {
    if value.is_empty() {
        return Err(IdError { kind, value: None });
    }

    let valid = value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, ':' | '-' | '_' | '.' | '/')
    });

    if valid {
        Ok(())
    } else {
        Err(IdError {
            kind,
            value: Some(value.to_owned()),
        })
    }
}

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                validate_identifier(stringify!($name), &value)?;
                Ok(Self(value))
            }

            pub fn new_unchecked(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new_unchecked(value)
            }
        }
    };
}

string_id!(AssertionId);
string_id!(ConceptId);
string_id!(EntityId);
string_id!(EvidenceId);
string_id!(LanguageId);
string_id!(ModelPackageId);
string_id!(ParameterId);
string_id!(QualifierId);
string_id!(VariableId);
string_id!(WorldId);
