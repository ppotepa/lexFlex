use super::derivation_set::DerivationSet;
use super::item::ChartItem;
use crate::category::CategoryOutcome;
use crate::category::{apply_backward, apply_forward};
use crate::explain::ApplicationRule;
use crate::meaning::apply_meaning;
use crate::metrics::ParseScore;
use lexflex_model::ConceptCatalog;

pub fn compose_all(
    left: &ChartItem,
    right: &ChartItem,
    catalog: &ConceptCatalog,
    max_semantic_nodes: usize,
    max_derivations_per_item: usize,
) -> Vec<ChartItem> {
    let mut output = Vec::new();
    let mut base = left.substitution.clone();
    if base.merge_for_composition(&right.substitution, catalog).is_err() {
        return output;
    }

    apply_forward(&left.category, &right.category, &base, catalog).into_applied(
        |(category, semantic_parameter, substitution)| {
            let meaning = apply_meaning(
                &left.meaning,
                &semantic_parameter,
                &right.meaning,
                max_semantic_nodes,
            ).ok()?;
            let unresolved_types = substitution
                .unresolved_query_type_count(&meaning.query_variables)
                .ok()?;
            let derivations = DerivationSet::composed(
                ApplicationRule::Forward {
                    semantic_parameter: semantic_parameter.clone(),
                },
                &left.derivations,
                &right.derivations,
                max_derivations_per_item,
            ).ok()?;
            Some(ChartItem::new(
                left.start,
                right.end,
                category,
                substitution,
                meaning,
                ParseScore::composed(left.score, right.score, unresolved_types),
                derivations,
            ))
        },
        &mut output,
    );

    apply_backward(&right.category, &left.category, &base, catalog).into_applied(
        |(category, semantic_parameter, substitution)| {
            let meaning = apply_meaning(
                &right.meaning,
                &semantic_parameter,
                &left.meaning,
                max_semantic_nodes,
            ).ok()?;
            let unresolved_types = substitution
                .unresolved_query_type_count(&meaning.query_variables)
                .ok()?;
            let derivations = DerivationSet::composed(
                ApplicationRule::Backward {
                    semantic_parameter: semantic_parameter.clone(),
                },
                &left.derivations,
                &right.derivations,
                max_derivations_per_item,
            ).ok()?;
            Some(ChartItem::new(
                left.start,
                right.end,
                category,
                substitution,
                meaning,
                ParseScore::composed(left.score, right.score, unresolved_types),
                derivations,
            ))
        },
        &mut output,
    );

    output.sort_by(|a, b| a.score.cmp(&b.score));
    output
}

impl<T> CategoryOutcome<T> {
    fn into_applied<U>(self, f: impl FnOnce(T) -> Option<U>, output: &mut Vec<U>) {
        if let CategoryOutcome::Applied(value) = self {
            if let Some(result) = f(value) {
                output.push(result);
            }
        }
    }
}