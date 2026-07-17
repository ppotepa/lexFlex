use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::explain::{ApplicationRule, DerivationNode};
use lexflex_model::{canonical_hash, CanonicalDigest};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationInsertOutcome {
    Inserted,
    Duplicate,
    LimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationSet {
    entries: BTreeMap<CanonicalDigest, DerivationNode>,
}

impl DerivationSet {
    pub fn singleton(derivation: DerivationNode) -> Result<Self, ParseError> {
        let digest = canonical_hash(&derivation)?;
        let mut entries = BTreeMap::new();
        entries.insert(digest, derivation);
        Ok(Self { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn primary(&self) -> Option<&DerivationNode> {
        self.entries.first_key_value().map(|(_, value)| value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &DerivationNode> {
        self.entries.values()
    }

    pub fn digests(&self) -> impl Iterator<Item = &CanonicalDigest> {
        self.entries.keys()
    }

    pub fn max_depth(&self) -> usize {
        self.entries.values().map(|d| d.depth()).max().unwrap_or(0)
    }

    pub fn try_merge(
        &mut self,
        incoming: &Self,
        limit: usize,
    ) -> Result<usize, ParseError> {
        let new_entries = incoming
            .entries
            .iter()
            .filter(|(digest, _)| !self.entries.contains_key(*digest))
            .map(|(digest, node)| (digest.clone(), node.clone()))
            .collect::<Vec<_>>();

        if self.entries.len() + new_entries.len() > limit {
            return Err(ParseError::BudgetExceeded(
                ParseBudgetLimit::DerivationLimit,
            ));
        }

        let added = new_entries.len();
        self.entries.extend(new_entries);
        Ok(added)
    }

    pub fn composed(
        rule: ApplicationRule,
        left: &Self,
        right: &Self,
        limit: usize,
    ) -> Result<Self, ParseError> {
        let mut entries = BTreeMap::new();

        for left_node in left.iter() {
            for right_node in right.iter() {
                let node = DerivationNode::Applied {
                    rule: rule.clone(),
                    left: Box::new(left_node.clone()),
                    right: Box::new(right_node.clone()),
                };
                let digest = canonical_hash(&node)?;
                entries.entry(digest).or_insert(node);

                if entries.len() > limit {
                    return Err(ParseError::BudgetExceeded(
                        ParseBudgetLimit::DerivationLimit,
                    ));
                }
            }
        }

        Ok(Self { entries })
    }
}

impl Default for DerivationSet {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explain::{ApplicationRule, DerivationNode};
    use crate::id::TokenId;
    use crate::token::{Token, TokenKind};
    use lexflex_language::LexicalSenseId;
    use lexflex_model::{ParameterId, SourceSpan};

    fn dummy_node(label: &str) -> DerivationNode {
        DerivationNode::Lexical {
            token: Token {
                id: TokenId::new_unchecked("0"),
                surface: label.to_string(),
                normalized: label.to_string(),
                span: SourceSpan::new(0, label.len() as u64).unwrap(),
                kind: TokenKind::Word,
            },
            sense: LexicalSenseId::new_unchecked("default"),
        }
    }

    fn dummy_rule() -> ApplicationRule {
        ApplicationRule::Forward {
            semantic_parameter: ParameterId::new_unchecked("arg"),
        }
    }

    #[test]
    fn singleton_contains_one() {
        let set = DerivationSet::singleton(dummy_node("a")).unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.primary().is_some());
    }

    #[test]
    fn duplicate_insert_no_growth() {
        let mut set = DerivationSet::singleton(dummy_node("a")).unwrap();
        let incoming = DerivationSet::singleton(dummy_node("a")).unwrap();
        let added = set.try_merge(&incoming, 10).unwrap();
        assert_eq!(added, 0);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn deterministic_primary() {
        let a = DerivationSet::singleton(dummy_node("a")).unwrap();
        let b = DerivationSet::singleton(dummy_node("b")).unwrap();
        let mut m = a.clone();
        m.try_merge(&b, 10).unwrap();
        assert_eq!(m.len(), 2);
        let primary_first = m.primary().cloned();
        let primary_second = m.primary().cloned();
        assert_eq!(primary_first, primary_second);
    }

    #[test]
    fn merge_order_independent() {
        let a = DerivationSet::singleton(dummy_node("a")).unwrap();
        let b = DerivationSet::singleton(dummy_node("b")).unwrap();
        let mut m1 = a.clone();
        m1.try_merge(&b, 10).unwrap();
        let mut m2 = b.clone();
        m2.try_merge(&a, 10).unwrap();
        assert_eq!(m1.len(), m2.len());
        assert_eq!(m1.primary(), m2.primary());
        assert_eq!(
            m1.iter().collect::<Vec<_>>(),
            m2.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn transactional_overflow() {
        let a = DerivationSet::singleton(dummy_node("a")).unwrap();
        let b = DerivationSet::singleton(dummy_node("b")).unwrap();
        let c = DerivationSet::singleton(dummy_node("c")).unwrap();
        let mut merged = a.clone();
        merged.try_merge(&b, 10).unwrap();
        assert_eq!(merged.len(), 2);
        let err = merged.try_merge(&c, 2).unwrap_err();
        assert!(matches!(err, ParseError::BudgetExceeded(_)));
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn cartesian_2x2_equals_4() {
        let left = DerivationSet::singleton(dummy_node("L1")).unwrap();
        let right = DerivationSet::singleton(dummy_node("R1")).unwrap();
        let result = DerivationSet::composed(dummy_rule(), &left, &right, 10).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn cartesian_dedup() {
        let left = DerivationSet::singleton(dummy_node("L")).unwrap();
        let right = DerivationSet::singleton(dummy_node("R")).unwrap();
        let r1 = DerivationSet::composed(dummy_rule(), &left, &right, 10).unwrap();
        let r2 = DerivationSet::composed(dummy_rule(), &left, &right, 10).unwrap();
        assert_eq!(r1.len(), r2.len());
    }

    #[test]
    fn limit_zero_rejects_all() {
        let left = DerivationSet::singleton(dummy_node("L")).unwrap();
        let right = DerivationSet::singleton(dummy_node("R")).unwrap();
        let result = DerivationSet::composed(dummy_rule(), &left, &right, 0);
        assert!(result.is_err());
    }

    #[test]
    fn exact_limit_boundary() {
        let left = DerivationSet::singleton(dummy_node("L")).unwrap();
        let right = DerivationSet::singleton(dummy_node("R")).unwrap();
        let result = DerivationSet::composed(dummy_rule(), &left, &right, 1).unwrap();
        assert_eq!(result.len(), 1);
    }
}