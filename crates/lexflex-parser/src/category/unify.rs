use crate::category::features::expected_features_match;
use crate::category::CategorySubstitution;
use crate::diagnostic::ParseError;
use lexflex_language::{CategoryType, SyntacticCategory};
use lexflex_model::{ConceptCatalog, TypeRelation};

pub(crate) fn unify_category(
    expected: &SyntacticCategory,
    actual: &SyntacticCategory,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> Result<bool, ParseError> {
    match (expected, actual) {
        (
            SyntacticCategory::Atom {
                kind: expected_kind,
                semantic_type: expected_type,
                features: expected_features,
            },
            SyntacticCategory::Atom {
                kind: actual_kind,
                semantic_type: actual_type,
                features: actual_features,
            },
        ) => {
            if expected_kind != actual_kind
                || !expected_features_match(expected_features, actual_features)
            {
                return Ok(false);
            }
            unify_type(expected_type, actual_type, substitution, catalog)
        }
        (
            SyntacticCategory::Function {
                result: expected_result,
                argument: expected_argument,
                direction: expected_direction,
                semantic_parameter: expected_parameter,
                features: expected_features,
            },
            SyntacticCategory::Function {
                result: actual_result,
                argument: actual_argument,
                direction: actual_direction,
                semantic_parameter: actual_parameter,
                features: actual_features,
            },
        ) => {
            if expected_direction != actual_direction
                || expected_parameter != actual_parameter
                || !expected_features_match(expected_features, actual_features)
            {
                return Ok(false);
            }
            Ok(
                unify_category(expected_result, actual_result, substitution, catalog)?
                    && unify_category(expected_argument, actual_argument, substitution, catalog)?,
            )
        }
        _ => Ok(false),
    }
}

pub(crate) fn unify_type(
    expected: &CategoryType,
    actual: &CategoryType,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> Result<bool, ParseError> {
    match (expected, actual) {
        (CategoryType::Concrete(expected), CategoryType::Concrete(actual)) => {
            Ok(TypeRelation::new(catalog).accepts(expected, actual))
        }
        (CategoryType::Variable(left), CategoryType::Variable(right)) => {
            substitution.alias(left.clone(), right.clone(), catalog)?;
            Ok(true)
        }
        (CategoryType::Variable(variable), CategoryType::Concrete(value))
        | (CategoryType::Concrete(value), CategoryType::Variable(variable)) => {
            match substitution.bind_concrete(variable.clone(), value.clone(), catalog) {
                Ok(()) => Ok(true),
                Err(ParseError::ConflictingQueryCategoryType { .. }) => Ok(false),
                Err(other) => Err(other),
            }
        }
    }
}
