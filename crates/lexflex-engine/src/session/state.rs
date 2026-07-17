use crate::knowledge::KnowledgeSnapshot;
use crate::session::{SessionIntegrityError, SESSION_SCHEMA};
use lexflex_model::CanonicalDigest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSessionState {
    pub schema: u32,
    pub session_id: String,
    pub knowledge: KnowledgeSnapshot,
    pub model_hash: CanonicalDigest,
    pub language_hash: CanonicalDigest,
}

impl EngineSessionState {
    pub fn new(
        session_id: impl Into<String>,
        model_hash: CanonicalDigest,
        language_hash: CanonicalDigest,
    ) -> Result<Self, SessionStateError> {
        Ok(Self {
            schema: SESSION_SCHEMA,
            session_id: session_id.into(),
            knowledge: KnowledgeSnapshot::new()?,
            model_hash,
            language_hash,
        })
    }

    pub fn rebuild_hash(&mut self) -> Result<(), SessionStateError> {
        self.knowledge.rebuild_hash()?;
        Ok(())
    }

    pub fn rebuild_knowledge(&mut self) -> Result<(), SessionStateError> {
        self.rebuild_hash()
    }

    pub fn evidence_count(&self) -> usize {
        self.knowledge.evidence_count()
    }

    pub fn snapshot_hash(&self) -> &str {
        self.knowledge.snapshot_hash.as_str()
    }

    pub fn verify(
        &self,
        current_model_hash: &CanonicalDigest,
        current_language_hash: &CanonicalDigest,
    ) -> Result<(), SessionIntegrityError> {
        if self.schema != SESSION_SCHEMA {
            return Err(SessionIntegrityError::SchemaMismatch {
                stored: self.schema,
                expected: SESSION_SCHEMA,
            });
        }

        if self.session_id.trim().is_empty() {
            return Err(SessionIntegrityError::EmptySessionId);
        }

        if &self.model_hash != current_model_hash {
            return Err(SessionIntegrityError::ModelHashMismatch {
                stored: self.model_hash.clone(),
                current: current_model_hash.clone(),
            });
        }

        if &self.language_hash != current_language_hash {
            return Err(SessionIntegrityError::LanguageHashMismatch {
                stored: self.language_hash.clone(),
                current: current_language_hash.clone(),
            });
        }

        self.knowledge.verify().map_err(SessionIntegrityError::Knowledge)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionStateError {
    #[error("knowledge snapshot error: {0}")]
    Knowledge(#[from] crate::knowledge::snapshot::KnowledgeSnapshotError),
}
