use crate::{canonical_hash, AssertionId, Evidence, SemanticExpression, WorldId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAssertion {
    pub id: AssertionId,
    pub expression: SemanticExpression,
    pub evidence: Vec<Evidence>,
    pub world: WorldId,
    pub canonical_hash: String,
}

impl SemanticAssertion {
    pub fn create(
        expression: SemanticExpression,
        mut evidence: Vec<Evidence>,
        world: WorldId,
    ) -> Self {
        evidence.sort();
        let canonical_hash = canonical_hash(&(&expression, &world));
        let id = AssertionId::new_unchecked(format!("assertion:{canonical_hash}"));
        Self {
            id,
            expression,
            evidence,
            world,
            canonical_hash,
        }
    }

    pub fn merge_evidence(&mut self, mut evidence: Vec<Evidence>) -> usize {
        let before = self.evidence.len();
        self.evidence.append(&mut evidence);
        self.evidence.sort();
        self.evidence.dedup();
        self.evidence.len() - before
    }
}
