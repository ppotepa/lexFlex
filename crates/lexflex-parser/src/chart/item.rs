use crate::category::CategorySubstitution;
use crate::explain::{DerivationNode, DerivationSet};
use crate::meaning::MeaningInstance;
use crate::metrics::ParseScore;
use lexflex_language::SyntacticCategory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartItem {
    pub start: usize,
    pub end: usize,
    pub category: SyntacticCategory,
    pub substitution: CategorySubstitution,
    pub meaning: MeaningInstance,
    pub score: ParseScore,
    pub derivations: DerivationSet,
}

impl ChartItem {
    pub fn new(
        start: usize,
        end: usize,
        category: SyntacticCategory,
        substitution: CategorySubstitution,
        meaning: MeaningInstance,
        score: ParseScore,
        derivations: DerivationSet,
    ) -> Result<Self, crate::diagnostic::ParseError> {
        if derivations.is_empty() {
            return Err(crate::diagnostic::ParseError::EmptyDerivationSet);
        }
        Ok(Self {
            start,
            end,
            category,
            substitution,
            meaning,
            score,
            derivations,
        })
    }

    pub fn lexical(
        start: usize,
        end: usize,
        category: SyntacticCategory,
        substitution: CategorySubstitution,
        meaning: MeaningInstance,
        score: ParseScore,
        derivation: DerivationNode,
    ) -> Result<Self, crate::diagnostic::ParseError> {
        Ok(Self {
            start,
            end,
            category,
            substitution,
            meaning,
            score,
            derivations: DerivationSet::singleton(derivation)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    Inserted {
        derivations: usize,
    },
    ReplacedBetter {
        removed_derivations: usize,
        inserted_derivations: usize,
    },
    AddedEquivalentDerivations {
        added: usize,
    },
    IgnoredDuplicateDerivations {
        duplicate: usize,
    },
    IgnoredWorse,
}
