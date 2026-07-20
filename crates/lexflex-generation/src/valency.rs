use crate::GenerationError;
use lexflex_language::{LanguageModel, SemanticAnchor};
use lexflex_model::{ConceptId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

pub(crate) struct OrderedBinding<'a> {
    pub(crate) parameter: &'a ParameterId,
    pub(crate) expression: &'a SemanticExpression,
    pub(crate) relation: Option<&'a str>,
    pub(crate) features: Option<&'a lexflex_language::FeatureStructure>,
}

pub(crate) fn validate_apply(
    concept: &ConceptId,
    bindings: &BTreeMap<ParameterId, SemanticExpression>,
    language: &LanguageModel,
) -> Result<(), GenerationError> {
    let Some(sense) = language
        .compiled_senses
        .values()
        .filter(|sense| sense.anchor.as_ref() == Some(&SemanticAnchor::Concept(concept.clone())))
        .min_by_key(|sense| (sense.priority, sense.id.clone()))
    else {
        return Ok(());
    };

    for binding in bindings.keys() {
        if !sense.valency.iter().any(|slot| &slot.parameter == binding) {
            return Err(GenerationError::UnknownValencyBinding(binding.clone()));
        }
    }
    for slot in &sense.valency {
        if slot.required && !bindings.contains_key(&slot.parameter) {
            return Err(GenerationError::MissingValencyBinding(
                slot.parameter.clone(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn ordered_bindings<'a>(
    concept: &ConceptId,
    bindings: &'a BTreeMap<ParameterId, SemanticExpression>,
    language: &'a LanguageModel,
) -> Vec<OrderedBinding<'a>> {
    let Some(sense) = language
        .compiled_senses
        .values()
        .filter(|sense| sense.anchor.as_ref() == Some(&SemanticAnchor::Concept(concept.clone())))
        .min_by_key(|sense| (sense.priority, sense.id.clone()))
    else {
        return bindings
            .iter()
            .map(|(parameter, expression)| OrderedBinding {
                parameter,
                expression,
                relation: None,
                features: None,
            })
            .collect();
    };

    let mut slots = sense.valency.iter().collect::<Vec<_>>();
    if let Some(order) = language.realizations.argument_orders.get(concept.as_str()) {
        slots.sort_by_key(|slot| {
            (
                order
                    .iter()
                    .position(|parameter| parameter == slot.parameter.as_str())
                    .unwrap_or(order.len()),
                slot.application_rank,
                slot.id.clone(),
            )
        });
    } else {
        slots.sort_by_key(|slot| (slot.application_rank, slot.id.clone()));
    }
    slots
        .into_iter()
        .filter_map(|slot| {
            bindings
                .get(&slot.parameter)
                .map(|expression| OrderedBinding {
                    parameter: &slot.parameter,
                    expression,
                    relation: slot
                        .surface_relation
                        .as_ref()
                        .map(|relation| relation.as_str()),
                    features: Some(slot.argument_category.features()),
                })
        })
        .collect()
}
