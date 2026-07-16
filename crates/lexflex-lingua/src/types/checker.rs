use crate::compiler::ResolvedExpression;
use crate::types::{SemanticType, TypeEnvironment, TypeError, ValueType};
use lexflex_model::{ConceptId, ParameterId, SemanticValue, TypeRelation};
use std::collections::BTreeMap;

pub struct TypeChecker<'a> {
    environment: &'a TypeEnvironment,
}

impl<'a> TypeChecker<'a> {
    pub fn new(environment: &'a TypeEnvironment) -> Self {
        Self { environment }
    }

    pub fn infer(&self, expression: &ResolvedExpression) -> Result<SemanticType, TypeError> {
        match expression {
            ResolvedExpression::Concept(id) => {
                let schema = self
                    .environment
                    .catalog
                    .concept(id)
                    .ok_or_else(|| TypeError::UnknownConcept(id.clone()))?;
                Ok(SemanticType::ConceptOf(schema.kind))
            }
            ResolvedExpression::Entity(id) => {
                let entity = self
                    .environment
                    .catalog
                    .entity(id)
                    .ok_or_else(|| TypeError::UnknownEntity(id.clone()))?;
                Ok(SemanticType::EntityOf(entity.primary_type.clone()))
            }
            ResolvedExpression::Value(value) => Ok(value_type(value)),
            ResolvedExpression::Local(symbol) => self
                .environment
                .locals
                .get(symbol)
                .cloned()
                .ok_or_else(|| TypeError::UnknownLocal(symbol.clone())),
            ResolvedExpression::QueryVariable(variable) => self
                .environment
                .variables
                .get(variable)
                .cloned()
                .ok_or_else(|| TypeError::UnknownVariable(variable.clone())),
            ResolvedExpression::Function(id) => self
                .environment
                .functions
                .get(id)
                .cloned()
                .map(SemanticType::Function)
                .ok_or_else(|| TypeError::UnknownFunction(id.clone())),
            ResolvedExpression::Lambda { parameters, body } => {
                let mut child = self.environment.clone();
                for parameter in parameters {
                    child
                        .locals
                        .insert(parameter.symbol.clone(), parameter.value_type.clone());
                }
                let checker = TypeChecker::new(&child);
                let body_type = checker.infer(body)?;
                Ok(SemanticType::Function(crate::types::FunctionType {
                    parameters: parameters
                        .iter()
                        .map(|parameter| {
                            (parameter.parameter_id.clone(), parameter.value_type.clone())
                        })
                        .collect(),
                    result: Box::new(body_type),
                }))
            }
            ResolvedExpression::Call { callee, arguments } => {
                let callee_type = self.infer(callee)?;
                let SemanticType::Function(function) = callee_type else {
                    return Err(TypeError::ExpectedFunction { found: callee_type });
                };
                for parameter in function.parameters.keys() {
                    if !arguments.contains_key(parameter) {
                        return Err(TypeError::MissingFunctionArgument(parameter.clone()));
                    }
                }
                for parameter in arguments.keys() {
                    if !function.parameters.contains_key(parameter) {
                        return Err(TypeError::UnknownFunctionArgument(parameter.clone()));
                    }
                }
                for (parameter, argument) in arguments {
                    let expected = function
                        .parameters
                        .get(parameter)
                        .ok_or_else(|| TypeError::UnknownFunctionArgument(parameter.clone()))?;
                    let actual = self.infer(argument)?;
                    if !self.compatible(&actual, expected) {
                        return Err(TypeError::ParameterTypeMismatch {
                            concept: ConceptId::new_unchecked("function"),
                            parameter: parameter.clone(),
                            expected: expected.clone(),
                            actual,
                        });
                    }
                }
                Ok(*function.result)
            }
            ResolvedExpression::ApplyConcept { concept, bindings } => {
                self.infer_apply_concept(concept, bindings)
            }
            ResolvedExpression::Satisfies { subject, concept } => {
                let subject_type = self.infer(subject)?;
                let predicate_type = self.infer(concept)?;
                match predicate_type {
                    SemanticType::Predicate(expected_subject)
                        if self.compatible(&subject_type, &expected_subject) =>
                    {
                        Ok(SemanticType::Boolean)
                    }
                    other => Err(TypeError::ExpectedPredicate { found: other }),
                }
            }
            ResolvedExpression::Equals { left, right } => {
                let left = self.infer(left)?;
                let right = self.infer(right)?;
                if self.compatible(&left, &right) {
                    Ok(SemanticType::Boolean)
                } else {
                    Err(TypeError::EqualityTypeMismatch { left, right })
                }
            }
            ResolvedExpression::And(items) | ResolvedExpression::Or(items) => {
                for item in items {
                    let actual = self.infer(item)?;
                    if actual != SemanticType::Boolean {
                        return Err(TypeError::ExpectedBoolean(actual));
                    }
                }
                Ok(SemanticType::Boolean)
            }
            ResolvedExpression::Not(inner) => {
                let actual = self.infer(inner)?;
                if actual != SemanticType::Boolean {
                    return Err(TypeError::ExpectedBoolean(actual));
                }
                Ok(SemanticType::Boolean)
            }
            ResolvedExpression::Exists { body, .. } | ResolvedExpression::ForAll { body, .. } => {
                let _ = self.infer(body)?;
                Ok(SemanticType::Boolean)
            }
            ResolvedExpression::Let {
                symbol,
                value,
                body,
            } => {
                let value_type = self.infer(value)?;
                let mut child = self.environment.clone();
                child.locals.insert(symbol.clone(), value_type);
                let checker = TypeChecker::new(&child);
                checker.infer(body)
            }
        }
    }

    pub fn compatible(&self, actual: &SemanticType, expected: &SemanticType) -> bool {
        TypeRelation::new(self.environment.catalog.as_ref()).accepts(expected, actual)
    }

    fn infer_apply_concept(
        &self,
        concept: &ConceptId,
        bindings: &BTreeMap<ParameterId, ResolvedExpression>,
    ) -> Result<SemanticType, TypeError> {
        let schema = self
            .environment
            .catalog
            .concept(concept)
            .ok_or_else(|| TypeError::UnknownConcept(concept.clone()))?;

        for (parameter_id, parameter) in &schema.parameters {
            if parameter.required && !bindings.contains_key(parameter_id) {
                return Err(TypeError::MissingParameter {
                    concept: concept.clone(),
                    parameter: parameter_id.clone(),
                });
            }
        }
        for (parameter_id, argument) in bindings {
            let expected =
                schema
                    .parameters
                    .get(parameter_id)
                    .ok_or_else(|| TypeError::UnknownParameter {
                        concept: concept.clone(),
                        parameter: parameter_id.clone(),
                    })?;
            let actual = self.infer(argument)?;
            if !self.compatible(&actual, &expected.value_type) {
                return Err(TypeError::ParameterTypeMismatch {
                    concept: concept.clone(),
                    parameter: parameter_id.clone(),
                    expected: expected.value_type.clone(),
                    actual,
                });
            }
        }
        Ok(schema.result_type.clone())
    }
}

fn value_type(value: &SemanticValue) -> SemanticType {
    match value {
        SemanticValue::Integer(_) => SemanticType::Value(ValueType::Integer),
        SemanticValue::Decimal(_) => SemanticType::Value(ValueType::Decimal),
        SemanticValue::Text(_) => SemanticType::Value(ValueType::Text),
        SemanticValue::Boolean(_) => SemanticType::Value(ValueType::Boolean),
        SemanticValue::Date(_) => SemanticType::Value(ValueType::Date),
        SemanticValue::Quantity(quantity) => SemanticType::Value(ValueType::Quantity {
            dimension: quantity.dimension.clone(),
        }),
    }
}
