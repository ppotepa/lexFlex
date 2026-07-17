use super::item::ChartItem;
use crate::diagnostic::ParseError;
use lexflex_model::{canonical_hash, CanonicalDigest};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChartItemKey {
    pub category: CanonicalDigest,
    pub meaning: CanonicalDigest,
    pub query_types: CanonicalDigest,
}

impl ChartItemKey {
    pub fn create(item: &ChartItem) -> Result<Self, ParseError> {
        let category = item
            .substitution
            .apply_category(&item.category)?;
        let query_types = item
            .substitution
            .apply_query_category_types(&item.meaning.query_variables)?;

        Ok(Self {
            category: canonical_hash(&category)?,
            meaning: canonical_hash(&item.meaning.expression)?,
            query_types: canonical_hash(&query_types)?,
        })
    }
}