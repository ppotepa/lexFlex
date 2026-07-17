use crate::budget::ParseBudget;
use crate::parser::context::ParseContext;
use crate::chart::{compose_all, Chart, ChartItem, InsertOutcome};
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::explain::DerivationNode;
use crate::metrics::ParseMetrics;

pub fn chart_stage(
    ctx: &mut ParseContext<'_>,
    candidates: &[Vec<crate::lexical::Candidate>],
) -> Result<Chart, ParseError> {
    let token_count = candidates.len();
    let mut chart = Chart::new();
    let mut total_items = 0usize;

    for (index, bucket) in candidates.iter().enumerate() {
        let cell = chart.cell_mut(index, index + 1);
        for candidate in bucket {
            let outcome = crate::chart::insert_item(
                cell,
                ChartItem::new(
                    index,
                    index + 1,
                    candidate.category.clone(),
                    Default::default(),
                    candidate.meaning.clone(),
                    candidate.score,
                    DerivationNode::Lexical {
                        token: candidate.token.clone(),
                        sense: candidate.sense.id.clone(),
                    },
                ),
                ctx.budget.max_items_per_cell,
                ctx.budget.max_alternative_derivations_per_item,
            )?;
            record_insert_outcome(outcome, &mut total_items, &mut ctx.metrics, ctx.budget)?;
        }
        ctx.metrics.max_cell_size = ctx.metrics.max_cell_size.max(cell.len());
    }

    for span in 2..=token_count {
        for start in 0..=token_count - span {
            let end = start + span;
            let mut cell_items = Vec::new();
            for split in start + 1..end {
                let left_items = chart
                    .cell(start, split)
                    .map(|cell| cell.values().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let right_items = chart
                    .cell(split, end)
                    .map(|cell| cell.values().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                for left in &left_items {
                    for right in &right_items {
                        let items = compose_all(
                            left,
                            right,
                            ctx.catalog.as_ref(),
                            ctx.budget.max_semantic_nodes,
                        )?;
                        if items.is_empty() {
                            ctx.metrics.rejected_application_count += 1;
                        }
                        for item in items {
                            if item.derivations.max_depth() > ctx.budget.max_derivation_depth {
                                return Err(ParseError::BudgetExceeded(
                                    ParseBudgetLimit::DerivationDepthLimit,
                                ));
                            }
                            ctx.metrics.max_derivation_depth =
                                ctx.metrics.max_derivation_depth.max(item.derivations.max_depth());
                            ctx.metrics.max_semantic_nodes =
                                ctx.metrics.max_semantic_nodes.max(item.meaning.semantic_nodes);
                            cell_items.push(item);
                        }
                    }
                }
            }
            let cell = chart.cell_mut(start, end);
            for item in cell_items {
let outcome = crate::chart::insert_item(cell, item, ctx.budget.max_items_per_cell, ctx.budget.max_alternative_derivations_per_item)?;
            record_insert_outcome(outcome, &mut total_items, &mut ctx.metrics, ctx.budget)?;
            }
            ctx.metrics.max_cell_size = ctx.metrics.max_cell_size.max(cell.len());
        }
    }
    Ok(chart)
}

fn record_insert_outcome(
    outcome: InsertOutcome,
    total_items: &mut usize,
    metrics: &mut ParseMetrics,
    budget: &ParseBudget,
) -> Result<(), ParseError> {
    match outcome {
        InsertOutcome::Inserted => {
            *total_items += 1;
            metrics.chart_item_count += 1;
            if *total_items > budget.max_total_items {
                return Err(ParseError::BudgetExceeded(ParseBudgetLimit::TotalItemLimit));
            }
        }
        InsertOutcome::ReplacedBetter => {
            metrics.chart_replacement_count += 1;
        }
        InsertOutcome::AddedEquivalentDerivations { added } => {
            metrics.alternative_derivation_count += added;
        }
        InsertOutcome::IgnoredWorse | InsertOutcome::IgnoredDuplicateDerivation => {
            metrics.semantic_duplicate_count += 1;
        }
    }
    Ok(())
}