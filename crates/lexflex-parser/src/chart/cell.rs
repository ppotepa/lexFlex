use super::item::ChartItem;
use super::key::ChartItemKey;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ChartCell {
    items: BTreeMap<ChartItemKey, ChartItem>,
}

#[allow(dead_code)]
impl ChartCell {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &ChartItem> {
        self.items.values()
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &ChartItemKey) -> Option<&ChartItem> {
        self.items.get(key)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, key: &ChartItemKey) -> Option<&mut ChartItem> {
        self.items.get_mut(key)
    }

    pub fn insert(
        &mut self,
        key: ChartItemKey,
        item: ChartItem,
        limit: usize,
    ) -> Result<crate::chart::item::InsertOutcome, crate::diagnostic::ParseError> {
        if let Some(existing) = self.items.get_mut(&key) {
            if item.score < existing.score {
                *existing = item;
                return Ok(crate::chart::item::InsertOutcome::ReplacedBetter);
            }
            if item.score == existing.score && item.derivation != existing.derivation {
                return Ok(crate::chart::item::InsertOutcome::AddedEquivalentDerivation);
            }
            return Ok(crate::chart::item::InsertOutcome::IgnoredWorse);
        }
        if self.items.len() >= limit {
            return Err(crate::diagnostic::ParseError::BudgetExceeded(
                crate::diagnostic::ParseBudgetLimit::CellItemLimit,
            ));
        }
        self.items.insert(key, item);
        Ok(crate::chart::item::InsertOutcome::Inserted)
    }
}

impl Default for ChartCell {
    fn default() -> Self {
        Self::new()
    }
}