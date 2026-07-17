use crate::{
    canonical_hash, normalize_expression, AssertionId, CanonicalDigest, CanonicalHashError,
    Evidence, EvidenceError, EvidenceId, NormalizationError, SemanticExpression, WorldId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAssertion {
    pub id: AssertionId,
    pub expression: SemanticExpression,
    pub evidence: BTreeMap<EvidenceId, Evidence>,
    pub world: WorldId,
    pub canonical_hash: CanonicalDigest,
}

impl SemanticAssertion {
    pub fn create(
        expression: SemanticExpression,
        evidence: impl IntoIterator<Item = Evidence>,
        world: WorldId,
    ) -> Result<Self, AssertionError> {
        let expression = normalize_expression(expression)?.expression;
        let evidence = collect_evidence(evidence)?;
        let canonical_hash = canonical_hash(&AssertionIdentity {
            expression: &expression,
            world: &world,
        })?;
        let id = AssertionId::new_unchecked(format!("assertion:{}", canonical_hash.as_str()));
        Ok(Self {
            id,
            expression,
            evidence,
            world,
            canonical_hash,
        })
    }

    pub fn verify(&self) -> Result<(), AssertionError> {
        for (key, evidence) in &self.evidence {
            if key != &evidence.id {
                return Err(AssertionError::EvidenceKeyMismatch {
                    key: key.clone(),
                    evidence: evidence.id.clone(),
                });
            }
            evidence.verify()?;
        }

        let normalized = normalize_expression(self.expression.clone())?.expression;
        if normalized != self.expression {
            return Err(AssertionError::ExpressionNotNormalized { normalized });
        }

        let expected_hash = canonical_hash(&AssertionIdentity {
            expression: &self.expression,
            world: &self.world,
        })?;
        if self.canonical_hash != expected_hash {
            return Err(AssertionError::HashMismatch {
                stored: self.canonical_hash.clone(),
                expected: expected_hash,
            });
        }

        let expected_id =
            AssertionId::new_unchecked(format!("assertion:{}", self.canonical_hash.as_str()));
        if self.id != expected_id {
            return Err(AssertionError::IdMismatch {
                stored: self.id.clone(),
                expected: expected_id,
            });
        }

        Ok(())
    }

    pub fn merge_evidence(
        &mut self,
        evidence: impl IntoIterator<Item = Evidence>,
    ) -> Result<usize, AssertionError> {
        let incoming = collect_evidence(evidence)?;
        let before = self.evidence.len();
        for (id, evidence) in incoming {
            match self.evidence.get(&id) {
                None => {
                    self.evidence.insert(id, evidence);
                }
                Some(existing) if existing == &evidence => {}
                Some(_) => return Err(AssertionError::ConflictingEvidence(id)),
            }
        }
        Ok(self.evidence.len() - before)
    }
}

#[derive(Debug, Serialize)]
struct AssertionIdentity<'a> {
    expression: &'a SemanticExpression,
    world: &'a WorldId,
}

fn collect_evidence(
    values: impl IntoIterator<Item = Evidence>,
) -> Result<BTreeMap<EvidenceId, Evidence>, AssertionError> {
    let mut output = BTreeMap::new();
    for evidence in values {
        evidence.verify()?;
        match output.get(&evidence.id) {
            None => {
                output.insert(evidence.id.clone(), evidence);
            }
            Some(existing) if existing == &evidence => {}
            Some(_) => return Err(AssertionError::ConflictingEvidence(evidence.id)),
        }
    }
    Ok(output)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssertionError {
    #[error("assertion normalization failed: {0}")]
    Normalization(#[from] NormalizationError),

    #[error("assertion canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),

    #[error("invalid evidence: {0}")]
    Evidence(#[from] EvidenceError),

    #[error("evidence key mismatch: key={key}, evidence={evidence}")]
    EvidenceKeyMismatch {
        key: EvidenceId,
        evidence: EvidenceId,
    },

    #[error("conflicting evidence with id {0}")]
    ConflictingEvidence(EvidenceId),

    #[error("assertion hash mismatch: stored={stored}, expected={expected}")]
    HashMismatch {
        stored: CanonicalDigest,
        expected: CanonicalDigest,
    },

    #[error("assertion id mismatch: stored={stored}, expected={expected}")]
    IdMismatch {
        stored: AssertionId,
        expected: AssertionId,
    },

    #[error("assertion expression is not normalized")]
    ExpressionNotNormalized { normalized: SemanticExpression },
}
