use lexflex_model::{
    canonical_hash, AssertionCatalogError, AssertionError, AssertionId, CanonicalDigest,
    CanonicalHashError, ConceptCatalog, SemanticAssertion,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeSnapshot {
    assertions: BTreeMap<AssertionId, SemanticAssertion>,
    snapshot_hash: CanonicalDigest,
}

impl<'de> Deserialize<'de> for KnowledgeSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawKnowledgeSnapshot {
            assertions: BTreeMap<AssertionId, SemanticAssertion>,
            snapshot_hash: CanonicalDigest,
        }

        let raw = RawKnowledgeSnapshot::deserialize(deserializer)?;
        let snapshot = Self {
            assertions: raw.assertions,
            snapshot_hash: raw.snapshot_hash,
        };
        snapshot
            .verify_structural()
            .map_err(serde::de::Error::custom)?;
        Ok(snapshot)
    }
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

    #[error("assertion catalog integrity failed for {assertion_id}: {source}")]
    AssertionCatalog {
        assertion_id: AssertionId,
        source: AssertionCatalogError,
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
            .map(|assertion| assertion.evidence().len())
            .sum()
    }

    pub fn assertions(&self) -> impl Iterator<Item = (&AssertionId, &SemanticAssertion)> {
        self.assertions.iter()
    }

    pub fn get(&self, id: &AssertionId) -> Option<&SemanticAssertion> {
        self.assertions.get(id)
    }

    pub fn len(&self) -> usize {
        self.assertions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assertions.is_empty()
    }

    pub fn snapshot_hash(&self) -> &CanonicalDigest {
        &self.snapshot_hash
    }

    fn verify_structural(&self) -> Result<(), KnowledgeSnapshotError> {
        for (key, assertion) in &self.assertions {
            if key != assertion.id() {
                return Err(KnowledgeSnapshotError::AssertionKeyMismatch {
                    key: key.clone(),
                    assertion: assertion.id().clone(),
                });
            }

            assertion
                .verify()
                .map_err(|source| KnowledgeSnapshotError::Assertion {
                    assertion_id: assertion.id().clone(),
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

    pub fn verify(&self, catalog: &ConceptCatalog) -> Result<(), KnowledgeSnapshotError> {
        self.verify_structural()?;
        for assertion in self.assertions.values() {
            assertion.verify_with_catalog(catalog).map_err(|source| {
                KnowledgeSnapshotError::AssertionCatalog {
                    assertion_id: assertion.id().clone(),
                    source,
                }
            })?;
        }
        Ok(())
    }

    pub(crate) fn rebuild_hash(&mut self) -> Result<(), KnowledgeSnapshotError> {
        self.snapshot_hash = canonical_hash(&self.assertions)?;
        Ok(())
    }

    pub fn upsert(
        &mut self,
        incoming: SemanticAssertion,
        catalog: &ConceptCatalog,
    ) -> Result<UpsertOutcome, KnowledgeSnapshotError> {
        incoming.verify_with_catalog(catalog).map_err(|source| {
            KnowledgeSnapshotError::AssertionCatalog {
                assertion_id: incoming.id().clone(),
                source,
            }
        })?;

        let mut candidate = self.clone();
        let outcome: Result<UpsertOutcome, KnowledgeSnapshotError> =
            match candidate.assertions.get_mut(incoming.id()) {
                Some(existing) => {
                    let added = existing
                        .merge_evidence(incoming.evidence().clone())
                        .map_err(|source| KnowledgeSnapshotError::Assertion {
                            assertion_id: existing.id().clone(),
                            source,
                        })?;
                    let assertion_id = existing.id().clone();
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
                    let assertion_id = incoming.id().clone();
                    candidate.assertions.insert(assertion_id.clone(), incoming);
                    Ok(UpsertOutcome::Inserted { assertion_id })
                }
            };
        let outcome = outcome?;

        candidate.rebuild_hash()?;
        candidate.verify(catalog)?;
        *self = candidate;
        Ok(outcome)
    }

    pub fn merge_evidence(
        &mut self,
        assertion_id: &AssertionId,
        evidence: lexflex_model::EvidenceSet,
        catalog: &ConceptCatalog,
    ) -> Result<usize, KnowledgeSnapshotError> {
        let mut candidate = self.clone();
        let added = match candidate.assertions.get_mut(assertion_id) {
            Some(existing) => existing.merge_evidence(evidence).map_err(|source| {
                KnowledgeSnapshotError::Assertion {
                    assertion_id: existing.id().clone(),
                    source,
                }
            })?,
            None => Err(KnowledgeSnapshotError::MissingAssertion {
                assertion_id: assertion_id.clone(),
            })?,
        };

        candidate.rebuild_hash()?;
        candidate.verify(catalog)?;
        *self = candidate;
        Ok(added)
    }

    pub fn clear(&mut self, catalog: &ConceptCatalog) -> Result<(), KnowledgeSnapshotError> {
        let mut candidate = self.clone();
        candidate.assertions.clear();
        candidate.rebuild_hash()?;
        candidate.verify(catalog)?;
        *self = candidate;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn from_assertions_for_test(
        assertions: BTreeMap<AssertionId, SemanticAssertion>,
    ) -> Result<Self, KnowledgeSnapshotError> {
        let snapshot_hash = canonical_hash(&assertions)?;
        Ok(Self {
            assertions,
            snapshot_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_model::{Evidence, EvidenceSet, SemanticExpression, WorldId};

    #[test]
    fn upsert_merges_evidence_for_same_assertion() {
        let mut snapshot = KnowledgeSnapshot::new().expect("valid snapshot");
        let catalog = lexflex_model::ConceptCatalog::default();
        let expression = SemanticExpression::Equals {
            left: Box::new(SemanticExpression::Value(
                lexflex_model::SemanticValue::Boolean(true),
            )),
            right: Box::new(SemanticExpression::Value(
                lexflex_model::SemanticValue::Boolean(true),
            )),
        };
        let assertion = SemanticAssertion::create(
            expression.clone(),
            EvidenceSet::singleton(Evidence::create("source:1", None, None).expect("evidence"))
                .expect("valid evidence set"),
            WorldId::new_unchecked("actual"),
            &catalog,
        )
        .expect("assertion");

        let first = snapshot
            .upsert(assertion.clone(), &catalog)
            .expect("upsert");
        assert!(matches!(first, UpsertOutcome::Inserted { .. }));

        let second = snapshot
            .upsert(
                SemanticAssertion::create(
                    expression,
                    EvidenceSet::singleton(
                        Evidence::create("source:2", None, None).expect("evidence"),
                    )
                    .expect("valid evidence set"),
                    WorldId::new_unchecked("actual"),
                    &catalog,
                )
                .expect("assertion"),
                &catalog,
            )
            .expect("upsert");
        assert!(matches!(
            second,
            UpsertOutcome::EvidenceMerged { added, .. } if added == 1
        ));
        assert_eq!(
            snapshot
                .assertions
                .get(assertion.id())
                .expect("assertion")
                .evidence()
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
                EvidenceSet::singleton(
                    Evidence::create("source:missing", None, None).expect("evidence"),
                )
                .expect("valid evidence set"),
                &lexflex_model::ConceptCatalog::default(),
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
