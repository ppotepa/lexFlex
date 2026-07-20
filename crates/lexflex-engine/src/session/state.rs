use crate::knowledge::KnowledgeSnapshot;
use crate::session::{SessionIntegrityError, ENGINE_SESSION_SCHEMA};
use lexflex_model::{CanonicalDigest, ConceptCatalog};
use lexflex_store::{validate_session_id, SessionId};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngineSessionState {
    schema: u32,
    session_id: String,
    knowledge: KnowledgeSnapshot,
    model_hash: CanonicalDigest,
    language_hash: CanonicalDigest,
}

#[derive(Deserialize)]
struct RawEngineSessionState {
    schema: u32,
    session_id: String,
    knowledge: crate::knowledge::KnowledgeSnapshot,
    model_hash: CanonicalDigest,
    language_hash: CanonicalDigest,
}

impl<'de> Deserialize<'de> for EngineSessionState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawEngineSessionState::deserialize(deserializer)?;
        if raw.schema != ENGINE_SESSION_SCHEMA {
            return Err(serde::de::Error::custom(format!(
                "session schema mismatch: stored={}, expected={}",
                raw.schema, ENGINE_SESSION_SCHEMA
            )));
        }
        SessionId::new(raw.session_id.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            schema: raw.schema,
            session_id: raw.session_id,
            knowledge: raw.knowledge,
            model_hash: raw.model_hash,
            language_hash: raw.language_hash,
        })
    }
}

impl EngineSessionState {
    pub fn new(
        session_id: impl Into<String>,
        model_hash: CanonicalDigest,
        language_hash: CanonicalDigest,
    ) -> Result<Self, SessionStateError> {
        let session_id = session_id.into();
        let session_id = SessionId::new(session_id).map_err(SessionStateError::SessionId)?;
        let session_id = session_id.as_str().to_owned();
        Ok(Self {
            schema: ENGINE_SESSION_SCHEMA,
            session_id,
            knowledge: KnowledgeSnapshot::new()?,
            model_hash,
            language_hash,
        })
    }

    pub fn evidence_count(&self) -> usize {
        self.knowledge.evidence_count()
    }

    pub fn knowledge(&self) -> &KnowledgeSnapshot {
        &self.knowledge
    }

    pub(crate) fn knowledge_mut(&mut self) -> &mut KnowledgeSnapshot {
        &mut self.knowledge
    }

    pub fn model_hash(&self) -> &CanonicalDigest {
        &self.model_hash
    }

    pub fn language_hash(&self) -> &CanonicalDigest {
        &self.language_hash
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn snapshot_hash(&self) -> &str {
        self.knowledge.snapshot_hash().as_str()
    }

    pub fn verify(
        &self,
        current_model_hash: &CanonicalDigest,
        current_language_hash: &CanonicalDigest,
        catalog: &ConceptCatalog,
    ) -> Result<(), SessionIntegrityError> {
        if self.schema != ENGINE_SESSION_SCHEMA {
            return Err(SessionIntegrityError::SchemaMismatch {
                stored: self.schema,
                expected: ENGINE_SESSION_SCHEMA,
            });
        }

        validate_session_id(&self.session_id).map_err(|_| {
            SessionIntegrityError::InvalidSessionId {
                value: self.session_id.clone(),
            }
        })?;

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

        self.knowledge
            .verify(catalog)
            .map_err(SessionIntegrityError::Knowledge)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionStateError {
    #[error("knowledge snapshot error: {0}")]
    Knowledge(#[from] crate::knowledge::snapshot::KnowledgeSnapshotError),
    #[error("invalid session id: {0}")]
    SessionId(#[from] lexflex_store::SessionStoreError),
}
