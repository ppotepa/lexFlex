use crate::compiler::{VerifiedCompiledEntry, VerifiedStandaloneProgram};
use crate::runtime::{
    ExecutionBudget, ExecutionPolicy, RuntimeEnvironment, RuntimeError, TraceOperation,
};
use crate::runtime::{ExecutionResult, InterpreterProgram, InterpreterState, TypedExecutionResult};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct LinguaInterpreter {
    budget: ExecutionBudget,
    policy: ExecutionPolicy,
}

impl LinguaInterpreter {
    pub fn execute_entry(
        &self,
        entry: &VerifiedCompiledEntry,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, RuntimeError> {
        let program = InterpreterProgram {
            concepts: entry.model().concepts(),
            functions: entry.model().functions(),
            entry: entry.entry(),
            entry_type: entry.entry_type(),
        };
        self.execute_view(&program, policy)
    }
    pub fn new(budget: ExecutionBudget) -> Self {
        Self {
            budget,
            policy: ExecutionPolicy::default(),
        }
    }

    pub fn with_policy(budget: ExecutionBudget, policy: ExecutionPolicy) -> Self {
        Self { budget, policy }
    }

    pub fn execute_program(
        &self,
        program: &VerifiedStandaloneProgram,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, RuntimeError> {
        let view = InterpreterProgram {
            concepts: program.concepts(),
            functions: program.functions(),
            entry: program.entry(),
            entry_type: program.entry_type(),
        };
        self.execute_view(&view, policy)
    }

    pub fn execute(
        &self,
        program: &VerifiedStandaloneProgram,
    ) -> Result<TypedExecutionResult, RuntimeError> {
        self.execute_program(program, self.policy.clone())
    }

    fn execute_view(
        &self,
        program: &InterpreterProgram<'_>,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, RuntimeError> {
        let mut state = InterpreterState::new(program, self.budget.clone(), policy);
        let environment = RuntimeEnvironment::default();
        let value = state.eval(program.entry, &environment)?;
        let semantic = value.into_semantic()?;
        let normalized_output = crate::normalize::SemanticNormalizer
            .normalize(semantic)
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
