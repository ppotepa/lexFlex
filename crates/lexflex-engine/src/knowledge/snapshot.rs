use lexflex_model::{canonical_hash, AssertionId, SemanticAssertion};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeSnapshot {
    pub assertions: BTreeMap<AssertionId, SemanticAssertion>,
    pub snapshot_hash: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeSnapshotError {
    MissingAssertion { assertion_id: AssertionId },
}

impl KnowledgeSnapshot {
    pub fn new() -> Self {
        let mut snapshot = Self::default();
        snapshot.rebuild_hash();
        snapshot
    }

    pub fn evidence_count(&self) -> usize {
        self.assertions
            .values()
            .map(|assertion| assertion.evidence.len())
            .sum()
    }

    pub fn rebuild_hash(&mut self) {
        self.snapshot_hash = canonical_hash(&self.assertions);
    }

    pub fn upsert(&mut self, incoming: SemanticAssertion) -> UpsertOutcome {
        match self.assertions.get_mut(&incoming.id) {
            Some(existing) => {
                let added = existing.merge_evidence(incoming.evidence.clone());
                let assertion_id = existing.id.clone();
                self.rebuild_hash();
                if added == 0 {
                    UpsertOutcome::Unchanged { assertion_id }
                } else {
                    UpsertOutcome::EvidenceMerged {
                        assertion_id,
                        added,
                    }
                }
            }
            None => {
                let assertion_id = incoming.id.clone();
                self.assertions.insert(assertion_id.clone(), incoming);
                self.rebuild_hash();
                UpsertOutcome::Inserted { assertion_id }
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
                let added = existing.merge_evidence(evidence);
                self.rebuild_hash();
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
        let mut snapshot = KnowledgeSnapshot::new();
        let assertion = SemanticAssertion::create(
            SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            vec![Evidence::create("source:1", None, None)],
            WorldId::new_unchecked("actual"),
        );

        let first = snapshot.upsert(assertion.clone());
        assert!(matches!(first, UpsertOutcome::Inserted { .. }));

        let second = snapshot.upsert(SemanticAssertion::create(
            SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            vec![Evidence::create("source:2", None, None)],
            WorldId::new_unchecked("actual"),
        ));
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
        let mut snapshot = KnowledgeSnapshot::new();
        let err = snapshot
            .merge_evidence(
                &AssertionId::new_unchecked("assertion:missing"),
                vec![Evidence::create("source:missing", None, None)],
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
