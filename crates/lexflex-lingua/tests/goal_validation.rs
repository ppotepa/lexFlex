mod support;

use lexflex_lingua::{
    validate_goal, EvidencePolicy, GoalValidationError, LinguaGoal, SemanticType,
};
use lexflex_model::{ConceptId, EntityId, SemanticExpression, VariableId};
use std::collections::BTreeMap;
use support::model::kernel_catalog;

fn valid_goal() -> LinguaGoal {
    let answer = VariableId::new_unchecked("answer");
    LinguaGoal {
        expression: SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(answer.clone())),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        variables: BTreeMap::from([(
            answer.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
        projection: vec![answer],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    }
}

#[test]
fn valid_goal_is_accepted() {
    let catalog = kernel_catalog();
    assert_eq!(validate_goal(&valid_goal(), &catalog), Ok(()));
}

#[test]
fn zero_limit_is_rejected() {
    let catalog = kernel_catalog();
    let mut goal = valid_goal();
    goal.limit = Some(0);
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::ZeroLimit)
    );
}

#[test]
fn empty_projection_is_rejected() {
    let catalog = kernel_catalog();
    let mut goal = valid_goal();
    goal.projection.clear();
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::EmptyProjection)
    );
}

#[test]
fn undeclared_projection_is_rejected() {
    let catalog = kernel_catalog();
    let mut goal = valid_goal();
    goal.projection = vec![VariableId::new_unchecked("missing")];
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::ProjectionVariableNotDeclared(
            VariableId::new_unchecked("missing")
        ))
    );
}

#[test]
fn unused_declared_variable_is_rejected() {
    let catalog = kernel_catalog();
    let mut goal = valid_goal();
    goal.variables.insert(
        VariableId::new_unchecked("unused"),
        SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
    );
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::UnusedVariable(
            VariableId::new_unchecked("unused")
        ))
    );
}

#[test]
fn non_boolean_expression_is_rejected() {
    let catalog = kernel_catalog();
    let answer = VariableId::new_unchecked("answer");
    let goal = LinguaGoal {
        expression: SemanticExpression::Variable(answer.clone()),
        variables: BTreeMap::from([(
            answer.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
        projection: vec![answer],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::NonBooleanExpression(
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY"))
        ))
    );
}

#[test]
fn bound_projection_is_rejected() {
    let catalog = kernel_catalog();
    let answer = VariableId::new_unchecked("answer");
    let goal = LinguaGoal {
        expression: SemanticExpression::Exists {
            variable: answer.clone(),
            body: Box::new(SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(answer.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("CITY"),
                    bindings: BTreeMap::new(),
                }),
            }),
        },
        variables: BTreeMap::from([(
            answer.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
        projection: vec![answer.clone()],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };
    assert_eq!(
        validate_goal(&goal, &catalog),
        Err(GoalValidationError::ProjectionVariableBound(answer))
    );
}
