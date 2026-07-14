use crate::document::id::DocumentId;
use crate::document::hash::sha256_bytes;
use crate::document::graph::schema::DocumentGraphSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphIdError {
    Empty,
    TooLong { bytes: usize },
    InvalidCharacter { index: usize, character: char },
}

impl fmt::Display for GraphIdError {
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

impl std::error::Error for GraphIdError {}

macro_rules! graph_id_type {
    ($name:ident, $max_bytes:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn generated(id: String) -> Self {
                validate_identifier(&id, $max_bytes).expect("generated graph id must be valid");
                Self(id)
            }

            pub fn new(id: &str) -> Result<Self, GraphIdError> {
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
            type Err = GraphIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
    };
}

graph_id_type!(GraphId, 160);
graph_id_type!(GraphNodeId, 200);
graph_id_type!(GraphEdgeId, 220);
graph_id_type!(GraphDiagnosticId, 220);

pub struct DocumentGraphIdFactory;

impl DocumentGraphIdFactory {
    pub fn graph(document_id: &DocumentId, schema: DocumentGraphSchema, options_fingerprint: &str) -> GraphId {
        let prefix = &options_fingerprint[..options_fingerprint.len().min(12)];
        GraphId::generated(format!(
            "{}:graph:v{}:a{}:{}",
            document_id.as_str(),
            schema.schema_version,
            schema.algorithm_version,
            prefix
        ))
    }

    pub fn document_root(graph_id: &GraphId) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:document", graph_id.as_str()))
    }

    pub fn block(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:block:{ordinal:05}", graph_id.as_str()))
    }

    pub fn paragraph(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:paragraph:{ordinal:05}", graph_id.as_str()))
    }

    pub fn source_sentence(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:source-sentence:{ordinal:05}", graph_id.as_str()))
    }

    pub fn semantic_sentence(graph_id: &GraphId, source_ordinal: usize, semantic_ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!(
            "{}:n:semantic-sentence:{source_ordinal:05}:{semantic_ordinal:04}",
            graph_id.as_str()
        ))
    }

    pub fn construction(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:construction:{ordinal:06}", graph_id.as_str()))
    }

    pub fn frame(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:frame:{ordinal:06}", graph_id.as_str()))
    }

    pub fn mention(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:mention:{ordinal:06}", graph_id.as_str()))
    }

    pub fn candidate(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:candidate:{ordinal:06}", graph_id.as_str()))
    }

    pub fn event(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:event:{ordinal:06}", graph_id.as_str()))
    }

    pub fn unresolved(graph_id: &GraphId, ordinal: usize) -> GraphNodeId {
        GraphNodeId::generated(format!("{}:n:unresolved:{ordinal:05}", graph_id.as_str()))
    }

    pub fn edge(graph_id: &GraphId, ordinal: usize) -> GraphEdgeId {
        GraphEdgeId::generated(format!("{}:e:{ordinal:07}", graph_id.as_str()))
    }

    pub fn diagnostic(graph_id: &GraphId, ordinal: usize) -> GraphDiagnosticId {
        GraphDiagnosticId::generated(format!("{}:gd:{ordinal:06}", graph_id.as_str()))
    }
}

fn validate_identifier(id: &str, max_bytes: usize) -> Result<(), GraphIdError> {
    if id.is_empty() {
        return Err(GraphIdError::Empty);
    }
    if id.len() > max_bytes {
        return Err(GraphIdError::TooLong { bytes: id.len() });
    }
    for (index, ch) in id.chars().enumerate() {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/')) {
            return Err(GraphIdError::InvalidCharacter { index, character: ch });
        }
    }
    Ok(())
}

pub(crate) fn options_fingerprint_bytes(options: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(options).expect("graph options serialization must succeed");
    sha256_bytes(&bytes)
}
