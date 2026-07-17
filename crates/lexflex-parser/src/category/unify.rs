use crate::category::features::expected_features_match;
use crate::category::outcome::{CategoryMismatch, CategoryOutcome};
use crate::category::CategorySubstitution;
use lexflex_language::{CategoryType, SyntacticCategory};
use lexflex_model::{ConceptCatalog, TypeRelation};

pub(crate) fn unify_category(
    expected: &SyntacticCategory,
    actual: &SyntacticCategory,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> CategoryOutcome<()> {
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
                return CategoryOutcome::NotApplicable(CategoryMismatch::AtomicKind {
                    expected: expected_kind.clone(),
                    actual: actual_kind.clone(),
                });
            }
            if !expected_features_match(expected_features, actual_features) {
                return CategoryOutcome::NotApplicable(CategoryMismatch::Shape);
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
                return CategoryOutcome::NotApplicable(CategoryMismatch::Direction {
                    expected: *expected_direction,
                    actual: *actual_direction,
                });
            }
            if expected_parameter != actual_parameter {
                return CategoryOutcome::NotApplicable(CategoryMismatch::SemanticParameter {
                    expected: expected_parameter.clone(),
                    actual: actual_parameter.clone(),
                });
            }
            if !expected_features_match(expected_features, actual_features) {
                return CategoryOutcome::NotApplicable(CategoryMismatch::Shape);
            }
            match unify_category(expected_result, actual_result, substitution, catalog) {
                CategoryOutcome::Applied(()) => {}
                other => return other,
            }
            unify_category(expected_argument, actual_argument, substitution, catalog)
        }
        (_, _) => CategoryOutcome::NotApplicable(CategoryMismatch::Shape),
    }
}

pub(crate) fn unify_type(
    expected: &CategoryType,
    actual: &CategoryType,
    substitution: &mut CategorySubstitution,
    catalog: &ConceptCatalog,
) -> CategoryOutcome<()> {
    match (expected, actual) {
        (CategoryType::Concrete(expected), CategoryType::Concrete(actual)) => {
            if TypeRelation::new(catalog).accepts(expected, actual) {
                CategoryOutcome::Applied(())
            } else {
                CategoryOutcome::NotApplicable(CategoryMismatch::SemanticType {
                    expected: expected.clone(),
                    actual: actual.clone(),
                })
            }
        }
        (CategoryType::Variable(left), CategoryType::Variable(right)) => {
            match substitution.alias(left.clone(), right.clone(), catalog) {
                Ok(()) => CategoryOutcome::Applied(()),
                Err(_) => CategoryOutcome::NotApplicable(CategoryMismatch::SubstitutionConflict {
                    existing: CategoryType::Variable(left.clone()),
                    incoming: CategoryType::Variable(right.clone()),
                }),
            }
        }
        (CategoryType::Variable(variable), CategoryType::Concrete(value))
        | (CategoryType::Concrete(value), CategoryType::Variable(variable)) => {
            match substitution.bind_concrete(variable.clone(), value.clone(), catalog) {
                Ok(()) => CategoryOutcome::Applied(()),
                Err(_) => CategoryOutcome::NotApplicable(CategoryMismatch::SubstitutionConflict {
                    existing: CategoryType::Variable(variable.clone()),
                    incoming: CategoryType::Concrete(value.clone()),
                }),
            }
        }
    }
}
