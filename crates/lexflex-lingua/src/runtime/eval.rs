use crate::compiler::ResolvedExpression;
use crate::runtime::{
    ClosureParameter, ClosureValue, InterpreterState, RuntimeEnvironment, RuntimeError,
    RuntimeValue, TraceOperation,
};
use lexflex_model::SemanticExpression;
use std::{collections::BTreeMap, sync::Arc};

impl InterpreterState<'_> {
    pub(super) fn semantic_value(
        &mut self,
        value: SemanticExpression,
    ) -> Result<RuntimeValue, RuntimeError> {
        self.state.value_nodes += 1;
        if self.state.value_nodes > self.budget.max_value_nodes {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(RuntimeValue::Semantic(value))
    }

    pub(super) fn eval(
        &mut self,
        expression: &ResolvedExpression,
        environment: &RuntimeEnvironment,
    ) -> Result<RuntimeValue, RuntimeError> {
        self.consume_step()?;
        match expression {
            ResolvedExpression::Concept(concept) => {
                self.semantic_value(SemanticExpression::Concept(concept.clone()))
            }
            ResolvedExpression::Entity(entity) => {
                self.semantic_value(SemanticExpression::Entity(entity.clone()))
            }
            ResolvedExpression::Value(value) => {
                self.semantic_value(SemanticExpression::Value(value.clone()))
            }
            ResolvedExpression::QueryVariable(variable) => {
                self.semantic_value(SemanticExpression::Variable(variable.clone()))
            }
            ResolvedExpression::Function(function_id) => {
                let function = self
                    .program
                    .functions
                    .get(function_id)
                    .ok_or_else(|| RuntimeError::UnknownFunction(function_id.clone()))?;
                let parameters = function
                    .parameters
                    .iter()
                    .map(|parameter| {
                        (
                            parameter.parameter_id.clone(),
                            ClosureParameter {
                                symbol: parameter.symbol.clone(),
                                value_type: parameter.value_type.clone(),
                            },
                        )
                    })
                    .collect();
                self.push_trace(TraceOperation::CreateClosure, "Function", BTreeMap::new())?;
                Ok(RuntimeValue::Closure(ClosureValue {
                    parameters,
                    body: Arc::new(function.body.clone()),
                    captured: Arc::new(RuntimeEnvironment::default()),
                }))
            }
            ResolvedExpression::Local(symbol) => environment
                .get(symbol)
                .cloned()
                .ok_or_else(|| RuntimeError::UnknownLocal(symbol.clone())),
            ResolvedExpression::Lambda { parameters, body } => {
                let parameters = parameters
                    .iter()
                    .map(|parameter| {
                        (
                            parameter.parameter_id.clone(),
                            ClosureParameter {
                                symbol: parameter.symbol.clone(),
                                value_type: parameter.value_type.clone(),
                            },
                        )
                    })
                    .collect();
                self.push_trace(TraceOperation::CreateClosure, "Lambda", BTreeMap::new())?;
                Ok(RuntimeValue::Closure(ClosureValue {
                    parameters,
                    body: Arc::new((**body).clone()),
                    captured: Arc::new(environment.clone()),
                }))
            }
            ResolvedExpression::Call { callee, arguments } => {
                self.eval_call(callee, arguments, environment)
            }
            ResolvedExpression::ApplyConcept { concept, bindings } => {
                self.eval_concept_application(None, concept, bindings, environment)
            }
            ResolvedExpression::Satisfies { subject, concept } => {
                let subject = self.eval(subject, environment)?.into_semantic()?;
                match concept.as_ref() {
                    ResolvedExpression::ApplyConcept { concept, bindings } => {
                        self.eval_concept_application(Some(subject), concept, bindings, environment)
                    }
                    other => {
                        let concept_value = self.eval(other, environment)?.into_semantic()?;
                        self.push_trace(
                            TraceOperation::BuildSatisfies,
                            "Satisfies",
                            BTreeMap::new(),
                        )?;
                        self.semantic_value(SemanticExpression::Satisfies {
                            subject: Box::new(subject),
                            predicate: Box::new(concept_value),
                        })
                    }
                }
            }
            ResolvedExpression::Equals { left, right } => {
                let left = self.eval(left, environment)?.into_semantic()?;
                let right = self.eval(right, environment)?.into_semantic()?;
                self.push_trace(TraceOperation::BuildEquals, "Equals", BTreeMap::new())?;
                self.semantic_value(SemanticExpression::Equals {
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            ResolvedExpression::And(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item, environment)?.into_semantic()?);
                }
                self.semantic_value(SemanticExpression::And(values))
            }
            ResolvedExpression::Or(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item, environment)?.into_semantic()?);
                }
                self.semantic_value(SemanticExpression::Or(values))
            }
            ResolvedExpression::Not(inner) => {
                let inner = self.eval(inner, environment)?.into_semantic()?;
                self.semantic_value(SemanticExpression::Not(Box::new(inner)))
            }
            ResolvedExpression::Exists {
                variable,
                value_type,
                body,
            } => {
                let body = self.eval(body, environment)?.into_semantic()?;
                self.semantic_value(SemanticExpression::Exists {
                    variable: variable.clone(),
                    value_type: value_type.clone(),
                    body: Box::new(body),
                })
            }
            ResolvedExpression::ForAll {
                variable,
                value_type,
                body,
            } => {
                let body = self.eval(body, environment)?.into_semantic()?;
                self.semantic_value(SemanticExpression::ForAll {
                    variable: variable.clone(),
                    value_type: value_type.clone(),
                    body: Box::new(body),
                })
            }
            ResolvedExpression::Let {
                symbol,
                value,
                body,
            } => {
                let value = self.eval(value, environment)?;
                let mut child = environment.clone();
                child.insert(symbol.clone(), value);
                self.track_environment(&child)?;
                self.eval(body, &child)
            }
        }
    }
}
