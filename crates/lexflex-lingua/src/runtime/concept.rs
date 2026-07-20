use crate::compiler::{CompiledConcept, CompiledConceptSemantics, ResolvedExpression};
use crate::runtime::{
    InterpreterState, RuntimeEnvironment, RuntimeError, RuntimeValue, TraceOperation,
};
use lexflex_model::{ConceptId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

impl InterpreterState<'_> {
    pub(super) fn eval_concept_application(
        &mut self,
        subject: Option<SemanticExpression>,
        concept_id: &ConceptId,
        bindings: &BTreeMap<ParameterId, ResolvedExpression>,
        environment: &RuntimeEnvironment,
    ) -> Result<RuntimeValue, RuntimeError> {
        let concept = self.program.concepts.get(concept_id);
        let evaluated_arguments = self.evaluate_concept_arguments(bindings, environment)?;
        let normalized = self.semantic_arguments(&evaluated_arguments)?;
        let application = SemanticExpression::Apply {
            concept: concept_id.clone(),
            bindings: normalized,
        };
        let Some(concept) = concept else {
            self.push_trace(
                TraceOperation::PreserveConceptApplication,
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
                TraceOperation::PreserveConceptApplication,
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
        if concept.self_parameter.is_some() && subject.is_none() {
            return Err(RuntimeError::MissingSelfSubject {
                concept: concept.declaration.concept_id.clone(),
            });
        }
        self.enter_expansion(&concept.declaration.concept_id)?;
        self.push_trace(
            TraceOperation::EnterConceptExpansion,
            "ApplyConcept",
            BTreeMap::from([
                (
                    "concept_id".into(),
                    concept.declaration.concept_id.to_string(),
                ),
                (
                    "binding_count".into(),
                    evaluated_arguments.len().to_string(),
                ),
            ]),
        )?;
        let child =
            self.bind_concept_arguments(concept, &evaluated_arguments, environment, subject)?;
        let result = self.eval(definition, &child);
        self.push_trace(
            TraceOperation::ExitConceptExpansion,
            "ApplyConcept",
            BTreeMap::from([(
                "concept_id".into(),
                concept.declaration.concept_id.to_string(),
            )]),
        )?;
        self.leave_expansion();
        result
    }

    fn evaluate_concept_arguments(
        &mut self,
        bindings: &BTreeMap<ParameterId, ResolvedExpression>,
        environment: &RuntimeEnvironment,
    ) -> Result<BTreeMap<ParameterId, RuntimeValue>, RuntimeError> {
        let mut values = BTreeMap::new();
        for (parameter, argument) in bindings {
            values.insert(parameter.clone(), self.eval(argument, environment)?);
        }
        Ok(values)
    }

    fn semantic_arguments(
        &self,
        values: &BTreeMap<ParameterId, RuntimeValue>,
    ) -> Result<BTreeMap<ParameterId, SemanticExpression>, RuntimeError> {
        let mut semantic = BTreeMap::new();
        for (parameter, value) in values {
            semantic.insert(parameter.clone(), value.clone().into_semantic()?);
        }
        Ok(semantic)
    }

    fn bind_concept_arguments(
        &mut self,
        concept: &CompiledConcept,
        bindings: &BTreeMap<ParameterId, RuntimeValue>,
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
                self.push_trace(
                    TraceOperation::BindSelfSubject,
                    "ApplyConcept",
                    BTreeMap::from([(
                        "concept_id".into(),
                        concept.declaration.concept_id.to_string(),
                    )]),
                )?;
            }
        }
        for parameter in &concept.parameters {
            let value = bindings
                .get(&parameter.parameter_id)
                .ok_or_else(|| RuntimeError::MissingArgument(parameter.parameter_id.clone()))?;
            child.insert(parameter.symbol.clone(), value.clone());
            self.push_trace(
                TraceOperation::BindConceptArgument,
                "ApplyConcept",
                BTreeMap::from([("parameter_id".into(), parameter.parameter_id.to_string())]),
            )?;
        }
        self.track_environment(&child)?;
        Ok(child)
    }
}
