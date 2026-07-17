use crate::compiler::ResolvedExpression;
use crate::runtime::{ClosureValue, InterpreterState, RuntimeEnvironment, RuntimeError, RuntimeValue, TraceOperation};
use lexflex_model::ParameterId;
use std::{collections::BTreeMap, sync::Arc};

impl InterpreterState<'_> {
    pub(super) fn eval_call(
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
            self.track_environment(&child)?;
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
