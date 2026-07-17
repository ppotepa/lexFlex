use super::cell::ChartCell;
use super::item::ChartItem;
use super::key::ChartItemKey;
use crate::category::{apply_backward, apply_forward};
use crate::meaning::apply_meaning;
use crate::diagnostic::ParseError;
use crate::explain::DerivationNode;
use crate::metrics::ParseScore;
use lexflex_model::ConceptCatalog;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanKey {
    pub start: usize,
    pub end: usize,
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

    #[allow(dead_code)]
    pub fn get_items(&self, start: usize, end: usize) -> Vec<ChartItem> {
        self.cells
            .get(&SpanKey { start, end })
            .map(|cell| cell.values().cloned().collect())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn insert_item(
        &mut self,
        span: (usize, usize),
        item: ChartItem,
        limit: usize,
    ) -> Result<crate::chart::item::InsertOutcome, ParseError> {
        let cell = self.cell_mut(span.0, span.1);
        cell.insert(ChartItemKey::create(&item)?, item, limit)
    }

    #[allow(dead_code)]
    pub fn cell_size(&self, start: usize, end: usize) -> usize {
        self.cells
            .get(&SpanKey { start, end })
            .map(|c| c.len())
            .unwrap_or(0)
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compose(
    left: &ChartItem,
    right: &ChartItem,
    catalog: &ConceptCatalog,
    max_semantic_nodes: usize,
) -> Result<Option<ChartItem>, ParseError> {
    let mut base = left.substitution.clone();
    base.merge(&right.substitution, catalog)?;

    if let Some((category, semantic_parameter, substitution)) =
        apply_forward(&left.category, &right.category, &base, catalog)?
    {
        let meaning = apply_meaning(
            &left.meaning,
            &semantic_parameter,
            &right.meaning,
            max_semantic_nodes,
        )?;
        let unresolved_types = meaning.query_variables.len();
        return Ok(Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            substitution,
            meaning,
            score: ParseScore::composed(left.score, right.score, unresolved_types),
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        }));
    }

    if let Some((category, semantic_parameter, substitution)) =
        apply_backward(&right.category, &left.category, &base, catalog)?
    {
        let meaning = apply_meaning(
            &right.meaning,
            &semantic_parameter,
            &left.meaning,
            max_semantic_nodes,
        )?;
        let unresolved_types = meaning.query_variables.len();
        return Ok(Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            substitution,
            meaning,
            score: ParseScore::composed(left.score, right.score, unresolved_types),
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        }));
    }

    Ok(None)
}