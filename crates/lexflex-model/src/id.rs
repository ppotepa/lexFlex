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

impl IdError {
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

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
        #[serde(try_from = "String", into = "String")]
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

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
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
