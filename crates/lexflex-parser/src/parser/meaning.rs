use crate::token::Token;
use lexflex_language::{CompiledLexicalSense, FeatureStructure, SyntacticCategory};
use lexflex_lingua::LinguaExpression;
use lexflex_model::{SemanticType, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub(super) token: Token,
    pub(super) sense: CompiledLexicalSense,
    pub(super) category: SyntacticCategory,
    pub(super) meaning: MeaningInstance,
    pub(super) score: i64,
}

#[derive(Debug, Clone)]
pub(super) struct MeaningInstance {
    pub(super) expression: LinguaExpression,
    pub(super) query_variables: BTreeMap<VariableId, SemanticType>,
    pub(super) semantic_type: SemanticType,
}

pub(super) fn meaning_instance(sense: &CompiledLexicalSense) -> MeaningInstance {
    let mut query_variables = BTreeMap::new();
    let semantic_type = inferred_semantic_type(&sense.category);
    collect_query_variables(
        &sense.meaning.expression,
        &mut query_variables,
        semantic_type.clone(),
    );
    MeaningInstance {
        expression: sense.meaning.expression.clone(),
        query_variables,
        semantic_type,
    }
}

fn inferred_semantic_type(category: &SyntacticCategory) -> SemanticType {
    match category.semantic_type() {
        lexflex_language::CategoryType::Concrete(semantic_type) => semantic_type.clone(),
        lexflex_language::CategoryType::Variable(_) => SemanticType::Entity,
    }
}

pub(super) fn collect_query_variables(
    expression: &LinguaExpression,
    output: &mut BTreeMap<VariableId, SemanticType>,
    inferred_type: SemanticType,
) {
    match expression {
        LinguaExpression::QueryVariable(variable) => {
            output.insert(variable.clone(), inferred_type);
        }
        LinguaExpression::Lambda { body, .. }
        | LinguaExpression::Not(body)
        | LinguaExpression::Exists { body, .. }
        | LinguaExpression::ForAll { body, .. }
        | LinguaExpression::Let { body, .. } => {
            collect_query_variables(body, output, inferred_type);
        }
        LinguaExpression::Call { callee, arguments } => {
            collect_query_variables(callee, output, inferred_type.clone());
            for value in arguments.values() {
                collect_query_variables(value, output, inferred_type.clone());
            }
        }
        LinguaExpression::ApplyConcept { bindings, .. } => {
            for value in bindings.values() {
                collect_query_variables(value, output, inferred_type.clone());
            }
        }
        LinguaExpression::Satisfies { subject, concept }
        | LinguaExpression::Equals {
            left: subject,
            right: concept,
        } => {
            collect_query_variables(subject, output, inferred_type.clone());
            collect_query_variables(concept, output, inferred_type);
        }
        LinguaExpression::And(items) | LinguaExpression::Or(items) => {
            for item in items {
                collect_query_variables(item, output, inferred_type.clone());
            }
        }
        LinguaExpression::Concept(_)
        | LinguaExpression::Entity(_)
        | LinguaExpression::Value(_)
        | LinguaExpression::Variable(_)
        | LinguaExpression::Function(_) => {}
    }
}

pub(super) fn score_candidate(form_index: usize, sense: &CompiledLexicalSense) -> i64 {
    i64::try_from(form_index).unwrap_or(0) + i64::from(sense.priority)
}

pub(super) fn candidate_order(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    left.score
        .cmp(&right.score)
        .then_with(|| left.category.cmp(&right.category))
        .then_with(|| left.sense.id.cmp(&right.sense.id))
}

pub(super) trait CategoryFeatureMerge {
    fn unify_features(&self, features: &FeatureStructure) -> Option<SyntacticCategory>;
}

impl CategoryFeatureMerge for SyntacticCategory {
    fn unify_features(&self, features: &FeatureStructure) -> Option<SyntacticCategory> {
        match self {
            SyntacticCategory::Atom {
                kind,
                semantic_type,
                features: category_features,
            } => Some(SyntacticCategory::Atom {
                kind: kind.clone(),
                semantic_type: semantic_type.clone(),
                features: category_features.unify(features).ok()?,
            }),
            SyntacticCategory::Function {
                result,
                argument,
                direction,
                semantic_parameter,
                features: category_features,
            } => Some(SyntacticCategory::Function {
                result: result.clone(),
                argument: argument.clone(),
                direction: *direction,
                semantic_parameter: semantic_parameter.clone(),
                features: category_features.unify(features).ok()?,
            }),
        }
    }
}

pub(super) fn apply_meaning(function: &MeaningInstance, argument: &MeaningInstance) -> MeaningInstance {
    let expression = reduce_lambda_expression(&function.expression, &argument.expression);
    let mut query_variables = function.query_variables.clone();
    merge_query_variables(
        &mut query_variables,
        &argument.query_variables,
        &argument.semantic_type,
    );
    MeaningInstance {
        expression,
        query_variables,
        semantic_type: function.semantic_type.clone(),
    }
}

fn reduce_lambda_expression(
    expression: &LinguaExpression,
    argument: &LinguaExpression,
) -> LinguaExpression {
    match expression {
        LinguaExpression::Lambda { parameters, body } if !parameters.is_empty() => {
            let parameter = &parameters[0];
            let body = substitute_expression_symbol(body, parameter.name.as_str(), argument);
            if parameters.len() == 1 {
                body
            } else {
                LinguaExpression::Lambda {
                    parameters: parameters[1..].to_vec(),
                    body: Box::new(body),
                }
            }
        }
        _ => expression.clone(),
    }
}

fn substitute_expression_symbol(
    expression: &LinguaExpression,
    symbol: &str,
    argument: &LinguaExpression,
) -> LinguaExpression {
    match expression {
        LinguaExpression::Variable(name) if name.as_str() == symbol => argument.clone(),
        LinguaExpression::Lambda { parameters, body } => {
            if parameters
                .iter()
                .any(|parameter| parameter.name.as_str() == symbol)
            {
                LinguaExpression::Lambda {
                    parameters: parameters.clone(),
                    body: body.clone(),
                }
            } else {
                LinguaExpression::Lambda {
                    parameters: parameters.clone(),
                    body: Box::new(substitute_expression_symbol(body, symbol, argument)),
                }
            }
        }
        LinguaExpression::Call { callee, arguments } => LinguaExpression::Call {
            callee: Box::new(substitute_expression_symbol(callee, symbol, argument)),
            arguments: arguments
                .iter()
                .map(|(parameter, value)| {
                    (
                        parameter.clone(),
                        substitute_expression_symbol(value, symbol, argument),
                    )
                })
                .collect(),
        },
        LinguaExpression::ApplyConcept { concept, bindings } => LinguaExpression::ApplyConcept {
            concept: concept.clone(),
            bindings: bindings
                .iter()
                .map(|(parameter, value)| {
                    (
                        parameter.clone(),
                        substitute_expression_symbol(value, symbol, argument),
                    )
                })
                .collect(),
        },
        LinguaExpression::Satisfies { subject, concept } => LinguaExpression::Satisfies {
            subject: Box::new(substitute_expression_symbol(subject, symbol, argument)),
            concept: Box::new(substitute_expression_symbol(concept, symbol, argument)),
        },
        LinguaExpression::Equals { left, right } => LinguaExpression::Equals {
            left: Box::new(substitute_expression_symbol(left, symbol, argument)),
            right: Box::new(substitute_expression_symbol(right, symbol, argument)),
        },
        LinguaExpression::And(items) => LinguaExpression::And(
            items
                .iter()
                .map(|item| substitute_expression_symbol(item, symbol, argument))
                .collect(),
        ),
        LinguaExpression::Or(items) => LinguaExpression::Or(
            items
                .iter()
                .map(|item| substitute_expression_symbol(item, symbol, argument))
                .collect(),
        ),
        LinguaExpression::Not(inner) => LinguaExpression::Not(Box::new(
            substitute_expression_symbol(inner, symbol, argument),
        )),
        LinguaExpression::Exists {
            variable,
            value_type,
            body,
        } => LinguaExpression::Exists {
            variable: variable.clone(),
            value_type: value_type.clone(),
            body: Box::new(substitute_expression_symbol(body, symbol, argument)),
        },
        LinguaExpression::ForAll {
            variable,
            value_type,
            body,
        } => LinguaExpression::ForAll {
            variable: variable.clone(),
            value_type: value_type.clone(),
            body: Box::new(substitute_expression_symbol(body, symbol, argument)),
        },
        LinguaExpression::Let { name, value, body } => LinguaExpression::Let {
            name: name.clone(),
            value: Box::new(substitute_expression_symbol(value, symbol, argument)),
            body: Box::new(substitute_expression_symbol(body, symbol, argument)),
        },
        other => other.clone(),
    }
}

fn merge_query_variables(
    target: &mut BTreeMap<VariableId, SemanticType>,
    source: &BTreeMap<VariableId, SemanticType>,
    _inferred_type: &SemanticType,
) {
    for (variable, value_type) in source {
        target.insert(variable.clone(), value_type.clone());
    }
}
