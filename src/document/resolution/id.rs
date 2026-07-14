use crate::document::graph::{GraphId, GraphNodeId};
use crate::document::hash::sha256_bytes;
use crate::document::id::DocumentId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionIdError {
    Empty,
    TooLong { bytes: usize },
    InvalidCharacter { index: usize, character: char },
}

impl fmt::Display for ResolutionIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "identifier must not be empty"),
            Self::TooLong { bytes } => write!(f, "identifier exceeds limit: {bytes}"),
            Self::InvalidCharacter { index, character } => {
                write!(f, "invalid identifier character at {index}: {character:?}")
            }
        }
    }
}

impl std::error::Error for ResolutionIdError {}

macro_rules! resolution_id_type {
    ($name:ident, $max_bytes:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn generated(id: String) -> Self {
                validate_identifier(&id, $max_bytes).expect("generated resolution id must be valid");
                Self(id)
            }

            pub fn new(id: &str) -> Result<Self, ResolutionIdError> {
                validate_identifier(id, $max_bytes)?;
                Ok(Self(id.to_string()))
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

        impl FromStr for $name {
            type Err = ResolutionIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
    };
}

resolution_id_type!(ResolutionId, 224);
resolution_id_type!(EntityClusterId, 224);
resolution_id_type!(ResolutionDecisionId, 224);
resolution_id_type!(SyntheticMentionId, 224);
resolution_id_type!(ResolutionDiagnosticId, 224);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionMentionRef {
    Graph(GraphNodeId),
    Synthetic(SyntheticMentionId),
}

impl Serialize for ResolutionMentionRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResolutionMentionRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Some(rest) = value.strip_prefix("graph:") {
            return GraphNodeId::new(rest)
                .map(Self::Graph)
                .map_err(serde::de::Error::custom);
        }
        if let Some(rest) = value.strip_prefix("synthetic:") {
            return SyntheticMentionId::new(rest)
                .map(Self::Synthetic)
                .map_err(serde::de::Error::custom);
        }
        Err(serde::de::Error::custom("invalid resolution mention ref"))
    }
}

impl fmt::Display for ResolutionMentionRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(id) => write!(f, "graph:{id}"),
            Self::Synthetic(id) => write!(f, "synthetic:{id}"),
        }
    }
}

pub struct DocumentEntityResolutionIdFactory;

impl DocumentEntityResolutionIdFactory {
    pub fn resolution(
        document_id: &DocumentId,
        graph_id: &GraphId,
        options_fingerprint: &str,
        graph_sha256: &str,
    ) -> ResolutionId {
        let options_prefix = &options_fingerprint[..options_fingerprint.len().min(12)];
        let graph_prefix = &graph_sha256[..graph_sha256.len().min(12)];
        ResolutionId::generated(format!(
            "{}:resolution:v1:a1:{}:{}:{}",
            document_id.as_str(),
            graph_id.as_str(),
            options_prefix,
            graph_prefix
        ))
    }

    pub fn cluster(resolution_id: &ResolutionId, ordinal: usize) -> EntityClusterId {
        EntityClusterId::generated(format!("{}:cluster:{ordinal:06}", resolution_id.as_str()))
    }

    pub fn decision(resolution_id: &ResolutionId, ordinal: usize) -> ResolutionDecisionId {
        ResolutionDecisionId::generated(format!("{}:decision:{ordinal:06}", resolution_id.as_str()))
    }

    pub fn synthetic(resolution_id: &ResolutionId, ordinal: usize) -> SyntheticMentionId {
        SyntheticMentionId::generated(format!("{}:synthetic:{ordinal:06}", resolution_id.as_str()))
    }

    pub fn diagnostic(resolution_id: &ResolutionId, ordinal: usize) -> ResolutionDiagnosticId {
        ResolutionDiagnosticId::generated(format!("{}:rd:{ordinal:06}", resolution_id.as_str()))
    }
}

fn validate_identifier(id: &str, max_bytes: usize) -> Result<(), ResolutionIdError> {
    if id.is_empty() {
        return Err(ResolutionIdError::Empty);
    }
    if id.len() > max_bytes {
        return Err(ResolutionIdError::TooLong { bytes: id.len() });
    }
    for (index, ch) in id.chars().enumerate() {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/')) {
            return Err(ResolutionIdError::InvalidCharacter { index, character: ch });
        }
    }
    Ok(())
}

pub(crate) fn options_fingerprint_bytes<T: Serialize>(options: &T) -> String {
    let bytes = serde_json::to_vec(options).expect("resolution options serialization must succeed");
    sha256_bytes(&bytes)
}
