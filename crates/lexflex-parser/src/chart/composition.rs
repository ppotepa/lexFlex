use super::item::ChartItem;
use crate::category::{apply_backward, apply_forward};
use crate::diagnostic::ParseError;
use crate::explain::{ApplicationRule, DerivationNode};
use crate::meaning::apply_meaning;
use crate::metrics::ParseScore;
use lexflex_model::ConceptCatalog;

pub fn compose_all(
    left: &ChartItem,
    right: &ChartItem,
    catalog: &ConceptCatalog,
    max_semantic_nodes: usize,
) -> Result<Vec<ChartItem>, ParseError> {
    let mut output = Vec::new();
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
        let unresolved_types = substitution.unresolved_query_type_count(&meaning.query_variables)?;
        output.push(ChartItem::new(
            left.start,
            right.end,
            category,
            substitution,
            meaning,
            ParseScore::composed(left.score, right.score, unresolved_types),
            DerivationNode::Applied {
                rule: ApplicationRule::Forward {
                    semantic_parameter: semantic_parameter.clone(),
                },
                left: Box::new(left.derivations.primary().clone()),
                right: Box::new(right.derivations.primary().clone()),
            },
        ));
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
        let unresolved_types = substitution.unresolved_query_type_count(&meaning.query_variables)?;
        output.push(ChartItem::new(
            left.start,
            right.end,
            category,
            substitution,
            meaning,
            ParseScore::composed(left.score, right.score, unresolved_types),
            DerivationNode::Applied {
                rule: ApplicationRule::Backward {
                    semantic_parameter: semantic_parameter.clone(),
                },
                left: Box::new(left.derivations.primary().clone()),
                right: Box::new(right.derivations.primary().clone()),
            },
        ));
    }

    output.sort_by(|a, b| a.score.cmp(&b.score));
    Ok(output)
}
