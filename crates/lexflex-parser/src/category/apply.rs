use crate::category::{unify_category, CategorySubstitution};
use crate::diagnostic::ParseError;
use lexflex_language::{CategoryType, SlashDirection, SyntacticCategory};
use lexflex_model::{ConceptCatalog, ParameterId, SemanticType, VariableId};
use std::collections::BTreeMap;

pub(crate) fn apply_forward(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    base: &CategorySubstitution,
    catalog: &ConceptCatalog,
) -> Result<Option<(SyntacticCategory, ParameterId, CategorySubstitution)>, ParseError> {
    apply_directional(function, argument, base, catalog, SlashDirection::Forward)
}

pub(crate) fn apply_backward(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    base: &CategorySubstitution,
    catalog: &ConceptCatalog,
) -> Result<Option<(SyntacticCategory, ParameterId, CategorySubstitution)>, ParseError> {
    apply_directional(function, argument, base, catalog, SlashDirection::Backward)
}

pub(crate) fn resolve_query_variables(
    query_variables: &BTreeMap<VariableId, CategoryType>,
    substitution: &CategorySubstitution,
) -> Result<BTreeMap<VariableId, SemanticType>, ParseError> {
    query_variables
        .iter()
        .map(|(variable, value)| Ok((variable.clone(), substitution.require_concrete(value)?)))
        .collect()
}

fn apply_directional(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    base: &CategorySubstitution,
    catalog: &ConceptCatalog,
    direction: SlashDirection,
) -> Result<Option<(SyntacticCategory, ParameterId, CategorySubstitution)>, ParseError> {
    let SyntacticCategory::Function {
        result,
        argument: expected_argument,
        direction: actual_direction,
        semantic_parameter,
        ..
    } = function
    else {
        return Ok(None);
    };

    if *actual_direction != direction {
        return Ok(None);
    }

    let mut substitution = base.clone();
    if !unify_category(expected_argument, argument, &mut substitution, catalog)? {
        return Ok(None);
    }

    let result = substitution.apply_category(result)?;
    Ok(Some((result, semantic_parameter.clone(), substitution)))
}
