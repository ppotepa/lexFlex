use lexflex_model::{
    canonical_hash, AssertionError, AssertionId, CanonicalDigest, CanonicalHashError,
    SemanticAssertion,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeSnapshot {
    pub assertions: BTreeMap<AssertionId, SemanticAssertion>,
    pub snapshot_hash: CanonicalDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertOutcome {
    Inserted {
        assertion_id: AssertionId,
    },
    EvidenceMerged {
        assertion_id: AssertionId,
        added: usize,
    },
    Unchanged {
        assertion_id: AssertionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeSnapshotError {
    #[error("assertion key mismatch: key={key}, assertion={assertion}")]
    AssertionKeyMismatch {
        key: AssertionId,
        assertion: AssertionId,
    },

    #[error("assertion integrity failed for {assertion_id}: {source}")]
    Assertion {
        assertion_id: AssertionId,
        source: AssertionError,
    },

    #[error("snapshot canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),

    #[error("snapshot hash mismatch: stored={stored}, expected={expected}")]
    SnapshotHashMismatch {
        stored: CanonicalDigest,
        expected: CanonicalDigest,
    },

    #[error("missing assertion {assertion_id}")]
    MissingAssertion { assertion_id: AssertionId },
}

impl KnowledgeSnapshot {
    pub fn new() -> Result<Self, KnowledgeSnapshotError> {
        let assertions = BTreeMap::new();
        let snapshot_hash = canonical_hash(&assertions)?;
        Ok(Self {
            assertions,
            snapshot_hash,
        })
    }

    pub fn evidence_count(&self) -> usize {
        self.assertions
            .values()
            .map(|assertion| assertion.evidence.len())
            .sum()
    }

    pub fn verify(&self) -> Result<(), KnowledgeSnapshotError> {
        for (key, assertion) in &self.assertions {
            if key != &assertion.id {
                return Err(KnowledgeSnapshotError::AssertionKeyMismatch {
                    key: key.clone(),
                    assertion: assertion.id.clone(),
                });
            }

            assertion
                .verify()
                .map_err(|source| KnowledgeSnapshotError::Assertion {
                    assertion_id: assertion.id.clone(),
                    source,
                })?;
        }

        let expected = canonical_hash(&self.assertions)?;
        if self.snapshot_hash != expected {
            return Err(KnowledgeSnapshotError::SnapshotHashMismatch {
                stored: self.snapshot_hash.clone(),
                expected,
            });
        }
        Ok(())
    }

    pub fn rebuild_hash(&mut self) -> Result<(), KnowledgeSnapshotError> {
        self.snapshot_hash = canonical_hash(&self.assertions)?;
        Ok(())
    }

    pub fn upsert(
        &mut self,
        incoming: SemanticAssertion,
    ) -> Result<UpsertOutcome, KnowledgeSnapshotError> {
        incoming
            .verify()
            .map_err(|source| KnowledgeSnapshotError::Assertion {
                assertion_id: incoming.id.clone(),
                source,
            })?;

        match self.assertions.get_mut(&incoming.id) {
            Some(existing) => {
                let added = existing
                    .merge_evidence(incoming.evidence.into_values())
                    .map_err(|source| KnowledgeSnapshotError::Assertion {
                        assertion_id: existing.id.clone(),
                        source,
                    })?;
                let assertion_id = existing.id.clone();
                self.rebuild_hash()?;
                self.verify()?;
                if added == 0 {
                    Ok(UpsertOutcome::Unchanged { assertion_id })
                } else {
                    Ok(UpsertOutcome::EvidenceMerged {
                        assertion_id,
                        added,
                    })
                }
            }
            None => {
                let assertion_id = incoming.id.clone();
                self.assertions.insert(assertion_id.clone(), incoming);
                self.rebuild_hash()?;
                self.verify()?;
                Ok(UpsertOutcome::Inserted { assertion_id })
            }
        }
    }

    pub fn merge_evidence(
        &mut self,
        assertion_id: &AssertionId,
        evidence: Vec<lexflex_model::Evidence>,
    ) -> Result<usize, KnowledgeSnapshotError> {
        match self.assertions.get_mut(assertion_id) {
            Some(existing) => {
                let added = existing.merge_evidence(evidence).map_err(|source| {
                    KnowledgeSnapshotError::Assertion {
                        assertion_id: existing.id.clone(),
                        source,
                    }
                })?;
                self.rebuild_hash()?;
                self.verify()?;
                Ok(added)
            }
            None => Err(KnowledgeSnapshotError::MissingAssertion {
                assertion_id: assertion_id.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_model::{EntityId, Evidence, SemanticExpression, WorldId};

    #[test]
    fn upsert_merges_evidence_for_same_assertion() {
        let mut snapshot = KnowledgeSnapshot::new().expect("valid snapshot");
        let assertion = SemanticAssertion::create(
            SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            vec![Evidence::create("source:1", None, None).expect("evidence")],
            WorldId::new_unchecked("actual"),
        )
        .expect("assertion");

        let first = snapshot.upsert(assertion.clone()).expect("upsert");
        assert!(matches!(first, UpsertOutcome::Inserted { .. }));

        let second = snapshot
            .upsert(
                SemanticAssertion::create(
                    SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
                    vec![Evidence::create("source:2", None, None).expect("evidence")],
                    WorldId::new_unchecked("actual"),
                )
                .expect("assertion"),
            )
            .expect("upsert");
        assert!(matches!(
            second,
            UpsertOutcome::EvidenceMerged { added, .. } if added == 1
        ));
        assert_eq!(
            snapshot
                .assertions
                .get(&assertion.id)
                .expect("assertion")
                .evidence
                .len(),
            2
        );
    }

    #[test]
    fn merge_evidence_rejects_missing_assertion() {
        let mut snapshot = KnowledgeSnapshot::new().expect("valid snapshot");
        let err = snapshot
            .merge_evidence(
                &AssertionId::new_unchecked("assertion:missing"),
                vec![Evidence::create("source:missing", None, None).expect("evidence")],
            )
            .expect_err("missing assertion must fail");

        assert_eq!(
            err,
            KnowledgeSnapshotError::MissingAssertion {
                assertion_id: AssertionId::new_unchecked("assertion:missing"),
            }
        );
    }
}
