use crate::diagnostic::ParseError;
use crate::explain::DerivationNode;
use lexflex_model::{canonical_hash, CanonicalDigest};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivationInsertOutcome {
    Inserted,
    Duplicate,
    LimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationSet {
    primary: DerivationNode,
    alternatives: BTreeMap<CanonicalDigest, DerivationNode>,
}

impl DerivationSet {
    pub fn new(primary: DerivationNode) -> Self {
        Self {
            primary,
            alternatives: BTreeMap::new(),
        }
    }

    pub fn primary(&self) -> &DerivationNode {
        &self.primary
    }

    pub fn all(&self) -> impl Iterator<Item = &DerivationNode> {
        std::iter::once(&self.primary).chain(self.alternatives.values())
    }

    pub fn insert(
        &mut self,
        derivation: DerivationNode,
        limit: usize,
    ) -> Result<DerivationInsertOutcome, ParseError> {
        let digest = canonical_hash(&derivation)?;

        if canonical_hash(&self.primary)? == digest || self.alternatives.contains_key(&digest) {
            return Ok(DerivationInsertOutcome::Duplicate);
        }

        if self.alternatives.len() >= limit {
            return Ok(DerivationInsertOutcome::LimitExceeded);
        }

        self.alternatives.insert(digest, derivation);
        Ok(DerivationInsertOutcome::Inserted)
    }

    pub fn max_depth(&self) -> usize {
        self.all().map(|d| d.depth()).max().unwrap_or(0)
    }
}
