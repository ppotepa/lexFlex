use crate::document::hash::sha256_bytes;
use crate::document::id::DocumentId;
use crate::document::resolution::ResolutionId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalDiscourseIdError {
    Empty,
    TooLong { bytes: usize },
    InvalidCharacter { index: usize, character: char },
}

impl fmt::Display for TemporalDiscourseIdError {
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

impl std::error::Error for TemporalDiscourseIdError {}

macro_rules! temporal_discourse_id_type {
    ($name:ident, $max_bytes:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn generated(id: String) -> Self {
                validate_identifier(&id, $max_bytes).expect("generated temporal discourse id must be valid");
                Self(id)
            }

            pub fn new(id: &str) -> Result<Self, TemporalDiscourseIdError> {
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
            type Err = TemporalDiscourseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
    };
}

temporal_discourse_id_type!(DocumentReferenceTimeId, 224);
temporal_discourse_id_type!(TemporalExpressionId, 224);
temporal_discourse_id_type!(EventTemporalAssignmentId, 224);
temporal_discourse_id_type!(TemporalRelationId, 224);
temporal_discourse_id_type!(EventCoreferenceDecisionId, 224);
temporal_discourse_id_type!(EventCoreferenceClusterId, 224);
temporal_discourse_id_type!(DiscourseRelationId, 224);

pub struct DocumentTemporalDiscourseIdFactory;

pub type DocumentTemporalDiscourseId = String;

impl DocumentTemporalDiscourseIdFactory {
    pub fn artifact(
        document_id: &DocumentId,
        graph_id: &crate::document::graph::GraphId,
        resolution_id: Option<&ResolutionId>,
        options_fingerprint: &str,
        graph_sha256: &str,
        resolution_sha256: Option<&str>,
    ) -> String {
        let document_part = short_prefix(document_id.as_str(), 24);
        let graph_part = short_prefix(graph_id.as_str(), 24);
        let resolution_part = short_prefix(resolution_id.map(|id| id.as_str()).unwrap_or("no-resolution"), 24);
        let options_prefix = &options_fingerprint[..options_fingerprint.len().min(12)];
        let graph_prefix = &graph_sha256[..graph_sha256.len().min(12)];
        let resolution_prefix = resolution_sha256
            .map(|value| &value[..value.len().min(12)])
            .unwrap_or("none");
        format!(
            "{}:temporal-discourse:v1:a1:{}:{}:{}:{}:{}",
            document_part,
            graph_part,
            resolution_part,
            options_prefix,
            graph_prefix,
            resolution_prefix
        )
    }

    pub fn reference_time(document_id: &DocumentId) -> DocumentReferenceTimeId {
        DocumentReferenceTimeId::generated(format!("{}:ref-time", document_id.as_str()))
    }

    pub fn temporal_expression(artifact_id: &str, ordinal: usize) -> TemporalExpressionId {
        TemporalExpressionId::generated(format!("{artifact_id}:temporal:{ordinal:05}"))
    }

    pub fn event_assignment(artifact_id: &str, ordinal: usize) -> EventTemporalAssignmentId {
        EventTemporalAssignmentId::generated(format!("{artifact_id}:assignment:{ordinal:05}"))
    }

    pub fn temporal_relation(artifact_id: &str, ordinal: usize) -> TemporalRelationId {
        TemporalRelationId::generated(format!("{artifact_id}:relation:{ordinal:05}"))
    }

    pub fn event_decision(artifact_id: &str, ordinal: usize) -> EventCoreferenceDecisionId {
        EventCoreferenceDecisionId::generated(format!("{artifact_id}:event-decision:{ordinal:05}"))
    }

    pub fn event_cluster(artifact_id: &str, ordinal: usize) -> EventCoreferenceClusterId {
        EventCoreferenceClusterId::generated(format!("{artifact_id}:event-cluster:{ordinal:05}"))
    }

    pub fn discourse_relation(artifact_id: &str, ordinal: usize) -> DiscourseRelationId {
        DiscourseRelationId::generated(format!("{artifact_id}:discourse:{ordinal:05}"))
    }
}

fn validate_identifier(id: &str, max_bytes: usize) -> Result<(), TemporalDiscourseIdError> {
    if id.is_empty() {
        return Err(TemporalDiscourseIdError::Empty);
    }
    if id.len() > max_bytes {
        return Err(TemporalDiscourseIdError::TooLong { bytes: id.len() });
    }
    for (index, ch) in id.chars().enumerate() {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/')) {
            return Err(TemporalDiscourseIdError::InvalidCharacter { index, character: ch });
        }
    }
    Ok(())
}

fn short_prefix(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

pub(crate) fn options_fingerprint_bytes<T: Serialize>(options: &T) -> String {
    let bytes = serde_json::to_vec(options).expect("temporal discourse options serialization must succeed");
    sha256_bytes(&bytes)
}
