use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeIdError(pub String);

impl fmt::Display for KnowledgeIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KnowledgeIdError {}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = KnowledgeIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_id(s)?;
                Ok(Self(s.to_string()))
            }
        }

        impl TryFrom<String> for $name {
            type Error = KnowledgeIdError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                validate_id(&value)?;
                Ok(Self(value))
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

fn validate_id(value: &str) -> Result<(), KnowledgeIdError> {
    if value.is_empty() {
        return Err(KnowledgeIdError("id must not be empty".into()));
    }
    if value.len() > 128 {
        return Err(KnowledgeIdError("id too long".into()));
    }
    if !value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')) {
        return Err(KnowledgeIdError("invalid id characters".into()));
    }
    Ok(())
}

id_type!(KnowledgeExtractionId);
id_type!(PropositionOccurrenceId);
id_type!(ClaimId);
id_type!(ValueId);
id_type!(QualifierId);
id_type!(ContradictionSetId);
id_type!(KnowledgeDiagnosticId);

#[derive(Debug, Clone, Default)]
pub struct KnowledgeExtractionIdFactory;

impl KnowledgeExtractionIdFactory {
    pub fn artifact(
        document_id: &crate::document::DocumentId,
        graph_id: &crate::document::graph::GraphId,
        resolution_id: Option<&crate::document::resolution::ResolutionId>,
        temporal_id: Option<&crate::document::temporal_discourse::DocumentTemporalDiscourseId>,
        options: &impl Serialize,
    ) -> KnowledgeExtractionId {
        let fingerprint = options_fingerprint_bytes(options);
        let mut bytes = format!("knowledge|{document_id}|{graph_id}|{fingerprint}|");
        if let Some(resolution_id) = resolution_id {
            bytes.push_str(&resolution_id.to_string());
        }
        bytes.push('|');
        if let Some(temporal_id) = temporal_id {
            bytes.push_str(&temporal_id.to_string());
        }
        KnowledgeExtractionId(bytes.into())
    }

    pub fn proposition(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> PropositionOccurrenceId {
        PropositionOccurrenceId(format!("{artifact_id}:p:{ordinal:06}"))
    }

    pub fn claim(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> ClaimId {
        ClaimId(format!("{artifact_id}:c:{ordinal:06}"))
    }

    pub fn value(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> ValueId {
        ValueId(format!("{artifact_id}:v:{ordinal:06}"))
    }

    pub fn qualifier(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> QualifierId {
        QualifierId(format!("{artifact_id}:q:{ordinal:06}"))
    }

    pub fn contradiction_set(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> ContradictionSetId {
        ContradictionSetId(format!("{artifact_id}:x:{ordinal:06}"))
    }

    pub fn diagnostic(artifact_id: &KnowledgeExtractionId, ordinal: usize) -> KnowledgeDiagnosticId {
        KnowledgeDiagnosticId(format!("{artifact_id}:d:{ordinal:06}"))
    }
}

pub(crate) fn options_fingerprint_bytes<T: Serialize>(options: &T) -> String {
    let bytes = serde_json::to_vec(options).expect("knowledge options serialization must succeed");
    crate::document::hash::sha256_bytes(&bytes)
}
