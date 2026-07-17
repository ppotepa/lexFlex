use crate::category::CategorySubstitution;
use crate::diagnostic::ParseError;
use crate::explain::DerivationNode;
use crate::metrics::ParseScore;
use crate::meaning::MeaningInstance;
use lexflex_language::SyntacticCategory;
use lexflex_model::{canonical_hash, CanonicalDigest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartItem {
    pub start: usize,
    pub end: usize,
    pub category: SyntacticCategory,
    pub substitution: CategorySubstitution,
    pub meaning: MeaningInstance,
    pub score: ParseScore,
    pub derivation: DerivationNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    Inserted,
    ReplacedBetter,
    AddedEquivalentDerivation,
    #[allow(dead_code)]
    IgnoredDuplicateDerivation,
    IgnoredWorse,
}

#[allow(dead_code)]
impl ChartItem {
    pub fn semantic_key(&self) -> Result<(CanonicalDigest, CanonicalDigest), ParseError> {
        let expr_hash = canonical_hash(&self.meaning.expression)?;
        let query_hash = canonical_hash(&self.meaning.query_variables)?;
        Ok((expr_hash, query_hash))
    }
}