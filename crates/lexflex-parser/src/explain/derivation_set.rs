use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::explain::{ApplicationRule, DerivationNode};
use lexflex_model::{canonical_hash, CanonicalDigest};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationSet {
    entries: BTreeMap<CanonicalDigest, DerivationNode>,
}

impl DerivationSet {
    pub fn singleton(derivation: DerivationNode) -> Result<Self, ParseError> {
        Self::try_from_iter([derivation])
    }

    pub fn try_from_iter(
        derivations: impl IntoIterator<Item = DerivationNode>,
    ) -> Result<Self, ParseError> {
        let mut entries = BTreeMap::new();

        for derivation in derivations {
            let digest = canonical_hash(&derivation)?;
            entries.entry(digest).or_insert(derivation);
        }

        if entries.is_empty() {
            return Err(ParseError::EmptyDerivationSet);
        }

        Ok(Self { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn primary(&self) -> &DerivationNode {
        match self.entries.first_key_value() {
            Some((_, value)) => value,
            None => unreachable!("validated DerivationSet"),
        }
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

    pub fn verify(&self) -> Result<(), ParseError> {
        if self.entries.is_empty() {
            return Err(ParseError::EmptyDerivationSet);
        }
        for (stored, node) in &self.entries {
            let expected = canonical_hash(node)?;
            if stored != &expected {
                return Err(ParseError::DerivationDigestMismatch {
                    stored: stored.clone(),
                    expected,
                });
            }
        }
        Ok(())
    }

    pub fn try_merge(&mut self, incoming: &Self, limit: usize) -> Result<usize, ParseError> {
        let new_entries = incoming
            .entries
            .iter()
            .filter(|(digest, _)| !self.entries.contains_key(*digest))
            .map(|(digest, node)| (digest.clone(), node.clone()))
            .collect::<Vec<_>>();

        let merged_len =
            self.entries
                .len()
                .checked_add(new_entries.len())
                .ok_or(ParseError::BudgetExceeded(
                    ParseBudgetLimit::DerivationPerItemLimit,
                ))?;

        if merged_len > limit {
            return Err(ParseError::BudgetExceeded(
                ParseBudgetLimit::DerivationPerItemLimit,
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
        let expected_max =
            left.len()
                .checked_mul(right.len())
                .ok_or(ParseError::BudgetExceeded(
                    ParseBudgetLimit::DerivationPerItemLimit,
                ))?;

        if expected_max > limit {
            return Err(ParseError::BudgetExceeded(
                ParseBudgetLimit::DerivationPerItemLimit,
            ));
        }

        let nodes = left
            .iter()
            .flat_map(|left_node| {
                right.iter().map({
                    let rule = rule.clone();
                    move |right_node| DerivationNode::Applied {
                        rule: rule.clone(),
                        left: Box::new(left_node.clone()),
                        right: Box::new(right_node.clone()),
                    }
                })
            })
            .collect::<Vec<_>>();

        Self::try_from_iter(nodes)
    }
}

impl<'de> Deserialize<'de> for DerivationSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = BTreeMap::<CanonicalDigest, DerivationNode>::deserialize(deserializer)?;
        let value = Self { entries };
        value.verify().map_err(serde::de::Error::custom)?;
        Ok(value)
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
        assert_eq!(set.primary(), &dummy_node("a"));
    }

    #[test]
    fn empty_iterator_rejected() {
        let result = DerivationSet::try_from_iter([]);
        assert!(matches!(result, Err(ParseError::EmptyDerivationSet)));
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
        let primary_first = m.primary().clone();
        let primary_second = m.primary().clone();
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
        assert_eq!(m1.iter().collect::<Vec<_>>(), m2.iter().collect::<Vec<_>>());
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
        let left = DerivationSet::try_from_iter([dummy_node("L1"), dummy_node("L2")]).unwrap();
        let right = DerivationSet::try_from_iter([dummy_node("R1"), dummy_node("R2")]).unwrap();
        let result = DerivationSet::composed(dummy_rule(), &left, &right, 10).unwrap();
        assert_eq!(result.len(), 4);
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

    #[test]
    fn deserialize_rejects_empty_set() {
        let source = "()";
        let result = ron::from_str::<DerivationSet>(source);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_wrong_digest_key() {
        let node = dummy_node("a");
        let wrong = lexflex_model::canonical_hash(&dummy_node("b")).expect("digest");
        let source = ron::to_string(&DerivationSet {
            entries: BTreeMap::from([(wrong, node)]),
        })
        .expect("serialize invalid set");

        let result = ron::from_str::<DerivationSet>(&source);

        assert!(result.is_err());
    }
}
