use crate::type_check::value::semantic_value_type;
use crate::{
    ExpressionTypeEnvironment, ExpressionTypeError, SemanticExpression, SemanticType, TypeRelation,
};

#[derive(Debug, Clone)]
pub struct ExpressionTypeBudget {
    pub max_nodes: usize,
    pub max_depth: usize,
}

impl Default for ExpressionTypeBudget {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_depth: 512,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionTypeChecker<'a> {
    relation: TypeRelation<'a>,
    budget: ExpressionTypeBudget,
}

impl<'a> ExpressionTypeChecker<'a> {
    pub fn new(catalog: &'a crate::ConceptCatalog) -> Self {
        Self {
            relation: TypeRelation::new(catalog),
            budget: ExpressionTypeBudget::default(),
        }
    }

    pub fn with_budget(catalog: &'a crate::ConceptCatalog, budget: ExpressionTypeBudget) -> Self {
        Self {
            relation: TypeRelation::new(catalog),
            budget,
        }
    }

    pub fn infer(
        &self,
        expression: &SemanticExpression,
        environment: &mut ExpressionTypeEnvironment<'_>,
    ) -> Result<SemanticType, ExpressionTypeError> {
        self.infer_inner(expression, environment, 0, 0)
    }

    fn infer_inner(
        &self,
        expression: &SemanticExpression,
        environment: &mut ExpressionTypeEnvironment<'_>,
        nodes: usize,
        depth: usize,
    ) -> Result<SemanticType, ExpressionTypeError> {
        if nodes > self.budget.max_nodes {
            return Err(ExpressionTypeError::NodeBudgetExceeded {
                max: self.budget.max_nodes,
            });
        }
        if depth > self.budget.max_depth {
            return Err(ExpressionTypeError::DepthBudgetExceeded {
                max: self.budget.max_depth,
            });
        }

        let next_nodes = nodes + 1;

        match expression {
            SemanticExpression::Concept(concept) => environment
                .catalog
                .concept(concept)
                .map(|schema| schema.result_type.clone())
                .ok_or_else(|| ExpressionTypeError::UnknownConcept(concept.clone())),
            SemanticExpression::Entity(entity) => {
                let definition = environment
                    .catalog
                    .entity(entity)
                    .ok_or_else(|| ExpressionTypeError::UnknownEntity(entity.clone()))?;
                let value_type = SemanticType::EntityOf(definition.primary_type.clone());
                Ok(value_type)
            }
            SemanticExpression::Value(value) => Ok(semantic_value_type(value)),
            SemanticExpression::Variable(variable) => environment
                .variable_type(variable)
                .cloned()
                .ok_or_else(|| ExpressionTypeError::UnknownVariable(variable.clone())),
            SemanticExpression::Apply { concept, bindings } => {
                let schema = environment
                    .catalog
                    .concept(concept)
                    .ok_or_else(|| ExpressionTypeError::UnknownConcept(concept.clone()))?;

                for parameter in schema.parameters.values() {
                    if parameter.required && !bindings.contains_key(&parameter.id) {
                        return Err(ExpressionTypeError::MissingParameter {
                            concept: concept.clone(),
                            parameter: parameter.id.clone(),
                        });
                    }
                }

                for (parameter_id, value) in bindings {
                    let parameter = schema.parameters.get(parameter_id).ok_or_else(|| {
                        ExpressionTypeError::UnknownParameter {
                            concept: concept.clone(),
                            parameter: parameter_id.clone(),
                        }
                    })?;
                    let actual = self.infer_inner(value, environment, next_nodes, depth + 1)?;
                    if !self.relation.accepts(&parameter.value_type, &actual) {
                        return Err(ExpressionTypeError::ParameterTypeMismatch {
                            concept: concept.clone(),
                            parameter: parameter_id.clone(),
                            expected: parameter.value_type.clone(),
                            actual,
                        });
                    }
                }

                Ok(schema.result_type.clone())
            }
            SemanticExpression::Satisfies { subject, predicate } => {
                let subject_type = self.infer_inner(subject, environment, next_nodes, depth + 1)?;
                let predicate_type =
                    self.infer_inner(predicate, environment, next_nodes, depth + 1)?;
                match predicate_type {
                    SemanticType::Predicate(expected_subject) => {
                        if self.relation.accepts(&expected_subject, &subject_type) {
                            Ok(SemanticType::Boolean)
                        } else {
                            Err(ExpressionTypeError::PredicateSubjectMismatch {
                                expected: (*expected_subject).clone(),
                                actual: subject_type,
                            })
                        }
                    }
                    other => Err(ExpressionTypeError::ExpectedPredicate(other)),
                }
            }
            SemanticExpression::Equals { left, right } => {
                let left_type = self.infer_inner(left, environment, next_nodes, depth + 1)?;
                let right_type = self.infer_inner(right, environment, next_nodes, depth + 1)?;
                if self.relation.accepts(&left_type, &right_type)
                    || self.relation.accepts(&right_type, &left_type)
                {
                    Ok(SemanticType::Boolean)
                } else {
                    Err(ExpressionTypeError::EqualityMismatch {
                        left: left_type,
                        right: right_type,
                    })
                }
            }
            SemanticExpression::And(items) | SemanticExpression::Or(items) => {
                for item in items {
                    let value_type = self.infer_inner(item, environment, next_nodes, depth + 1)?;
                    if value_type != SemanticType::Boolean {
                        return Err(ExpressionTypeError::ExpectedBoolean(value_type));
                    }
                }
                Ok(SemanticType::Boolean)
            }
            SemanticExpression::Not(inner) => {
                let value_type = self.infer_inner(inner, environment, next_nodes, depth + 1)?;
                if value_type != SemanticType::Boolean {
                    return Err(ExpressionTypeError::ExpectedBoolean(value_type));
                }
                Ok(SemanticType::Boolean)
            }
            SemanticExpression::Exists {
                variable,
                value_type,
                body,
            }
            | SemanticExpression::ForAll {
                variable,
                value_type,
                body,
            } => {
                environment.push_bound(variable.clone(), value_type.clone());
                let body_result = self.infer_inner(body, environment, next_nodes, depth + 1);
                if !environment.pop_bound() {
                    return Err(ExpressionTypeError::BoundScopeUnderflow);
                }
                let body_type = body_result?;
                if body_type != SemanticType::Boolean {
                    return Err(ExpressionTypeError::ExpectedBoolean(body_type));
                }
                Ok(SemanticType::Boolean)
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => {
                let base_type = self.infer_inner(expression, environment, next_nodes, depth + 1)?;
                for (qualifier, value) in qualifiers {
                    self.infer_inner(value, environment, next_nodes, depth + 1)
                        .map_err(|error| ExpressionTypeError::InvalidQualifier {
                            qualifier: qualifier.clone(),
                            message: error.to_string(),
                        })?;
                }
                Ok(base_type)
            }
        }
    }
}
