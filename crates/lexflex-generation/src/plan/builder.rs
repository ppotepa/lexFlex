use super::node::{GenerationNode, GenerationPlan};
use lexflex_language::LanguageModel;
use lexflex_model::{SemanticExpression, SemanticValue};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum PlanError {
    #[error("semantic expression cannot be canonically hashed")]
    CanonicalHash,
}

pub(crate) fn build_plan(
    expression: &SemanticExpression,
    language: &LanguageModel,
) -> Result<GenerationPlan, PlanError> {
    let root = node_for(expression);
    let semantic_hash = expression
        .canonical_hash()
        .map_err(|_| PlanError::CanonicalHash)?;
    Ok(GenerationPlan::new(
        root,
        language.manifest.language.clone(),
        semantic_hash,
    ))
}

fn node_for(expression: &SemanticExpression) -> GenerationNode {
    match expression {
        SemanticExpression::Value(SemanticValue::Boolean(_))
        | SemanticExpression::Value(SemanticValue::Integer(_))
        | SemanticExpression::Value(SemanticValue::Decimal(_))
        | SemanticExpression::Value(SemanticValue::Text(_))
        | SemanticExpression::Value(SemanticValue::Date(_))
        | SemanticExpression::Value(SemanticValue::Quantity(_)) => GenerationNode::Value,
        SemanticExpression::Satisfies { subject, predicate } => GenerationNode::Clause {
            subject: Box::new(node_for(subject)),
            predicate: Box::new(node_for(predicate)),
        },
        SemanticExpression::Apply { concept, bindings } => GenerationNode::Apply {
            concept: concept.clone(),
            bindings: bindings
                .iter()
                .map(|(parameter, value)| (parameter.clone(), node_for(value)))
                .collect(),
        },
        SemanticExpression::And(items) => GenerationNode::Coordination {
            conjunction: true,
            items: items.iter().map(node_for).collect(),
        },
        SemanticExpression::Or(items) => GenerationNode::Coordination {
            conjunction: false,
            items: items.iter().map(node_for).collect(),
        },
        SemanticExpression::Not(inner) => GenerationNode::Negated(Box::new(node_for(inner))),
        SemanticExpression::Exists { variable, body, .. } => GenerationNode::Quantified {
            universal: false,
            variable: variable.clone(),
            body: Box::new(node_for(body)),
        },
        SemanticExpression::ForAll { variable, body, .. } => GenerationNode::Quantified {
            universal: true,
            variable: variable.clone(),
            body: Box::new(node_for(body)),
        },
        SemanticExpression::Equals { left, right } => GenerationNode::Clause {
            subject: Box::new(node_for(left)),
            predicate: Box::new(node_for(right)),
        },
        other => GenerationNode::Lexical {
            expression: other.clone(),
        },
    }
}
