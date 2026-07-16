use crate::compiler::{CompileError, CompiledProgram, ResolvedExpression};
use crate::id::ProgramId;
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

    pub fn verify_compiled(
        &self,
        compiled: &CompiledProgram,
    ) -> Result<VerificationReport, CompileError> {
        let mut declarations = compiled
            .concepts
            .values()
            .cloned()
            .map(|concept| LinguaDeclaration::Concept(concept.declaration))
            .collect::<Vec<_>>();
        declarations.extend(
            compiled
                .functions
                .values()
                .cloned()
                .map(|function| LinguaDeclaration::Function(function.declaration)),
        );
        self.verify_program(&LinguaProgram {
            id: ProgramId::new("compiled"),
            declarations,
            entry: expression_to_source(&compiled.entry),
        })
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

fn expression_to_source(expression: &ResolvedExpression) -> LinguaExpression {
    match expression {
        ResolvedExpression::Concept(id) => LinguaExpression::Concept(id.clone()),
        ResolvedExpression::Entity(id) => LinguaExpression::Entity(id.clone()),
        ResolvedExpression::Value(value) => LinguaExpression::Value(value.clone()),
        ResolvedExpression::Local(symbol) => {
            LinguaExpression::Variable(crate::id::SymbolName::new(symbol.as_str()))
        }
        ResolvedExpression::QueryVariable(variable) => {
            LinguaExpression::QueryVariable(variable.clone())
        }
        ResolvedExpression::Function(function_id) => {
            LinguaExpression::Function(function_id.clone())
        }
        ResolvedExpression::Lambda { parameters, body } => LinguaExpression::Lambda {
            parameters: parameters
                .iter()
                .map(|parameter| crate::syntax::LambdaParameter {
                    name: crate::id::SymbolName::new(parameter.symbol.as_str()),
                    parameter_id: parameter.parameter_id.clone(),
                    value_type: parameter.value_type.clone(),
                })
                .collect(),
            body: Box::new(expression_to_source(body)),
        },
        ResolvedExpression::Call { callee, arguments } => LinguaExpression::Call {
            callee: Box::new(expression_to_source(callee)),
            arguments: arguments
                .iter()
                .map(|(parameter, value)| (parameter.clone(), expression_to_source(value)))
                .collect(),
        },
        ResolvedExpression::ApplyConcept { concept, bindings } => LinguaExpression::ApplyConcept {
            concept: concept.clone(),
            bindings: bindings
                .iter()
                .map(|(parameter, value)| (parameter.clone(), expression_to_source(value)))
                .collect(),
        },
        ResolvedExpression::Satisfies { subject, concept } => LinguaExpression::Satisfies {
            subject: Box::new(expression_to_source(subject)),
            concept: Box::new(expression_to_source(concept)),
        },
        ResolvedExpression::Equals { left, right } => LinguaExpression::Equals {
            left: Box::new(expression_to_source(left)),
            right: Box::new(expression_to_source(right)),
        },
        ResolvedExpression::And(items) => {
            LinguaExpression::And(items.iter().map(expression_to_source).collect())
        }
        ResolvedExpression::Or(items) => {
            LinguaExpression::Or(items.iter().map(expression_to_source).collect())
        }
        ResolvedExpression::Not(inner) => {
            LinguaExpression::Not(Box::new(expression_to_source(inner)))
        }
        ResolvedExpression::Exists {
            variable,
            value_type,
            body,
        } => LinguaExpression::Exists {
            variable: variable.clone(),
            value_type: value_type.clone(),
            body: Box::new(expression_to_source(body)),
        },
        ResolvedExpression::ForAll {
            variable,
            value_type,
            body,
        } => LinguaExpression::ForAll {
            variable: variable.clone(),
            value_type: value_type.clone(),
            body: Box::new(expression_to_source(body)),
        },
        ResolvedExpression::Let {
            symbol,
            value,
            body,
        } => LinguaExpression::Let {
            name: crate::id::SymbolName::new(symbol.as_str()),
            value: Box::new(expression_to_source(value)),
            body: Box::new(expression_to_source(body)),
        },
    }
}
