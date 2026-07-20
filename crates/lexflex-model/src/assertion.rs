use crate::{
    canonical_hash, normalize_expression, AssertionId, CanonicalDigest, CanonicalHashError,
    EvidenceError, EvidenceSet, NormalizationError, SemanticExpression, WorldId,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticAssertion {
    id: AssertionId,
    expression: SemanticExpression,
    evidence: EvidenceSet,
    world: WorldId,
    canonical_hash: CanonicalDigest,
}

impl SemanticAssertion {
    pub fn create(
        expression: SemanticExpression,
        evidence: EvidenceSet,
        world: WorldId,
        catalog: &crate::ConceptCatalog,
    ) -> Result<Self, crate::AssertionCatalogError> {
        let assertion = Self::create_structural(expression, evidence, world)
            .map_err(crate::AssertionCatalogError::Integrity)?;
        assertion.verify_with_catalog(catalog)?;
        Ok(assertion)
    }

    pub(crate) fn create_structural(
        expression: SemanticExpression,
        evidence: EvidenceSet,
        world: WorldId,
    ) -> Result<Self, AssertionError> {
        let expression = normalize_expression(expression)?.expression;
        evidence.verify()?;
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
        self.evidence.verify()?;

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

    pub fn verify_with_catalog(
        &self,
        catalog: &crate::ConceptCatalog,
    ) -> Result<(), crate::AssertionCatalogError> {
        crate::assertion_catalog::verify_assertion_with_catalog(self, catalog)
    }

    pub fn merge_evidence(&mut self, evidence: EvidenceSet) -> Result<usize, AssertionError> {
        self.evidence
            .merge(evidence)
            .map_err(AssertionError::Evidence)
    }

    pub fn id(&self) -> &AssertionId {
        &self.id
    }

    pub fn expression(&self) -> &SemanticExpression {
        &self.expression
    }

    pub fn evidence(&self) -> &EvidenceSet {
        &self.evidence
    }

    pub fn world(&self) -> &WorldId {
        &self.world
    }

    pub fn canonical_hash(&self) -> &CanonicalDigest {
        &self.canonical_hash
    }
}

impl<'de> Deserialize<'de> for SemanticAssertion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSemanticAssertion {
            id: AssertionId,
            expression: SemanticExpression,
            evidence: EvidenceSet,
            world: WorldId,
            canonical_hash: CanonicalDigest,
        }

        let raw = RawSemanticAssertion::deserialize(deserializer)?;
        let assertion = Self {
            id: raw.id,
            expression: raw.expression,
            evidence: raw.evidence,
            world: raw.world,
            canonical_hash: raw.canonical_hash,
        };
        assertion.verify().map_err(serde::de::Error::custom)?;
        Ok(assertion)
    }
}

#[derive(Debug, Serialize)]
struct AssertionIdentity<'a> {
    expression: &'a SemanticExpression,
    world: &'a WorldId,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssertionError {
    #[error("assertion normalization failed: {0}")]
    Normalization(#[from] NormalizationError),

    #[error("assertion canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),

    #[error("invalid evidence: {0}")]
    Evidence(#[from] EvidenceError),

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
