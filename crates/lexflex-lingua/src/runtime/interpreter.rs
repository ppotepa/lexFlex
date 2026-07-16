use crate::compiler::{
    CompiledConcept, CompiledConceptSemantics, CompiledProgram, ResolvedExpression,
};
use crate::runtime::{
    BudgetState, ClosureParameter, ClosureValue, ExecutionBudget, ExecutionPolicy, ExecutionTrace,
    ExecutionTraceEvent, ExpansionMode, RuntimeEnvironment, RuntimeError, RuntimeValue,
    TraceOperation,
};
use crate::syntax::ExpansionPolicy;
use lexflex_model::{ParameterId, SemanticExpression};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct LinguaInterpreter {
    budget: ExecutionBudget,
    policy: ExecutionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub value: SemanticExpression,
    pub trace: ExecutionTrace,
    pub steps: u64,
}

#[derive(Debug, Clone)]
struct InterpreterState<'a> {
    program: &'a CompiledProgram,
    budget: ExecutionBudget,
    policy: ExecutionPolicy,
    state: BudgetState,
    trace: ExecutionTrace,
    next_sequence: u64,
}

impl LinguaInterpreter {
    pub fn new(budget: ExecutionBudget) -> Self {
        Self {
            budget,
            policy: ExecutionPolicy::default(),
        }
    }

    pub fn with_policy(budget: ExecutionBudget, policy: ExecutionPolicy) -> Self {
        Self { budget, policy }
    }

    pub fn execute(&self, program: &CompiledProgram) -> Result<ExecutionResult, RuntimeError> {
        self.execute_with_policy(program, self.policy.clone())
    }

    pub fn execute_with_policy(
        &self,
        program: &CompiledProgram,
        policy: ExecutionPolicy,
    ) -> Result<ExecutionResult, RuntimeError> {
        let mut state = InterpreterState::new(program, self.budget.clone(), policy);
        let environment = RuntimeEnvironment::default();
        let value = state.eval(&program.entry, &environment)?;
        let semantic = value.into_semantic()?;
        let (normalized, report) = crate::normalize::ExpressionNormalizer.normalize(semantic);
        if report.changed {
            state.push_trace(TraceOperation::Normalize, "Normalize", BTreeMap::new())?;
        }
        state.push_trace(TraceOperation::Return, "Return", BTreeMap::new())?;
        Ok(ExecutionResult {
            value: normalized,
            trace: state.trace,
            steps: state.state.steps,
        })
    }
}

impl<'a> InterpreterState<'a> {
    fn new(program: &'a CompiledProgram, budget: ExecutionBudget, policy: ExecutionPolicy) -> Self {
        Self {
            program,
            budget,
            policy,
            state: BudgetState::default(),
            trace: ExecutionTrace::default(),
            next_sequence: 0,
        }
    }

    fn consume_step(&mut self) -> Result<(), RuntimeError> {
        self.state.steps += 1;
        if self.state.steps > self.budget.max_steps {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(())
    }

    fn enter_call(&mut self) -> Result<(), RuntimeError> {
        self.state.call_depth += 1;
        if self.state.call_depth > self.budget.max_call_depth {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(())
    }

    fn leave_call(&mut self) {
        self.state.call_depth = self.state.call_depth.saturating_sub(1);
    }

    fn push_trace(
        &mut self,
        operation: TraceOperation,
        expression_kind: &'static str,
        details: BTreeMap<String, String>,
    ) -> Result<(), RuntimeError> {
        if self.trace.events.len() >= self.budget.max_trace_events {
            return Err(RuntimeError::BudgetExceeded);
        }
        self.trace.events.push(ExecutionTraceEvent {
            sequence: self.next_sequence,
            operation,
            expression_kind: expression_kind.into(),
            details,
        });
        self.next_sequence += 1;
        Ok(())
    }

    fn semantic_value(&mut self, value: SemanticExpression) -> Result<RuntimeValue, RuntimeError> {
        self.state.value_nodes += 1;
        if self.state.value_nodes > self.budget.max_value_nodes {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(RuntimeValue::Semantic(value))
    }

    fn should_expand_concept(&self, concept: &CompiledConcept) -> bool {
        let has_definition = matches!(concept.semantics, CompiledConceptSemantics::Defined { .. });
        match self.policy.expansion {
            ExpansionMode::PreserveApplications => false,
            ExpansionMode::ExpandTransparent => {
                matches!(concept.expansion, ExpansionPolicy::Transparent) && has_definition
            }
            ExpansionMode::ExpandAllDefined => {
                matches!(
                    concept.expansion,
                    ExpansionPolicy::Transparent | ExpansionPolicy::OnDemand
                ) && has_definition
            }
        }
    }

    fn eval(
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
            ResolvedExpression::Exists { variable, body, .. } => {
                let body = self.eval(body, environment)?.into_semantic()?;
                self.semantic_value(SemanticExpression::Exists {
                    variable: variable.clone(),
                    body: Box::new(body),
                })
            }
            ResolvedExpression::ForAll { variable, body, .. } => {
                let body = self.eval(body, environment)?.into_semantic()?;
                self.semantic_value(SemanticExpression::ForAll {
                    variable: variable.clone(),
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
                self.eval(body, &child)
            }
        }
    }

    fn eval_concept_application(
        &mut self,
        subject: Option<SemanticExpression>,
        concept_id: &lexflex_model::ConceptId,
        bindings: &BTreeMap<ParameterId, ResolvedExpression>,
        environment: &RuntimeEnvironment,
    ) -> Result<RuntimeValue, RuntimeError> {
        let concept = self.program.concepts.get(concept_id);
        let mut normalized = BTreeMap::new();
        for (parameter, argument) in bindings {
            let value = self.eval(argument, environment)?.into_semantic()?;
            normalized.insert(parameter.clone(), value);
        }
        let application = SemanticExpression::Apply {
            concept: concept_id.clone(),
            bindings: normalized,
        };
        let Some(concept) = concept else {
            self.push_trace(
                TraceOperation::ApplyConcept,
                "ApplyConcept",
                BTreeMap::new(),
            )?;
            return self.semantic_value(match subject {
                Some(subject) => SemanticExpression::Satisfies {
                    subject: Box::new(subject),
                    predicate: Box::new(application),
                },
                None => application,
            });
        };
        if !self.should_expand_concept(concept) {
            self.push_trace(
                TraceOperation::ApplyConcept,
                "ApplyConcept",
                BTreeMap::new(),
            )?;
            return self.semantic_value(match subject {
                Some(subject) => SemanticExpression::Satisfies {
                    subject: Box::new(subject),
                    predicate: Box::new(application),
                },
                None => application,
            });
        }
        let CompiledConceptSemantics::Defined { body: definition } = &concept.semantics else {
            return self.semantic_value(match subject {
                Some(subject) => SemanticExpression::Satisfies {
                    subject: Box::new(subject),
                    predicate: Box::new(application),
                },
                None => application,
            });
        };
        let child = self.bind_concept_arguments(concept, bindings, environment, subject)?;
        self.push_trace(
            TraceOperation::ApplyConcept,
            "ApplyConcept",
            BTreeMap::new(),
        )?;
        self.eval(definition, &child)
    }

    fn bind_concept_arguments(
        &mut self,
        concept: &CompiledConcept,
        bindings: &BTreeMap<ParameterId, ResolvedExpression>,
        environment: &RuntimeEnvironment,
        subject: Option<SemanticExpression>,
    ) -> Result<RuntimeEnvironment, RuntimeError> {
        let mut child = environment.clone();
        if let Some(subject) = subject {
            if let Some(self_parameter) = concept.self_parameter.as_ref() {
                child.insert(
                    self_parameter.symbol.clone(),
                    RuntimeValue::Semantic(subject),
                );
            }
        }
        for parameter in &concept.parameters {
            let argument = bindings
                .get(&parameter.parameter_id)
                .ok_or_else(|| RuntimeError::MissingArgument(parameter.parameter_id.clone()))?;
            let value = self.eval(argument, environment)?;
            child.insert(parameter.symbol.clone(), value);
        }
        Ok(child)
    }

    fn eval_call(
        &mut self,
        callee: &ResolvedExpression,
        arguments: &BTreeMap<ParameterId, ResolvedExpression>,
        environment: &RuntimeEnvironment,
    ) -> Result<RuntimeValue, RuntimeError> {
        self.enter_call()?;
        let result = (|| {
            let closure = self.eval(callee, environment)?.into_closure()?;
            let mut child = closure.captured.as_ref().clone();
            let mut remaining = BTreeMap::new();
            for (parameter_id, parameter) in &closure.parameters {
                if let Some(argument) = arguments.get(parameter_id) {
                    let value = self.eval(argument, environment)?;
                    child.insert(parameter.symbol.clone(), value);
                } else {
                    remaining.insert(parameter_id.clone(), parameter.clone());
                }
            }
            self.push_trace(TraceOperation::CallClosure, "Call", BTreeMap::new())?;
            if remaining.is_empty() {
                self.eval(&closure.body, &child)
            } else {
                Ok(RuntimeValue::Closure(ClosureValue {
                    parameters: remaining,
                    body: closure.body.clone(),
                    captured: Arc::new(child),
                }))
            }
        })();
        self.leave_call();
        result
    }
}
