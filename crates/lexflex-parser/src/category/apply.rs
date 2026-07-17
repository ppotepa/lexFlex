use crate::category::outcome::{CategoryMismatch, CategoryOutcome};
use crate::category::{unify_category, CategorySubstitution};
use lexflex_language::{CategoryType, SlashDirection, SyntacticCategory};
use lexflex_model::{ConceptCatalog, ParameterId, SemanticType, VariableId};
use std::collections::BTreeMap;

pub(crate) fn apply_forward(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    base: &CategorySubstitution,
    catalog: &ConceptCatalog,
) -> CategoryOutcome<(SyntacticCategory, ParameterId, CategorySubstitution)> {
    apply_directional(function, argument, base, catalog, SlashDirection::Forward)
}

pub(crate) fn apply_backward(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    base: &CategorySubstitution,
    catalog: &ConceptCatalog,
) -> CategoryOutcome<(SyntacticCategory, ParameterId, CategorySubstitution)> {
    apply_directional(function, argument, base, catalog, SlashDirection::Backward)
}

pub(crate) fn resolve_query_variables(
    query_variables: &BTreeMap<VariableId, CategoryType>,
    substitution: &CategorySubstitution,
) -> Result<BTreeMap<VariableId, SemanticType>, crate::diagnostic::ParseError> {
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
) -> CategoryOutcome<(SyntacticCategory, ParameterId, CategorySubstitution)> {
    let SyntacticCategory::Function {
        result,
        argument: expected_argument,
        direction: actual_direction,
        semantic_parameter,
        ..
    } = function
    else {
        return CategoryOutcome::NotApplicable(CategoryMismatch::Shape);
    };

    if *actual_direction != direction {
        return CategoryOutcome::NotApplicable(CategoryMismatch::Direction {
            expected: direction,
            actual: *actual_direction,
        });
    }

    let mut substitution = base.clone();
    match unify_category(expected_argument, argument, &mut substitution, catalog) {
        CategoryOutcome::Applied(()) => {}
        CategoryOutcome::NotApplicable(mismatch) => {
            return CategoryOutcome::NotApplicable(mismatch);
        }
    }

    match substitution.apply_category(result) {
        Ok(result) => CategoryOutcome::Applied((result, semantic_parameter.clone(), substitution)),
        Err(error) => CategoryOutcome::NotApplicable(CategoryMismatch::Shape), // fallback
    }
}
