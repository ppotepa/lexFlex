use super::cell::ChartCell;
use super::item::{ChartItem, InsertOutcome};
use super::key::ChartItemKey;
use crate::diagnostic::ParseError;

pub(crate) fn insert_item(
    cell: &mut ChartCell,
    item: ChartItem,
    limit: usize,
) -> Result<InsertOutcome, ParseError> {
    let key = ChartItemKey::create(&item)?;
    cell.insert(key, item, limit)
}