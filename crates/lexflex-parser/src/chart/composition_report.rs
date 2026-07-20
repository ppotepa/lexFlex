use super::item::ChartItem;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompositionReport {
    pub items: Vec<ChartItem>,
    pub attempted: usize,
    pub rejected: usize,
}
