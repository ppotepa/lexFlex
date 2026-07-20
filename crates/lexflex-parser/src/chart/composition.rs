use super::item::ChartItem;
use crate::category::{apply_backward, apply_forward, CategoryOutcome};
use crate::chart::CompositionReport;
use crate::diagnostic::ParseError;
use crate::explain::{ApplicationRule, DerivationSet};
use crate::meaning::apply_meaning;
use crate::metrics::ParseScore;
use lexflex_model::ConceptCatalog;

pub fn compose_all(
    left: &ChartItem,
    right: &ChartItem,
    catalog: &ConceptCatalog,
    max_semantic_nodes: usize,
    max_derivations_per_item: usize,
) -> Result<CompositionReport, ParseError> {
    let mut report = CompositionReport::default();
    let base = match left
        .substitution
        .merged_for_composition(&right.substitution, catalog)
        .map_err(ParseError::from)?
    {
        CategoryOutcome::Applied(value) => value,
        CategoryOutcome::NotApplicable(_) => {
            report.attempted = 2;
            report.rejected = 2;
            return Ok(report);
        }
    };

    report.attempted += 1;
    match apply_forward(&left.category, &right.category, &base, catalog)
        .map_err(ParseError::from)?
    {
        CategoryOutcome::Applied((category, semantic_parameter, substitution)) => {
            report.items.push(compose_applied(
                left,
                right,
                AppliedComposition {
                    category,
                    semantic_parameter,
                    substitution,
                    direction: ApplicationDirection::Forward,
                    max_semantic_nodes,
                    max_derivations_per_item,
                },
            )?);
        }
        CategoryOutcome::NotApplicable(_) => report.rejected += 1,
    }

    report.attempted += 1;
    match apply_backward(&right.category, &left.category, &base, catalog)
        .map_err(ParseError::from)?
    {
        CategoryOutcome::Applied((category, semantic_parameter, substitution)) => {
            report.items.push(compose_applied(
                left,
                right,
                AppliedComposition {
                    category,
                    semantic_parameter,
                    substitution,
                    direction: ApplicationDirection::Backward,
                    max_semantic_nodes,
                    max_derivations_per_item,
                },
            )?);
        }
        CategoryOutcome::NotApplicable(_) => report.rejected += 1,
    }

    report.items.sort_by(|a, b| a.score.cmp(&b.score));
    Ok(report)
}

#[derive(Debug, Clone, Copy)]
enum ApplicationDirection {
    Forward,
    Backward,
}

struct AppliedComposition {
    category: lexflex_language::SyntacticCategory,
    semantic_parameter: lexflex_model::ParameterId,
    substitution: crate::category::CategorySubstitution,
    direction: ApplicationDirection,
    max_semantic_nodes: usize,
    max_derivations_per_item: usize,
}

fn compose_applied(
    left: &ChartItem,
    right: &ChartItem,
    applied: AppliedComposition,
) -> Result<ChartItem, ParseError> {
    let (function_meaning, argument_meaning, rule) = match applied.direction {
        ApplicationDirection::Forward => (
            &left.meaning,
            &right.meaning,
            ApplicationRule::Forward {
                semantic_parameter: applied.semantic_parameter.clone(),
            },
        ),
        ApplicationDirection::Backward => (
            &right.meaning,
            &left.meaning,
            ApplicationRule::Backward {
                semantic_parameter: applied.semantic_parameter.clone(),
            },
        ),
    };

    let meaning = apply_meaning(
        function_meaning,
        &applied.semantic_parameter,
        argument_meaning,
        applied.max_semantic_nodes,
    )?;
    let unresolved_types = applied
        .substitution
        .unresolved_query_type_count(&meaning.query_variables)?;
    let derivations = DerivationSet::composed(
        rule,
        &left.derivations,
        &right.derivations,
        applied.max_derivations_per_item,
    )?;

    ChartItem::new(
        left.start,
        right.end,
        applied.category,
        applied.substitution,
        meaning,
        ParseScore::composed(left.score, right.score, unresolved_types),
        derivations,
    )
}
