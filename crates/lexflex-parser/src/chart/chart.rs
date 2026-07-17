use super::derivation_set::DerivationInsertOutcome;
use super::item::{ChartItem, InsertOutcome};
use super::key::ChartItemKey;
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanKey {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ChartCell {
    items: BTreeMap<ChartItemKey, ChartItem>,
}

impl ChartCell {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &ChartItem> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn insert(
        &mut self,
        key: ChartItemKey,
        item: ChartItem,
        limit: usize,
        alt_derivation_limit: usize,
    ) -> Result<InsertOutcome, ParseError> {
        if let Some(existing) = self.items.get_mut(&key) {
            if item.score < existing.score {
                *existing = item;
                return Ok(InsertOutcome::ReplacedBetter);
            }
            if item.score == existing.score {
                let primary_digest = lexflex_model::canonical_hash(&item.derivations.primary())?;
                let existing_primary = lexflex_model::canonical_hash(&existing.derivations.primary())?;
                if primary_digest == existing_primary {
                    let mut added = 0usize;
                    for alt in item.derivations.all().skip(1) {
                        if existing.derivations.insert(alt.clone(), alt_derivation_limit)?
                            == DerivationInsertOutcome::Inserted
                        {
                            added += 1;
                        }
                    }
                    if added > 0 {
                        return Ok(InsertOutcome::AddedEquivalentDerivations { added });
                    }
                    return Ok(InsertOutcome::IgnoredDuplicateDerivation);
                }
                match existing.derivations.insert(
                    item.derivations.primary().clone(),
                    alt_derivation_limit,
                )? {
                    DerivationInsertOutcome::Inserted => {
                        return Ok(InsertOutcome::AddedEquivalentDerivations { added: 1 });
                    }
                    DerivationInsertOutcome::LimitExceeded => {
                        return Err(ParseError::BudgetExceeded(
                            ParseBudgetLimit::AlternativeDerivationLimit,
                        ));
                    }
                    DerivationInsertOutcome::Duplicate => {
                        return Ok(InsertOutcome::IgnoredDuplicateDerivation);
                    }
                }
            }
            return Ok(InsertOutcome::IgnoredWorse);
        }
        if self.items.len() >= limit {
            return Err(ParseError::BudgetExceeded(
                ParseBudgetLimit::CellItemLimit,
            ));
        }
        self.items.insert(key, item);
        Ok(InsertOutcome::Inserted)
    }
}

impl Default for ChartCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Chart {
    cells: BTreeMap<SpanKey, ChartCell>,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    pub fn cell(&self, start: usize, end: usize) -> Option<&ChartCell> {
        self.cells.get(&SpanKey { start, end })
    }

    pub fn cell_mut(&mut self, start: usize, end: usize) -> &mut ChartCell {
        self.cells
            .entry(SpanKey { start, end })
            .or_default()
    }


}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}