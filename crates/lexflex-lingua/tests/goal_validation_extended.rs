mod support;

use lexflex_lingua::{
    validate_goal, EvidencePolicy, GoalValidationError, LinguaGoal, SemanticType,
};
use lexflex_model::{ConceptId, SemanticExpression, VariableId};
use std::collections::BTreeMap;
use support::model::kernel_catalog;

#[test]
fn duplicate_projection_is_rejected() {
    let answer = VariableId::new_unchecked("answer");
    let goal = LinguaGoal {
        expression: SemanticExpression::Variable(answer.clone()),
        variables: BTreeMap::from([(
            answer.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
        projection: vec![answer.clone(), answer.clone()],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };
    assert!(matches!(
        validate_goal(&goal, &kernel_catalog()),
        Err(GoalValidationError::DuplicateProjection(_))
            | Err(GoalValidationError::NonBooleanExpression(_))
    ));
}
