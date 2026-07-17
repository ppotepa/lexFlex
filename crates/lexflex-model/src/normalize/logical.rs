use crate::{SemanticExpression, SemanticValue};

use super::metrics::NormalizationReport;

pub(crate) fn collapse_and(
    mut items: Vec<SemanticExpression>,
    report: &mut NormalizationReport,
) -> SemanticExpression {
    items.sort();
    let before = items.len();
    items.dedup();
    report.removed_duplicates += before.saturating_sub(items.len());
    match items.len() {
        0 => SemanticExpression::Value(SemanticValue::Boolean(true)),
        1 => items.remove(0),
        _ => SemanticExpression::And(items),
    }
}

pub(crate) fn collapse_or(
    mut items: Vec<SemanticExpression>,
    report: &mut NormalizationReport,
) -> SemanticExpression {
    items.sort();
    let before = items.len();
    items.dedup();
    report.removed_duplicates += before.saturating_sub(items.len());
    match items.len() {
        0 => SemanticExpression::Value(SemanticValue::Boolean(false)),
        1 => items.remove(0),
        _ => SemanticExpression::Or(items),
    }
}
