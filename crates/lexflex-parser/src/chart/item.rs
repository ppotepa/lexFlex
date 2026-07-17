use super::derivation_set::DerivationSet;
use crate::category::CategorySubstitution;
use crate::explain::DerivationNode;
use crate::metrics::ParseScore;
use crate::meaning::MeaningInstance;
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
    ) -> Self {
        Self {
            start,
            end,
            category,
            substitution,
            meaning,
            score,
            derivations,
        }
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
    Inserted,
    ReplacedBetter,
    AddedEquivalentDerivations { added: usize },
    IgnoredDuplicateDerivation,
    IgnoredWorse,
}