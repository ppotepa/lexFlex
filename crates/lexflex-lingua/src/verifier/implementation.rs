use crate::compiler::{CompileError, ResolvedExpression};
use crate::syntax::{ConceptSemantics, LinguaDeclaration, LinguaExpression, LinguaProgram};
use crate::verifier::limits::{VerificationLimits, VerificationReport};
use crate::verifier::recursion::find_recursive_concepts;
use lexflex_model::ConceptId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct LinguaVerifier {
    limits: VerificationLimits,
}

impl LinguaVerifier {
    pub fn new(limits: VerificationLimits) -> Self {
        Self { limits }
    }

    pub fn verify_program(
        &self,
        program: &LinguaProgram,
    ) -> Result<VerificationReport, CompileError> {
        if program.declarations.len() > self.limits.max_declarations {
            return Err(CompileError::Diagnostic("too many declarations".into()));
        }

        let mut declaration_ids = BTreeSet::new();
        let mut concept_ids = BTreeSet::new();
        let mut graph: BTreeMap<ConceptId, Vec<ConceptId>> = BTreeMap::new();
        let mut expressions: Vec<&LinguaExpression> = vec![&program.entry];

        for declaration in &program.declarations {
            match declaration {
                LinguaDeclaration::Concept(concept) => {
                    if !concept_ids.insert(concept.concept_id.clone()) {
                        return Err(CompileError::Diagnostic(
                            format!("duplicate concept id: {}", concept.concept_id).into(),
                        ));
                    }
                    if !declaration_ids.insert(concept.declaration_id.clone()) {
                        return Err(CompileError::Diagnostic(
                            format!("duplicate declaration id: {}", concept.declaration_id).into(),
                        ));
                    }
                    if concept.parameters.len() > self.limits.max_parameters_per_function {
                        return Err(CompileError::Diagnostic(
                            "too many concept parameters".into(),
                        ));
                    }
                    let body = match &concept.semantics {
                        ConceptSemantics::Primitive => None,
                        ConceptSemantics::Defined { body } => Some(body),
                    };
                    graph.insert(concept.concept_id.clone(), collect_concepts(&body));
                    if let Some(definition) = body {
                        expressions.push(definition);
                    }
                }
                LinguaDeclaration::Function(function) => {
                    if !declaration_ids.insert(function.declaration_id.clone()) {
                        return Err(CompileError::Diagnostic(
                            format!("duplicate declaration id: {}", function.declaration_id).into(),
                        ));
                    }
                    if function.parameters.len() > self.limits.max_parameters_per_function {
                        return Err(CompileError::Diagnostic(
                            "too many function parameters".into(),
                        ));
                    }
                    expressions.push(&function.body);
                }
            }
        }

        if !self.limits.allow_recursive_concepts && !find_recursive_concepts(&graph).is_empty() {
            return Err(CompileError::Diagnostic(
                "recursive concept definitions are not allowed".into(),
            ));
        }

        let (expression_node_count, max_expression_depth) = expression_metrics(&expressions);
        let report = VerificationReport {
            declaration_count: program.declarations.len(),
            expression_node_count,
            max_expression_depth,
            concept_dependencies: graph,
        };

        if report.max_expression_depth > self.limits.max_expression_depth
            || report.expression_node_count > self.limits.max_expression_nodes
        {
            return Err(CompileError::Diagnostic(
                "expression limits exceeded".into(),
            ));
        }

        Ok(report)
    }

    pub fn verify_resolved_entry(
        &self,
        entry: &ResolvedExpression,
    ) -> Result<VerificationReport, CompileError> {
        let (expression_node_count, max_expression_depth) = resolved_expression_metrics(entry);
        let report = VerificationReport {
            declaration_count: 0,
            expression_node_count,
            max_expression_depth,
            concept_dependencies: BTreeMap::new(),
        };
        if report.max_expression_depth > self.limits.max_expression_depth
            || report.expression_node_count > self.limits.max_expression_nodes
        {
            return Err(CompileError::Diagnostic(
                "entry expression limits exceeded".into(),
            ));
        }
        Ok(report)
    }
}

fn collect_concepts(expression: &Option<&LinguaExpression>) -> Vec<ConceptId> {
    let mut concepts = BTreeSet::new();
    let mut stack = Vec::new();

    if let Some(expression) = expression.as_ref() {
        stack.push(*expression);
    }

    while let Some(expression) = stack.pop() {
        match expression {
            LinguaExpression::Concept(id) | LinguaExpression::ApplyConcept { concept: id, .. } => {
                concepts.insert(id.clone());
            }
            LinguaExpression::Lambda { body, .. }
            | LinguaExpression::Not(body)
            | LinguaExpression::Exists { body, .. }
            | LinguaExpression::ForAll { body, .. } => stack.push(body),
            LinguaExpression::Call { callee, arguments } => {
                stack.push(callee);
                for value in arguments.values() {
                    stack.push(value);
                }
            }
            LinguaExpression::Satisfies { subject, concept }
            | LinguaExpression::Equals {
                left: subject,
                right: concept,
            } => {
                stack.push(subject);
                stack.push(concept);
            }
            LinguaExpression::And(items) | LinguaExpression::Or(items) => {
                for item in items {
                    stack.push(item);
                }
            }
            LinguaExpression::Let { value, body, .. } => {
                stack.push(value);
                stack.push(body);
            }
            LinguaExpression::Entity(_)
            | LinguaExpression::Value(_)
            | LinguaExpression::Variable(_)
            | LinguaExpression::QueryVariable(_)
            | LinguaExpression::Function(_) => {}
        }
    }

    concepts.into_iter().collect()
}

fn expression_metrics(expressions: &[&LinguaExpression]) -> (usize, usize) {
    let mut nodes = 0usize;
    let mut max_depth = 0usize;
    for expression in expressions {
        let mut stack = vec![(*expression, 1usize)];

        while let Some((expression, depth)) = stack.pop() {
            nodes += 1;
            max_depth = max_depth.max(depth);
            match expression {
                LinguaExpression::Concept(_)
                | LinguaExpression::Entity(_)
                | LinguaExpression::Value(_)
                | LinguaExpression::Variable(_)
                | LinguaExpression::QueryVariable(_)
                | LinguaExpression::Function(_) => {}
                LinguaExpression::Lambda { body, .. }
                | LinguaExpression::Not(body)
                | LinguaExpression::Exists { body, .. }
                | LinguaExpression::ForAll { body, .. } => {
                    stack.push((body, depth + 1));
                }
                LinguaExpression::Call { callee, arguments } => {
                    stack.push((callee, depth + 1));
                    for value in arguments.values() {
                        stack.push((value, depth + 1));
                    }
                }
                LinguaExpression::ApplyConcept { bindings, .. } => {
                    for value in bindings.values() {
                        stack.push((value, depth + 1));
                    }
                }
                LinguaExpression::Satisfies { subject, concept }
                | LinguaExpression::Equals {
                    left: subject,
                    right: concept,
                } => {
                    stack.push((subject, depth + 1));
                    stack.push((concept, depth + 1));
                }
                LinguaExpression::And(items) | LinguaExpression::Or(items) => {
                    for item in items {
                        stack.push((item, depth + 1));
                    }
                }
                LinguaExpression::Let { value, body, .. } => {
                    stack.push((value, depth + 1));
                    stack.push((body, depth + 1));
                }
            }
        }
    }

    (nodes, max_depth)
}

fn resolved_expression_metrics(expression: &ResolvedExpression) -> (usize, usize) {
    let mut nodes = 0usize;
    let mut max_depth = 0usize;
    let mut stack = vec![(expression, 1usize)];

    while let Some((expression, depth)) = stack.pop() {
        nodes += 1;
        max_depth = max_depth.max(depth);
        match expression {
            ResolvedExpression::Concept(_)
            | ResolvedExpression::Entity(_)
            | ResolvedExpression::Value(_)
            | ResolvedExpression::Local(_)
            | ResolvedExpression::QueryVariable(_)
            | ResolvedExpression::Function(_) => {}
            ResolvedExpression::Lambda { body, .. }
            | ResolvedExpression::Not(body)
            | ResolvedExpression::Exists { body, .. }
            | ResolvedExpression::ForAll { body, .. } => stack.push((body, depth + 1)),
            ResolvedExpression::Call { callee, arguments } => {
                stack.push((callee, depth + 1));
                stack.extend(arguments.values().map(|value| (value, depth + 1)));
            }
            ResolvedExpression::ApplyConcept { bindings, .. } => {
                stack.extend(bindings.values().map(|value| (value, depth + 1)));
            }
            ResolvedExpression::Satisfies { subject, concept }
            | ResolvedExpression::Equals {
                left: subject,
                right: concept,
            } => {
                stack.push((subject, depth + 1));
                stack.push((concept, depth + 1));
            }
            ResolvedExpression::And(items) | ResolvedExpression::Or(items) => {
                stack.extend(items.iter().map(|item| (item, depth + 1)));
            }
            ResolvedExpression::Let { value, body, .. } => {
                stack.push((value, depth + 1));
                stack.push((body, depth + 1));
            }
        }
    }

    (nodes, max_depth)
}
