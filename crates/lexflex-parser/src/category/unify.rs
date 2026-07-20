use crate::category::features::unify_expected_features;
use crate::category::outcome::{CategoryMismatch, CategoryOutcome};
use crate::category::CategorySubstitution;
use lexflex_language::{CategoryType, SyntacticCategory};
use lexflex_model::{ConceptCatalog, TypeRelation};

pub(crate) fn unify_category(
    expected: &SyntacticCategory,
    actual: &SyntacticCategory,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> crate::category::CategoryResult<()> {
    let mut candidate = substitution.clone();
    let outcome = unify_category_inner(expected, actual, &mut candidate, catalog)?;
    if matches!(outcome, CategoryOutcome::Applied(())) {
        *substitution = candidate;
    }
    Ok(outcome)
}

fn unify_category_inner(
    expected: &SyntacticCategory,
    actual: &SyntacticCategory,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> crate::category::CategoryResult<()> {
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
            if expected_kind != actual_kind {
                return Ok(CategoryOutcome::NotApplicable(
                    CategoryMismatch::AtomicKind {
                        expected: expected_kind.clone(),
                        actual: actual_kind.clone(),
                    },
                ));
            }
            if let Err(mismatch) = unify_expected_features(expected_features, actual_features) {
                return Ok(CategoryOutcome::NotApplicable(mismatch));
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
            if expected_direction != actual_direction {
                return Ok(CategoryOutcome::NotApplicable(
                    CategoryMismatch::Direction {
                        expected: *expected_direction,
                        actual: *actual_direction,
                    },
                ));
            }
            if expected_parameter != actual_parameter {
                return Ok(CategoryOutcome::NotApplicable(
                    CategoryMismatch::SemanticParameter {
                        expected: expected_parameter.clone(),
                        actual: actual_parameter.clone(),
                    },
                ));
            }
            if let Err(mismatch) = unify_expected_features(expected_features, actual_features) {
                return Ok(CategoryOutcome::NotApplicable(mismatch));
            }
            match unify_category_inner(expected_result, actual_result, substitution, catalog)? {
                CategoryOutcome::Applied(()) => {}
                other => return Ok(other),
            }
            unify_category_inner(expected_argument, actual_argument, substitution, catalog)
        }
        (_, _) => Ok(CategoryOutcome::NotApplicable(CategoryMismatch::Shape)),
    }
}

pub(crate) fn unify_type(
    expected: &CategoryType,
    actual: &CategoryType,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> crate::category::CategoryResult<()> {
    match (expected, actual) {
        (CategoryType::Concrete(expected), CategoryType::Concrete(actual)) => {
            if TypeRelation::new(catalog).accepts(expected, actual) {
                Ok(CategoryOutcome::Applied(()))
            } else {
                Ok(CategoryOutcome::NotApplicable(
                    CategoryMismatch::SemanticType {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    },
                ))
            }
        }
        (CategoryType::Variable(left), CategoryType::Variable(right)) => {
            substitution.alias(left.clone(), right.clone(), catalog)
        }
        (CategoryType::Variable(variable), CategoryType::Concrete(value))
        | (CategoryType::Concrete(value), CategoryType::Variable(variable)) => {
            substitution.bind_concrete(variable.clone(), value.clone(), catalog)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_language::{FeatureStructure, SlashDirection};
    use lexflex_model::{ConceptId, ParameterId, SemanticType};

    fn variable(name: &str) -> CategoryType {
        CategoryType::Variable(lexflex_language::CategoryTypeVariableId::new_unchecked(
            name,
        ))
    }

    fn np(value: CategoryType) -> SyntacticCategory {
        SyntacticCategory::noun_phrase(value, FeatureStructure::default())
    }

    fn function(result: SyntacticCategory, argument: SyntacticCategory) -> SyntacticCategory {
        SyntacticCategory::Function {
            result: Box::new(result),
            argument: Box::new(argument),
            direction: SlashDirection::Forward,
            semantic_parameter: ParameterId::new_unchecked("scope"),
            features: FeatureStructure::default(),
        }
    }

    #[test]
    fn function_late_mismatch_does_not_mutate_substitution() {
        let catalog = ConceptCatalog::default();
        let mut substitution = CategorySubstitution::default();
        let expected = function(
            np(variable("X")),
            np(CategoryType::Concrete(SemanticType::Boolean)),
        );
        let actual = function(
            np(CategoryType::Concrete(SemanticType::EntityOf(
                ConceptId::new_unchecked("CITY"),
            ))),
            np(CategoryType::Concrete(SemanticType::EntityOf(
                ConceptId::new_unchecked("CITY"),
            ))),
        );

        let result = unify_category(&expected, &actual, &mut substitution, &catalog);

        assert!(matches!(
            result,
            Ok(CategoryOutcome::NotApplicable(
                CategoryMismatch::SemanticType { .. }
            ))
        ));
        assert_eq!(
            substitution
                .resolve_variable(&lexflex_language::CategoryTypeVariableId::new_unchecked(
                    "X"
                ))
                .expect("resolve"),
            variable("X")
        );
    }

    #[test]
    fn successful_unification_commits_substitution() {
        let catalog = ConceptCatalog::default();
        let mut substitution = CategorySubstitution::default();
        let expected = np(variable("X"));
        let actual = np(CategoryType::Concrete(SemanticType::Boolean));

        let result = unify_category(&expected, &actual, &mut substitution, &catalog);

        assert_eq!(result, Ok(CategoryOutcome::Applied(())));
        assert_eq!(
            substitution
                .resolve_variable(&lexflex_language::CategoryTypeVariableId::new_unchecked(
                    "X"
                ))
                .expect("resolve"),
            CategoryType::Concrete(SemanticType::Boolean)
        );
    }
}
