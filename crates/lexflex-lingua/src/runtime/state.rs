use crate::compiler::{CompiledConcept, CompiledConceptSemantics, CompiledProgram};
use crate::runtime::{
    BudgetState, ExecutionBudget, ExecutionPolicy, ExecutionTrace, ExecutionTraceEvent,
    ExpansionMode, RuntimeEnvironment, RuntimeError, TraceOperation,
};
use crate::syntax::ExpansionPolicy;
use lexflex_model::ConceptId;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct InterpreterState<'a> {
    pub(crate) program: &'a CompiledProgram,
    pub(crate) budget: ExecutionBudget,
    pub(crate) policy: ExecutionPolicy,
    pub(crate) state: BudgetState,
    pub(crate) trace: ExecutionTrace,
    pub(crate) next_sequence: u64,
    pub(crate) expansion_stack: Vec<ConceptId>,
}

impl<'a> InterpreterState<'a> {
    pub(super) fn new(
        program: &'a CompiledProgram,
        budget: ExecutionBudget,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            program,
            budget,
            policy,
            state: BudgetState::default(),
            trace: ExecutionTrace::default(),
            next_sequence: 0,
            expansion_stack: Vec::new(),
        }
    }

    pub(super) fn consume_step(&mut self) -> Result<(), RuntimeError> {
        self.state.steps += 1;
        if self.state.steps > self.budget.max_steps {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(())
    }

    pub(super) fn enter_call(&mut self) -> Result<(), RuntimeError> {
        self.state.call_depth += 1;
        if self.state.call_depth > self.budget.max_call_depth {
            return Err(RuntimeError::BudgetExceeded);
        }
        Ok(())
    }

    pub(super) fn leave_call(&mut self) {
        self.state.call_depth = self.state.call_depth.saturating_sub(1);
    }

    pub(super) fn push_trace(
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

    pub(super) fn track_environment(
        &mut self,
        environment: &RuntimeEnvironment,
    ) -> Result<(), RuntimeError> {
        self.state.max_observed_environment_bindings = self
            .state
            .max_observed_environment_bindings
            .max(environment.len());
        if environment.len() > self.budget.max_environment_bindings {
            return Err(RuntimeError::EnvironmentBindingLimitExceeded {
                max: self.budget.max_environment_bindings,
            });
        }
        Ok(())
    }

    pub(super) fn enter_expansion(&mut self, concept: &ConceptId) -> Result<(), RuntimeError> {
        if let Some(index) = self
            .expansion_stack
            .iter()
            .position(|current| current == concept)
        {
            let mut path = self.expansion_stack[index..].to_vec();
            path.push(concept.clone());
            return Err(RuntimeError::ExpansionCycle { path });
        }
        if self.expansion_stack.len() >= self.budget.max_expansion_depth {
            return Err(RuntimeError::ExpansionDepthExceeded {
                max: self.budget.max_expansion_depth,
            });
        }
        self.state.expansions =
            self.state
                .expansions
                .checked_add(1)
                .ok_or(RuntimeError::ExpansionCountExceeded {
                    max: self.budget.max_expansions,
                })?;
        if self.state.expansions > self.budget.max_expansions {
            return Err(RuntimeError::ExpansionCountExceeded {
                max: self.budget.max_expansions,
            });
        }
        self.expansion_stack.push(concept.clone());
        self.state.expansion_depth = self.expansion_stack.len();
        Ok(())
    }

    pub(super) fn leave_expansion(&mut self) {
        self.expansion_stack.pop();
        self.state.expansion_depth = self.expansion_stack.len();
    }

    pub(super) fn should_expand_concept(&self, concept: &CompiledConcept) -> bool {
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
}
