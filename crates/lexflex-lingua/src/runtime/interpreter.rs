use crate::compiler::CompiledProgram;
use crate::runtime::{
    ExecutionBudget, ExecutionPolicy, RuntimeEnvironment, RuntimeError, TraceOperation,
};
use crate::runtime::{ExecutionResult, InterpreterState, TypedExecutionResult};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct LinguaInterpreter {
    budget: ExecutionBudget,
    policy: ExecutionPolicy,
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

    pub fn execute(&self, program: &CompiledProgram) -> Result<TypedExecutionResult, RuntimeError> {
        self.execute_with_policy(program, self.policy.clone())
    }

    pub fn execute_with_policy(
        &self,
        program: &CompiledProgram,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, RuntimeError> {
        let mut state = InterpreterState::new(program, self.budget.clone(), policy);
        let environment = RuntimeEnvironment::default();
        let value = state.eval(&program.entry, &environment)?;
        let semantic = value.into_semantic()?;
        let normalized_output = crate::normalize::SemanticNormalizer.normalize(semantic)
            .map_err(|error| RuntimeError::Normalization(error.to_string()))?;
        let normalized = normalized_output.expression;
        let report = normalized_output.report;
        if report.changed {
            state.push_trace(TraceOperation::Normalize, "Normalize", BTreeMap::new())?;
        }
        state.push_trace(TraceOperation::Return, "Return", BTreeMap::new())?;
        Ok(TypedExecutionResult {
            execution: ExecutionResult {
                value: normalized,
                trace: state.trace,
                steps: state.state.steps,
            },
            entry_type: program.entry_type.clone(),
        })
    }
}
